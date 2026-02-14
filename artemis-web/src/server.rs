use crate::state::AppState;
use axum::{
    Router,
    routing::{get, post},
};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;

pub async fn run_server(state: AppState, addr: SocketAddr) -> anyhow::Result<()> {
    let app = Router::new()
        .route("/health", get(|| async { "OK" }))
        .route("/metrics", get(crate::api::metrics::metrics))
        .route("/api/registry/register", post(crate::api::registry::register))
        .route("/api/registry/register.json", post(crate::api::registry::register))
        .route("/api/registry/heartbeat", post(crate::api::registry::heartbeat))
        .route("/api/registry/heartbeat.json", post(crate::api::registry::heartbeat))
        .route("/api/registry/unregister", post(crate::api::registry::unregister))
        .route("/api/registry/unregister.json", post(crate::api::registry::unregister))
        .route("/api/discovery/service", post(crate::api::discovery::get_service))
        .route("/api/discovery/service.json", post(crate::api::discovery::get_service))
        .route("/api/discovery/services", get(crate::api::discovery::get_services))
        .route("/api/discovery/services.json", get(crate::api::discovery::get_services))
        // Replication endpoints
        .route("/api/replication/registry/register.json", post(crate::api::replication::replicate_register))
        .route("/api/replication/registry/heartbeat.json", post(crate::api::replication::replicate_heartbeat))
        .route("/api/replication/registry/unregister.json", post(crate::api::replication::replicate_unregister))
        .route("/api/replication/registry/services.json", get(crate::api::replication::get_all_services))
        .route("/ws", get(crate::websocket::ws_handler))
        .layer(CorsLayer::permissive())
        .with_state(state);

    tracing::info!("Starting Artemis Web Server on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;

    axum::serve(listener, app).with_graceful_shutdown(shutdown_signal()).await?;

    Ok(())
}

async fn shutdown_signal() {
    use tokio::signal;

    let ctrl_c = async {
        signal::ctrl_c().await.expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("Signal received, starting graceful shutdown");
}
