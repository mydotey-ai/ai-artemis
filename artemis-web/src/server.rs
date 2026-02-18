use crate::state::AppState;
use artemis_management::{management_routes, ManagementState};
use axum::{routing::delete, routing::get, routing::patch, routing::post, routing::put, Router};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;

pub async fn run_server(state: AppState, addr: SocketAddr) -> anyhow::Result<()> {
    // 核心服务路由 (注册、发现、复制、状态、监控)
    let core_routes = Router::new()
        .route("/health", get(|| async { "OK" }))
        .route("/metrics", get(crate::api::metrics::metrics))
        // Registry endpoints
        .route("/api/registry/register", post(crate::api::registry::register))
        .route("/api/registry/register.json", post(crate::api::registry::register))
        .route("/api/registry/heartbeat", post(crate::api::registry::heartbeat))
        .route("/api/registry/heartbeat.json", post(crate::api::registry::heartbeat))
        .route("/api/registry/unregister", post(crate::api::registry::unregister))
        .route("/api/registry/unregister.json", post(crate::api::registry::unregister))
        // Discovery endpoints
        .route(
            "/api/discovery/service",
            post(crate::api::discovery::get_service)
                .get(crate::api::discovery::get_service_by_query),
        )
        .route(
            "/api/discovery/service.json",
            post(crate::api::discovery::get_service)
                .get(crate::api::discovery::get_service_by_query),
        )
        .route(
            "/api/discovery/services",
            post(crate::api::discovery::get_services)
                .get(crate::api::discovery::get_services_by_query),
        )
        .route(
            "/api/discovery/services.json",
            post(crate::api::discovery::get_services)
                .get(crate::api::discovery::get_services_by_query),
        )
        .route("/api/discovery/lookup.json", post(crate::api::discovery::lookup_instance))
        // Replication endpoints
        .route(
            "/api/replication/registry/register.json",
            post(crate::api::replication::replicate_register),
        )
        .route(
            "/api/replication/registry/heartbeat.json",
            post(crate::api::replication::replicate_heartbeat),
        )
        .route(
            "/api/replication/registry/unregister.json",
            post(crate::api::replication::replicate_unregister),
        )
        .route(
            "/api/replication/registry/services.json",
            post(crate::api::replication::get_all_services)
                .get(crate::api::replication::get_all_services_by_query),
        )
        // Phase 23: 批量复制 API
        .route(
            "/api/replication/registry/batch-register.json",
            post(crate::api::replication::batch_register),
        )
        .route(
            "/api/replication/registry/batch-heartbeat.json",
            post(crate::api::replication::batch_heartbeat),
        )
        .route(
            "/api/replication/registry/batch-unregister.json",
            post(crate::api::replication::batch_unregister),
        )
        .route(
            "/api/replication/registry/services-delta.json",
            post(crate::api::replication::get_services_delta),
        )
        .route(
            "/api/replication/registry/sync-full.json",
            post(crate::api::replication::sync_full_data),
        )
        // Routing endpoints - Group management (保留在 artemis-web,因为需要访问 registry_service)
        .route("/api/routing/groups", post(crate::api::routing::create_group))
        .route("/api/routing/groups", get(crate::api::routing::list_groups))
        .route("/api/routing/groups/by-id/{group_id}", get(crate::api::routing::get_group))
        .route(
            "/api/routing/groups/{group_key}",
            delete(crate::api::routing::delete_group),
        )
        .route(
            "/api/routing/groups/{group_key}",
            put(crate::api::routing::update_group),
        )
        .route(
            "/api/routing/groups/{group_key}",
            patch(crate::api::routing::update_group),
        )
        .route("/api/routing/groups/{group_key}/tags", post(crate::api::routing::add_group_tags))
        .route("/api/routing/groups/{group_key}/tags", get(crate::api::routing::get_group_tags))
        .route(
            "/api/routing/groups/{group_key}/tags/{tag_key}",
            delete(crate::api::routing::remove_group_tag),
        )
        .route(
            "/api/routing/groups/{group_key}/instances",
            get(crate::api::routing::get_group_instances),
        )
        .route(
            "/api/routing/groups/{group_key}/instances",
            post(crate::api::routing::add_instance_to_group),
        )
        .route(
            "/api/routing/groups/{group_key}/instances/{instance_id}",
            delete(crate::api::routing::remove_instance_from_group),
        )
        .route(
            "/api/routing/services/{service_id}/instances",
            post(crate::api::routing::batch_add_service_instances),
        )
        // Routing endpoints - Rule management
        .route("/api/routing/rules", post(crate::api::routing::create_rule))
        .route("/api/routing/rules", get(crate::api::routing::list_rules))
        .route("/api/routing/rules/{rule_id}", get(crate::api::routing::get_rule))
        .route(
            "/api/routing/rules/{rule_id}",
            delete(crate::api::routing::delete_rule),
        )
        .route(
            "/api/routing/rules/{rule_id}",
            put(crate::api::routing::update_rule),
        )
        .route(
            "/api/routing/rules/{rule_id}",
            patch(crate::api::routing::update_rule),
        )
        .route("/api/routing/rules/{rule_id}/publish", post(crate::api::routing::publish_rule))
        .route(
            "/api/routing/rules/{rule_id}/unpublish",
            post(crate::api::routing::unpublish_rule),
        )
        // Routing endpoints - Rule group association
        .route("/api/routing/rules/{rule_id}/groups", post(crate::api::routing::add_rule_group))
        .route("/api/routing/rules/{rule_id}/groups", get(crate::api::routing::get_rule_groups))
        .route(
            "/api/routing/rules/{rule_id}/groups/{group_id}",
            delete(crate::api::routing::remove_rule_group),
        )
        .route(
            "/api/routing/rules/{rule_id}/groups/{group_id}",
            put(crate::api::routing::update_rule_group),
        )
        .route(
            "/api/routing/rules/{rule_id}/groups/{group_id}",
            patch(crate::api::routing::update_rule_group),
        )
        // Status endpoints
        .route("/api/status/node.json", post(crate::api::status::get_cluster_node_status_post))
        .route("/api/status/node.json", get(crate::api::status::get_cluster_node_status_get))
        .route("/api/status/cluster.json", post(crate::api::status::get_cluster_status_post))
        .route("/api/status/cluster.json", get(crate::api::status::get_cluster_status_get))
        .route("/api/status/leases.json", post(crate::api::status::get_leases_status_post))
        .route("/api/status/leases.json", get(crate::api::status::get_leases_status_get))
        .route(
            "/api/status/legacy-leases.json",
            post(crate::api::status::get_legacy_leases_status_post),
        )
        .route(
            "/api/status/legacy-leases.json",
            get(crate::api::status::get_legacy_leases_status_get),
        )
        .route("/api/status/config.json", post(crate::api::status::get_config_status_post))
        .route("/api/status/config.json", get(crate::api::status::get_config_status_get))
        .route(
            "/api/status/deployment.json",
            post(crate::api::status::get_deployment_status_post),
        )
        .route(
            "/api/status/deployment.json",
            get(crate::api::status::get_deployment_status_get),
        )
        // WebSocket endpoint
        .route("/ws", get(crate::websocket::ws_handler))
        .with_state(state.clone());

    // 创建管理状态 (从 AppState 中提取管理相关的 managers)
    let management_state = ManagementState::new(
        state.auth_manager.clone(),
        state.instance_manager.clone(),
        state.group_manager.clone(),
        state.route_manager.clone(),
        state.zone_manager.clone(),
        state.canary_manager.clone(),
        state.audit_manager.clone(),
    );

    // 管理路由 (从 artemis-management crate)
    let mgmt_routes = management_routes(management_state);

    // 合并所有路由
    let app = Router::new()
        .merge(core_routes)
        .merge(mgmt_routes)
        .layer(CorsLayer::permissive());

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
