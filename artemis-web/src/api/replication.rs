use crate::state::AppState;
use artemis_core::model::*;
use artemis_core::traits::RegistryService;
use axum::{Json, extract::State, http::{HeaderMap, StatusCode}};

/// 复制-注册端点
///
/// 注意:此端点必须包含 X-Artemis-Replication header 以防止复制循环
pub async fn replicate_register(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<ReplicateRegisterRequest>,
) -> Result<Json<ReplicateRegisterResponse>, StatusCode> {
    // 检查复制标记 header
    if !headers.contains_key("x-artemis-replication") {
        tracing::warn!("Replication request without X-Artemis-Replication header");
        return Err(StatusCode::BAD_REQUEST);
    }

    let response = state.registry_service.register_from_replication(request).await;
    Ok(Json(response))
}

/// 复制-心跳端点
pub async fn replicate_heartbeat(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<ReplicateHeartbeatRequest>,
) -> Result<Json<ReplicateHeartbeatResponse>, StatusCode> {
    if !headers.contains_key("x-artemis-replication") {
        tracing::warn!("Replication request without X-Artemis-Replication header");
        return Err(StatusCode::BAD_REQUEST);
    }

    let response = state.registry_service.heartbeat_from_replication(request).await;
    Ok(Json(response))
}

/// 复制-注销端点
pub async fn replicate_unregister(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<ReplicateUnregisterRequest>,
) -> Result<Json<ReplicateUnregisterResponse>, StatusCode> {
    if !headers.contains_key("x-artemis-replication") {
        tracing::warn!("Replication request without X-Artemis-Replication header");
        return Err(StatusCode::BAD_REQUEST);
    }

    let response = state.registry_service.unregister_from_replication(request).await;
    Ok(Json(response))
}

/// 获取所有服务(用于新节点启动同步)
pub async fn get_all_services(
    State(state): State<AppState>,
) -> Json<GetAllServicesResponse> {
    let response = state.registry_service.get_all_services().await;
    Json(response)
}
