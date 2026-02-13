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
        .route("/ws", get(crate::websocket::ws_handler))
        .layer(CorsLayer::permissive())
        .with_state(state);

    axum::serve(tokio::net::TcpListener::bind(addr).await?, app).await?;
    Ok(())
}
