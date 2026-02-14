use artemis_server::{
    RegistryServiceImpl, cache::VersionedCacheManager, discovery::DiscoveryServiceImpl,
    lease::LeaseManager, registry::RegistryRepository,
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
        #[arg(short, long, default_value = "0.0.0.0:8080")]
        addr: String,
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
        Commands::Server { addr } => {
            println!("Starting Artemis server on {}", addr);
            start_server(addr.parse()?).await
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

async fn start_server(addr: SocketAddr) -> anyhow::Result<()> {
    let repository = RegistryRepository::new();
    let lease_manager = Arc::new(LeaseManager::new(Duration::from_secs(30)));
    let cache = Arc::new(VersionedCacheManager::new());
    let change_manager = Arc::new(artemis_server::InstanceChangeManager::new());

    let registry_service = Arc::new(RegistryServiceImpl::new(
        repository.clone(),
        lease_manager.clone(),
        change_manager,
    ));

    let discovery_service = Arc::new(DiscoveryServiceImpl::new(repository, cache.clone()));

    let session_manager = Arc::new(artemis_web::websocket::SessionManager::new());

    let state = AppState { registry_service, discovery_service, cache, session_manager };

    println!("Artemis server listening on {}", addr);
    run_server(state, addr).await
}
