//! Management API routes definition

use crate::web::api::{audit, auth, canary, instance, zone};
use crate::web::middleware;
use crate::web::state::ManagementState;
use axum::{
    middleware as axum_middleware,
    routing::{delete, get, patch, post},
    Router,
};
use tower_http::cors::CorsLayer;

/// Create management routes with authentication and authorization
pub fn management_routes(state: ManagementState) -> Router {
    // 公开路由 (无需认证)
    let public_routes = Router::new()
        .route("/api/auth/login", post(auth::login))
        .route("/api/auth/roles", get(auth::list_roles))
        .with_state(state.clone());

    // 受保护路由 (需要 JWT 认证)
    let protected_routes = Router::new()
        // ===== 认证相关 API =====
        .route("/api/auth/logout", post(auth::logout))
        .route("/api/auth/refresh", post(auth::refresh_token))
        .route("/api/auth/user", get(auth::get_current_user))
        .route("/api/auth/permissions", get(auth::get_user_permissions))
        .route("/api/auth/password/change", post(auth::change_password))
        .route(
            "/api/auth/password/reset/{user_id}",
            post(auth::reset_password),
        )
        .route("/api/auth/sessions", get(auth::list_sessions))
        .route(
            "/api/auth/sessions/{session_id}",
            delete(auth::revoke_session),
        )
        .route("/api/auth/check-permission", post(auth::check_permission))
        // 用户管理
        .route("/api/auth/users", get(auth::list_users))
        .route("/api/auth/users", post(auth::create_user))
        .route("/api/auth/users/{user_id}", get(auth::get_user))
        .route("/api/auth/users/{user_id}", patch(auth::update_user))
        .route("/api/auth/users/{user_id}", delete(auth::delete_user))
        .route(
            "/api/auth/users/{user_id}/status",
            patch(auth::update_user_status),
        )
        .route(
            "/api/auth/users/{user_id}/login-history",
            get(auth::get_login_history),
        )
        // ===== 实例操作 API =====
        .route(
            "/api/management/instance/operate-instance.json",
            post(instance::operate_instance),
        )
        .route(
            "/api/management/instance/get-instance-operations.json",
            post(instance::get_instance_operations),
        )
        .route(
            "/api/management/instance/is-instance-down.json",
            post(instance::is_instance_down),
        )
        .route(
            "/api/management/server/operate-server.json",
            post(instance::operate_server),
        )
        .route(
            "/api/management/server/is-server-down.json",
            post(instance::is_server_down),
        )
        .route(
            "/api/management/all-instance-operations.json",
            post(instance::get_all_instance_operations_post),
        )
        .route(
            "/api/management/all-instance-operations.json",
            get(instance::get_all_instance_operations_get),
        )
        .route(
            "/api/management/all-server-operations.json",
            post(instance::get_all_server_operations_post),
        )
        .route(
            "/api/management/all-server-operations.json",
            get(instance::get_all_server_operations_get),
        )
        // ===== Zone 管理 API =====
        .route("/api/management/zone/pull-out", post(zone::pull_out_zone))
        .route("/api/management/zone/pull-in", post(zone::pull_in_zone))
        .route(
            "/api/management/zone/status/{zone_id}/{region_id}",
            get(zone::get_zone_status),
        )
        .route(
            "/api/management/zone/operations",
            get(zone::list_zone_operations),
        )
        .route(
            "/api/management/zone/{zone_id}/{region_id}",
            delete(zone::delete_zone_operation),
        )
        // ===== 金丝雀发布 API =====
        .route(
            "/api/management/canary/config",
            post(canary::set_canary_config),
        )
        .route(
            "/api/management/canary/config/{service_id}",
            get(canary::get_canary_config),
        )
        .route("/api/management/canary/enable", post(canary::enable_canary))
        .route(
            "/api/management/canary/config/{service_id}",
            delete(canary::delete_canary_config),
        )
        .route(
            "/api/management/canary/configs",
            get(canary::list_canary_configs),
        )
        // ===== 审计日志 API =====
        .route("/api/management/audit/logs", get(audit::query_logs))
        .route(
            "/api/management/audit/instance-logs",
            get(audit::query_instance_logs),
        )
        .route(
            "/api/management/audit/server-logs",
            get(audit::query_server_logs),
        )
        .route(
            "/api/management/log/group-logs.json",
            post(audit::query_group_logs),
        )
        .route(
            "/api/management/log/route-rule-logs.json",
            post(audit::query_route_rule_logs),
        )
        .route(
            "/api/management/log/route-rule-group-logs.json",
            post(audit::query_route_rule_group_logs),
        )
        .route(
            "/api/management/log/zone-operation-logs.json",
            post(audit::query_zone_operation_logs),
        )
        .route(
            "/api/management/log/group-instance-logs.json",
            post(audit::query_group_instance_logs),
        )
        .route(
            "/api/management/log/service-instance-logs.json",
            post(audit::query_service_instance_logs),
        )
        // 应用 JWT 认证中间件
        .route_layer(axum_middleware::from_fn_with_state(
            state.clone(),
            middleware::jwt_auth,
        ))
        .with_state(state);

    // 合并公开路由和受保护路由
    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .layer(CorsLayer::permissive())
}
