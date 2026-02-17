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
        .route("/api/discovery/service", post(crate::api::discovery::get_service).get(crate::api::discovery::get_service_by_query))
        .route("/api/discovery/service.json", post(crate::api::discovery::get_service).get(crate::api::discovery::get_service_by_query))
        .route("/api/discovery/services", post(crate::api::discovery::get_services).get(crate::api::discovery::get_services_by_query))
        .route("/api/discovery/services.json", post(crate::api::discovery::get_services).get(crate::api::discovery::get_services_by_query))
        .route("/api/discovery/lookup.json", post(crate::api::discovery::lookup_instance))
        // Replication endpoints
        .route("/api/replication/registry/register.json", post(crate::api::replication::replicate_register))
        .route("/api/replication/registry/heartbeat.json", post(crate::api::replication::replicate_heartbeat))
        .route("/api/replication/registry/unregister.json", post(crate::api::replication::replicate_unregister))
        .route("/api/replication/registry/services.json", post(crate::api::replication::get_all_services).get(crate::api::replication::get_all_services_by_query))
        // Phase 23: 批量复制 API
        .route("/api/replication/registry/batch-register.json", post(crate::api::replication::batch_register))
        .route("/api/replication/registry/batch-heartbeat.json", post(crate::api::replication::batch_heartbeat))
        .route("/api/replication/registry/batch-unregister.json", post(crate::api::replication::batch_unregister))
        .route("/api/replication/registry/services-delta.json", post(crate::api::replication::get_services_delta))
        .route("/api/replication/registry/sync-full.json", post(crate::api::replication::sync_full_data))
        // Management endpoints - Instance operations
        .route("/api/management/instance/operate-instance.json", post(crate::api::management::operate_instance))
        .route("/api/management/instance/get-instance-operations.json", post(crate::api::management::get_instance_operations))
        .route("/api/management/instance/is-instance-down.json", post(crate::api::management::is_instance_down))
        // Management endpoints - Server operations
        .route("/api/management/server/operate-server.json", post(crate::api::management::operate_server))
        .route("/api/management/server/is-server-down.json", post(crate::api::management::is_server_down))
        // Management endpoints - Batch query operations (Phase 25)
        .route("/api/management/all-instance-operations.json", post(crate::api::management::get_all_instance_operations_post))
        .route("/api/management/all-instance-operations.json", get(crate::api::management::get_all_instance_operations_get))
        .route("/api/management/all-server-operations.json", post(crate::api::management::get_all_server_operations_post))
        .route("/api/management/all-server-operations.json", get(crate::api::management::get_all_server_operations_get))
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
        .route("/api/routing/groups/{group_key}/instances", post(crate::api::routing::add_instance_to_group))
        .route("/api/routing/groups/{group_key}/instances/{instance_id}", axum::routing::delete(crate::api::routing::remove_instance_from_group))
        .route("/api/routing/services/{service_id}/instances", post(crate::api::routing::batch_add_service_instances))
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
        // Phase 24: 审计日志细分 API
        .route("/api/management/log/group-logs.json", post(crate::api::audit::query_group_logs))
        .route("/api/management/log/route-rule-logs.json", post(crate::api::audit::query_route_rule_logs))
        .route("/api/management/log/route-rule-group-logs.json", post(crate::api::audit::query_route_rule_group_logs))
        .route("/api/management/log/zone-operation-logs.json", post(crate::api::audit::query_zone_operation_logs))
        .route("/api/management/log/group-instance-logs.json", post(crate::api::audit::query_group_instance_logs))
        .route("/api/management/log/service-instance-logs.json", post(crate::api::audit::query_service_instance_logs))
        // Auth endpoints - Public (no JWT required)
        .route("/api/auth/login", post(crate::api::auth::login))
        .route("/api/auth/roles", get(crate::api::auth::list_roles))
        // Auth endpoints - Protected (JWT required)
        .route("/api/auth/logout", post(crate::api::auth::logout))
        .route("/api/auth/refresh", post(crate::api::auth::refresh_token))
        .route("/api/auth/user", get(crate::api::auth::get_current_user))
        .route("/api/auth/permissions", get(crate::api::auth::get_user_permissions))
        .route("/api/auth/password/change", post(crate::api::auth::change_password))
        .route("/api/auth/password/reset/:user_id", post(crate::api::auth::reset_password))
        .route("/api/auth/sessions", get(crate::api::auth::list_sessions))
        .route("/api/auth/sessions/:session_id", axum::routing::delete(crate::api::auth::revoke_session))
        .route("/api/auth/check-permission", post(crate::api::auth::check_permission))
        .route("/api/auth/users", get(crate::api::auth::list_users).post(crate::api::auth::create_user))
        .route("/api/auth/users/:user_id", get(crate::api::auth::get_user))
        .route("/api/auth/users/:user_id", axum::routing::put(crate::api::auth::update_user))
        .route("/api/auth/users/:user_id", axum::routing::delete(crate::api::auth::delete_user))
        .route("/api/auth/users/:user_id/status", axum::routing::patch(crate::api::auth::update_user_status))
        .route("/api/auth/users/:user_id/login-history", get(crate::api::auth::get_login_history))
        // Status endpoints
        .route("/api/status/node.json", post(crate::api::status::get_cluster_node_status_post))
        .route("/api/status/node.json", get(crate::api::status::get_cluster_node_status_get))
        .route("/api/status/cluster.json", post(crate::api::status::get_cluster_status_post))
        .route("/api/status/cluster.json", get(crate::api::status::get_cluster_status_get))
        .route("/api/status/leases.json", post(crate::api::status::get_leases_status_post))
        .route("/api/status/leases.json", get(crate::api::status::get_leases_status_get))
        .route("/api/status/legacy-leases.json", post(crate::api::status::get_legacy_leases_status_post))
        .route("/api/status/legacy-leases.json", get(crate::api::status::get_legacy_leases_status_get))
        .route("/api/status/config.json", post(crate::api::status::get_config_status_post))
        .route("/api/status/config.json", get(crate::api::status::get_config_status_get))
        .route("/api/status/deployment.json", post(crate::api::status::get_deployment_status_post))
        .route("/api/status/deployment.json", get(crate::api::status::get_deployment_status_get))
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
