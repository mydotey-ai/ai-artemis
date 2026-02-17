use crate::state::AppState;
use artemis_core::model::*;
use artemis_server::traits::RegistryService;
use axum::{
    Json,
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
};
use serde::Deserialize;

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

/// 获取所有服务(用于新节点启动同步) - POST 版本
pub async fn get_all_services(State(state): State<AppState>) -> Json<GetAllServicesResponse> {
    let response = state.registry_service.get_all_services().await;
    Json(response)
}

// ===== GET All Services (GET 版本 - Phase 22 新增) =====

#[derive(Debug, Deserialize)]
pub struct GetAllServicesQuery {
    #[serde(rename = "regionId")]
    pub region_id: String,
    #[serde(rename = "zoneId")]
    pub zone_id: Option<String>,
}

/// 获取所有服务 - GET 版本 (支持 regionId 和 zoneId 查询参数)
pub async fn get_all_services_by_query(
    State(state): State<AppState>,
    Query(_query): Query<GetAllServicesQuery>,
) -> Json<GetAllServicesResponse> {
    // 注意: Java 版本虽然接受 regionId/zoneId 参数,但实际返回所有服务
    // 这里保持与 Java 版本一致的行为
    let response = state.registry_service.get_all_services().await;
    Json(response)
}

// ===== Phase 23: 批量复制 API =====

/// 批量注册端点
pub async fn batch_register(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<BatchRegisterRequest>,
) -> Result<Json<BatchRegisterResponse>, StatusCode> {
    if !headers.contains_key("x-artemis-replication") {
        tracing::warn!("Batch register request without X-Artemis-Replication header");
        return Err(StatusCode::BAD_REQUEST);
    }

    let response = state.registry_service.batch_register(request).await;
    Ok(Json(response))
}

/// 批量心跳端点
pub async fn batch_heartbeat(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<BatchHeartbeatRequest>,
) -> Result<Json<BatchHeartbeatResponse>, StatusCode> {
    if !headers.contains_key("x-artemis-replication") {
        tracing::warn!("Batch heartbeat request without X-Artemis-Replication header");
        return Err(StatusCode::BAD_REQUEST);
    }

    let response = state.registry_service.batch_heartbeat(request).await;
    Ok(Json(response))
}

/// 批量注销端点
pub async fn batch_unregister(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<BatchUnregisterRequest>,
) -> Result<Json<BatchUnregisterResponse>, StatusCode> {
    if !headers.contains_key("x-artemis-replication") {
        tracing::warn!("Batch unregister request without X-Artemis-Replication header");
        return Err(StatusCode::BAD_REQUEST);
    }

    let response = state.registry_service.batch_unregister(request).await;
    Ok(Json(response))
}

/// 增量同步端点 - 获取指定时间戳之后的服务变更
pub async fn get_services_delta(
    State(state): State<AppState>,
    Json(request): Json<ServicesDeltaRequest>,
) -> Json<ServicesDeltaResponse> {
    let response = state.registry_service.get_services_delta(request).await;
    Json(response)
}

/// 全量同步端点 - 新节点加入时的完整数据同步
pub async fn sync_full_data(
    State(state): State<AppState>,
    Json(request): Json<SyncFullDataRequest>,
) -> Json<SyncFullDataResponse> {
    let response = state.registry_service.sync_full_data(request).await;
    Json(response)
}
