//! Zone management HTTP API

use crate::state::AppState;
use artemis_core::model::OperateZoneRequest;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: None,
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            message: Some(message),
        }
    }
}

/// POST /api/management/zone/pull-out - 拉出整个 Zone
pub async fn pull_out_zone(
    State(state): State<AppState>,
    Json(req): Json<OperateZoneRequest>,
) -> impl IntoResponse {
    match state
        .zone_manager
        .pull_out_zone(&req.zone_id, &req.region_id, req.operator_id.clone())
    {
        Ok(_) => (
            StatusCode::OK,
            Json(ApiResponse::success("Zone pulled out successfully".to_string())),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<String>::error(e.to_string())),
        ),
    }
}

/// POST /api/management/zone/pull-in - 拉入整个 Zone
pub async fn pull_in_zone(
    State(state): State<AppState>,
    Json(req): Json<OperateZoneRequest>,
) -> impl IntoResponse {
    match state
        .zone_manager
        .pull_in_zone(&req.zone_id, &req.region_id, req.operator_id.clone())
    {
        Ok(_) => (
            StatusCode::OK,
            Json(ApiResponse::success("Zone pulled in successfully".to_string())),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<String>::error(e.to_string())),
        ),
    }
}

/// GET /api/management/zone/status/:zone_id/:region_id - 查询 Zone 状态
pub async fn get_zone_status(
    State(state): State<AppState>,
    Path((zone_id, region_id)): Path<(String, String)>,
) -> impl IntoResponse {
    let is_down = state.zone_manager.is_zone_down(&zone_id, &region_id);
    let status = state.zone_manager.get_zone_status(&zone_id, &region_id);

    let response = serde_json::json!({
        "zone_id": zone_id,
        "region_id": region_id,
        "is_down": is_down,
        "operation": status.as_ref().map(|s| s.operation.to_string()),
        "operator_id": status.as_ref().map(|s| s.operator_id.clone()),
    });

    (StatusCode::OK, Json(ApiResponse::success(response)))
}

#[derive(Debug, Deserialize)]
pub struct ListZoneOpsQuery {
    pub region_id: Option<String>,
}

/// GET /api/management/zone/operations - 列出所有 Zone 操作
pub async fn list_zone_operations(
    State(state): State<AppState>,
    Query(query): Query<ListZoneOpsQuery>,
) -> impl IntoResponse {
    let operations = state
        .zone_manager
        .list_operations(query.region_id.as_deref());

    (StatusCode::OK, Json(ApiResponse::success(operations)))
}

/// DELETE /api/management/zone/:zone_id/:region_id - 移除 Zone 操作记录
pub async fn delete_zone_operation(
    State(state): State<AppState>,
    Path((zone_id, region_id)): Path<(String, String)>,
) -> impl IntoResponse {
    // Pull in is equivalent to deleting pull-out record
    match state
        .zone_manager
        .pull_in_zone(&zone_id, &region_id, "system".to_string())
    {
        Ok(_) => (
            StatusCode::OK,
            Json(ApiResponse::success("Zone operation removed".to_string())),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<String>::error(e.to_string())),
        ),
    }
}
