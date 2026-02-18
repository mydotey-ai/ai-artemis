// 使用 mimalloc 作为全局内存分配器
// mimalloc 是 Microsoft 开发的高性能分配器，适合长时间运行的服务器程序
// 特点：低延迟、低碎片、优秀的多线程扩展性
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use artemis_management::auth::UserRole;
use artemis_management::{
    AuthManager, ConfigLoader, Database, GroupManager, GroupRoutingFilter, InstanceManager,
    ManagementDiscoveryFilter, RouteEngine, RouteManager,
};
use artemis_core::model::Service;
use artemis_server::config::ArtemisConfig;
use artemis_server::{
    RegistryServiceImpl, cache::VersionedCacheManager, cluster::ClusterManager,
    discovery::DiscoveryServiceImpl, lease::LeaseManager, registry::RegistryRepository,
    replication::ReplicationManager,
};
use tracing::info;
use artemis_web::{server::run_server, state::AppState};
use clap::{Parser, Subcommand};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

#[derive(Parser)]
#[command(name = "artemis")]
#[command(about = "Artemis Service Registry")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Server {
        /// Configuration file path (TOML format)
        #[arg(short, long)]
        config: Option<String>,

        /// Listen address (overrides config file)
        #[arg(short, long)]
        addr: Option<String>,
    },
    Service {
        #[command(subcommand)]
        action: ServiceAction,
    },
    Instance {
        #[command(subcommand)]
        action: InstanceAction,
    },
    ConvertConfig {
        input: String,
        output: String,
    },
}

#[derive(Subcommand)]
enum ServiceAction {
    List,
}

#[derive(Subcommand)]
enum InstanceAction {
    List { service_id: String },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Server { config, addr } => start_server(config, addr).await,
        Commands::Service { action } => match action {
            ServiceAction::List => {
                println!("Listing services...");
                Ok(())
            }
        },
        Commands::Instance { action } => match action {
            InstanceAction::List { service_id } => {
                println!("Listing instances for {}", service_id);
                Ok(())
            }
        },
        Commands::ConvertConfig { input, output } => {
            println!("Converting {} to {}", input, output);
            Ok(())
        }
    }
}

async fn start_server(
    config_path: Option<String>,
    addr_override: Option<String>,
) -> anyhow::Result<()> {
    // 1. Load configuration
    let config = if let Some(path) = config_path {
        println!("Loading configuration from {}", path);
        ArtemisConfig::from_file(&path)?
    } else {
        println!("Using default configuration");
        ArtemisConfig::default()
    };

    // 2. Determine listen address
    let listen_addr: SocketAddr =
        if let Some(addr_str) = addr_override { addr_str.parse()? } else { config.listen_addr() };

    println!("Node ID: {}", config.server.node_id);
    println!("Region: {}, Zone: {}", config.server.region, config.server.zone);
    println!("Cluster mode: {}", if config.cluster.enabled { "enabled" } else { "disabled" });

    // 3a. Initialize database (optional)
    let database = if let Some(db_config) = &config.database {
        println!("Initializing database: {} (type: {})", db_config.url, db_config.db_type);
        let db = Arc::new(Database::new(&db_config.url, db_config.max_connections).await?);

        // 运行数据库迁移
        db.run_migrations().await?;
        println!("Database migrations completed");

        Some(db)
    } else {
        println!("Database disabled - using in-memory storage only");
        None
    };

    // 3. Initialize core components
    let repository = RegistryRepository::new();
    let lease_manager = Arc::new(LeaseManager::new(Duration::from_secs(config.lease.ttl_secs)));
    let cache = Arc::new(VersionedCacheManager::new());
    let change_manager = Arc::new(artemis_server::InstanceChangeManager::new());

    // 4. Initialize cluster components (if enabled)
    let (cluster_manager, replication_manager) = if config.cluster.enabled {
        println!("Initializing cluster components...");

        // 创建集群管理器
        let cluster = Arc::new(ClusterManager::new(
            config.server.node_id.clone(),
            config.cluster.peers.clone().unwrap_or_default(),
        ));

        // 启动健康检查任务
        cluster.clone().start_health_check_task();

        // 创建复制管理器和工作器
        let (repl_mgr, event_rx) = ReplicationManager::new();
        let repl_mgr = Arc::new(repl_mgr);

        // 启动复制工作器
        ReplicationManager::start_worker(event_rx, cluster.clone(), config.replication.clone());

        println!("Cluster initialized with {} peers", cluster.node_count());

        (Some(cluster), Some(repl_mgr))
    } else {
        (None, None)
    };

    // 5. Create services (with cache and replication support)
    let registry_service = Arc::new(RegistryServiceImpl::new(
        repository.clone(),
        lease_manager.clone(),
        cache.clone(),
        change_manager.clone(),
        replication_manager.clone(),
    ));

    // 5a. Start lease eviction task (清理过期实例)
    {
        let repository = repository.clone();
        let cache = cache.clone();
        let change_manager = change_manager.clone();
        let replication_manager = replication_manager.clone();
        let cleanup_interval = Duration::from_secs(config.lease.cleanup_interval_secs);

        lease_manager.clone().start_eviction_task(cleanup_interval, move |key| {
            info!("Evicting expired instance: {:?}", key);
            let service_id = key.service_id.clone();

            // 从 repository 移除实例
            if let Some(instance) = repository.remove(&key) {
                // 更新缓存
                let instances = repository.get_instances_by_service(&service_id);
                if instances.is_empty() {
                    // 没有实例，删除缓存
                    cache.remove_service(&service_id);
                } else {
                    // 有实例，更新缓存
                    let service = Service {
                        service_id: service_id.clone(),
                        metadata: None,
                        instances,
                        logic_instances: None,
                    };
                    cache.update_service(service);
                }

                // 发布变更事件
                change_manager.publish_unregister(&key, &instance);

                // 触发复制
                if let Some(ref repl_mgr) = replication_manager {
                    repl_mgr.publish_unregister(key.clone());
                }
            }
        });
    }

    // 6. Initialize management components
    let instance_manager = Arc::new(InstanceManager::new());

    // 7. Initialize routing components (with optional database)
    let group_manager = Arc::new(GroupManager::with_database(database.clone()));
    let route_manager = Arc::new(RouteManager::with_database(database.clone()));
    let zone_manager = Arc::new(artemis_management::ZoneManager::with_database(database.clone()));
    let canary_manager =
        Arc::new(artemis_management::CanaryManager::with_database(database.clone()));
    let audit_manager = Arc::new(artemis_management::AuditManager::new());
    let route_engine = Arc::new(RouteEngine::new());

    // Initialize authentication manager
    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "artemis-default-secret-change-in-production".to_string());
    let auth_manager = Arc::new(AuthManager::with_database(database.clone(), jwt_secret));

    // Load auth data from database
    if let Err(e) = auth_manager.load_from_database().await {
        println!("Warning: Failed to load auth data from database: {:?}", e);
    }

    // Create default admin user if database is empty
    if auth_manager.list_users().is_empty() {
        println!("Creating default admin user (username: admin, password: admin123)");
        match auth_manager.create_user(
            "admin",
            Some("admin@artemis.local".to_string()),
            Some("Default administrator account".to_string()),
            "admin123",
            UserRole::Admin,
        ) {
            Ok(_) => println!("Default admin user created successfully"),
            Err(e) => println!("Warning: Failed to create default admin user: {}", e),
        }
    }

    // 7a. Load persisted configurations from database
    if let Some(ref db) = database {
        println!("Loading persisted configurations from database...");
        let loader = ConfigLoader::new(
            db.clone(),
            group_manager.clone(),
            route_manager.clone(),
            zone_manager.clone(),
            canary_manager.clone(),
        );
        loader.load_all().await?;
        println!("Configurations loaded successfully");
    }

    // 8. Create discovery service with filters
    let mut discovery_service = DiscoveryServiceImpl::new(repository, cache.clone());

    // Add management filter (pull-in/pull-out)
    discovery_service.add_filter(Arc::new(ManagementDiscoveryFilter::new(
        instance_manager.clone(),
    )));

    // Add group routing filter
    discovery_service.add_filter(Arc::new(GroupRoutingFilter::new(
        route_manager.clone(),
        route_engine.clone(),
    )));

    let discovery_service = Arc::new(discovery_service);

    let session_manager = Arc::new(artemis_web::websocket::SessionManager::new());

    // Load balancer for discovery lookup
    let load_balancer = Arc::new(artemis_server::discovery::LoadBalancer::new());

    // Status service
    let status_service = Arc::new(artemis_server::StatusService::new(
        cluster_manager.clone(),
        lease_manager,
        config.server.node_id.clone(),
        config.server.region.clone(),
        config.server.zone.clone(),
        format!("http://{}", config.server.listen_addr),
        "artemis".to_string(), // app_id
    ));

    // 9. Create AppState
    let state = AppState {
        registry_service,
        discovery_service,
        cache,
        session_manager,
        cluster_manager,
        replication_manager,
        instance_manager,
        group_manager,
        route_manager,
        zone_manager,
        canary_manager,
        audit_manager,
        auth_manager,
        load_balancer,
        status_service,
    };

    // 10. Start server
    println!("Artemis server listening on {}", listen_addr);
    run_server(state, listen_addr).await
}
