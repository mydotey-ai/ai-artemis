use artemis_core::config::ArtemisConfig;
use artemis_server::{
    RegistryServiceImpl, cache::VersionedCacheManager, discovery::DiscoveryServiceImpl,
    lease::LeaseManager, registry::RegistryRepository,
    cluster::ClusterManager, replication::ReplicationManager,
};
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
        Commands::Server { config, addr } => {
            start_server(config, addr).await
        }
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

async fn start_server(config_path: Option<String>, addr_override: Option<String>) -> anyhow::Result<()> {
    // 1. Load configuration
    let config = if let Some(path) = config_path {
        println!("Loading configuration from {}", path);
        ArtemisConfig::from_file(&path)?
    } else {
        println!("Using default configuration");
        ArtemisConfig::default()
    };

    // 2. Determine listen address
    let listen_addr: SocketAddr = if let Some(addr_str) = addr_override {
        addr_str.parse()?
    } else {
        config.listen_addr()
    };

    println!("Node ID: {}", config.server.node_id);
    println!("Region: {}, Zone: {}", config.server.region, config.server.zone);
    println!("Cluster mode: {}", if config.cluster.enabled { "enabled" } else { "disabled" });

    // 3. Initialize core components
    let repository = RegistryRepository::new();
    let lease_manager = Arc::new(LeaseManager::new(
        Duration::from_secs(config.lease.ttl_secs)
    ));
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
        ReplicationManager::start_worker(
            event_rx,
            cluster.clone(),
            config.replication.clone(),
        );

        println!("Cluster initialized with {} peers", cluster.node_count());

        (Some(cluster), Some(repl_mgr))
    } else {
        (None, None)
    };

    // 5. Create services (with replication support)
    let registry_service = Arc::new(RegistryServiceImpl::new(
        repository.clone(),
        lease_manager.clone(),
        change_manager,
        replication_manager.clone(),
    ));

    let discovery_service = Arc::new(DiscoveryServiceImpl::new(repository, cache.clone()));

    let session_manager = Arc::new(artemis_web::websocket::SessionManager::new());

    // 6. Create AppState
    let state = AppState {
        registry_service,
        discovery_service,
        cache,
        session_manager,
        cluster_manager,
        replication_manager,
    };

    // 7. Start server
    println!("Artemis server listening on {}", listen_addr);
    run_server(state, listen_addr).await
}
