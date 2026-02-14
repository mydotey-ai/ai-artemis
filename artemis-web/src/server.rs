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
        // Management endpoints - Instance operations
        .route("/api/management/instance/operate-instance.json", post(crate::api::management::operate_instance))
        .route("/api/management/instance/get-instance-operations.json", post(crate::api::management::get_instance_operations))
        .route("/api/management/instance/is-instance-down.json", post(crate::api::management::is_instance_down))
        // Management endpoints - Server operations
        .route("/api/management/server/operate-server.json", post(crate::api::management::operate_server))
        .route("/api/management/server/is-server-down.json", post(crate::api::management::is_server_down))
        // Routing endpoints - Group management
        .route("/api/routing/groups", post(crate::api::routing::create_group))
        .route("/api/routing/groups", get(crate::api::routing::list_groups))
        .route("/api/routing/groups/by-id/{group_id}", get(crate::api::routing::get_group))
        .route("/api/routing/groups/{group_key}", axum::routing::delete(crate::api::routing::delete_group))
        .route("/api/routing/groups/{group_key}", axum::routing::patch(crate::api::routing::update_group))
        .route("/api/routing/groups/{group_key}/tags", post(crate::api::routing::add_group_tags))
        .route("/api/routing/groups/{group_key}/tags", get(crate::api::routing::get_group_tags))
        .route("/api/routing/groups/{group_key}/tags/{tag_key}", axum::routing::delete(crate::api::routing::remove_group_tag))
        .route("/api/routing/groups/{group_key}/instances", get(crate::api::routing::get_group_instances))
        // Routing endpoints - Rule management
        .route("/api/routing/rules", post(crate::api::routing::create_rule))
        .route("/api/routing/rules", get(crate::api::routing::list_rules))
        .route("/api/routing/rules/{rule_id}", get(crate::api::routing::get_rule))
        .route("/api/routing/rules/{rule_id}", axum::routing::delete(crate::api::routing::delete_rule))
        .route("/api/routing/rules/{rule_id}", axum::routing::patch(crate::api::routing::update_rule))
        .route("/api/routing/rules/{rule_id}/publish", post(crate::api::routing::publish_rule))
        .route("/api/routing/rules/{rule_id}/unpublish", post(crate::api::routing::unpublish_rule))
        // Routing endpoints - Rule group association
        .route("/api/routing/rules/{rule_id}/groups", post(crate::api::routing::add_rule_group))
        .route("/api/routing/rules/{rule_id}/groups", get(crate::api::routing::get_rule_groups))
        .route("/api/routing/rules/{rule_id}/groups/{group_id}", axum::routing::delete(crate::api::routing::remove_rule_group))
        .route("/api/routing/rules/{rule_id}/groups/{group_id}", axum::routing::patch(crate::api::routing::update_rule_group))
        // Zone management endpoints
        .route("/api/management/zone/pull-out", post(crate::api::zone::pull_out_zone))
        .route("/api/management/zone/pull-in", post(crate::api::zone::pull_in_zone))
        .route("/api/management/zone/status/{zone_id}/{region_id}", get(crate::api::zone::get_zone_status))
        .route("/api/management/zone/operations", get(crate::api::zone::list_zone_operations))
        .route("/api/management/zone/{zone_id}/{region_id}", axum::routing::delete(crate::api::zone::delete_zone_operation))
        // Canary release endpoints
        .route("/api/management/canary/config", post(crate::api::canary::set_canary_config))
        .route("/api/management/canary/config/{service_id}", get(crate::api::canary::get_canary_config))
        .route("/api/management/canary/enable", post(crate::api::canary::enable_canary))
        .route("/api/management/canary/config/{service_id}", axum::routing::delete(crate::api::canary::delete_canary_config))
        .route("/api/management/canary/configs", get(crate::api::canary::list_canary_configs))
        // Audit log endpoints
        .route("/api/management/audit/logs", get(crate::api::audit::query_logs))
        .route("/api/management/audit/instance-logs", get(crate::api::audit::query_instance_logs))
        .route("/api/management/audit/server-logs", get(crate::api::audit::query_server_logs))
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
