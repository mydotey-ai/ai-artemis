//! Audit log HTTP API

use crate::state::AppState;
use axum::{
    extract::{Query, State},
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
}

#[derive(Debug, Deserialize)]
pub struct QueryLogsParams {
    pub operation_type: Option<String>,
    pub operator_id: Option<String>,
    pub limit: Option<usize>,
}

/// GET /api/management/audit/logs - 查询所有操作日志
pub async fn query_logs(
    State(state): State<AppState>,
    Query(params): Query<QueryLogsParams>,
) -> impl IntoResponse {
    let logs = state.audit_manager.query_logs(
        params.operation_type.as_deref(),
        params.operator_id.as_deref(),
        params.limit,
    );

    (StatusCode::OK, Json(ApiResponse::success(logs)))
}

#[derive(Debug, Deserialize)]
pub struct QueryInstanceLogsParams {
    pub service_id: Option<String>,
    pub operator_id: Option<String>,
    pub limit: Option<usize>,
}

/// GET /api/management/audit/instance-logs - 查询实例操作日志
pub async fn query_instance_logs(
    State(state): State<AppState>,
    Query(params): Query<QueryInstanceLogsParams>,
) -> impl IntoResponse {
    let logs = state.audit_manager.query_instance_logs(
        params.service_id.as_deref(),
        params.operator_id.as_deref(),
        params.limit,
    );

    (StatusCode::OK, Json(ApiResponse::success(logs)))
}

#[derive(Debug, Deserialize)]
pub struct QueryServerLogsParams {
    pub server_id: Option<String>,
    pub operator_id: Option<String>,
    pub limit: Option<usize>,
}

/// GET /api/management/audit/server-logs - 查询服务器操作日志
pub async fn query_server_logs(
    State(state): State<AppState>,
    Query(params): Query<QueryServerLogsParams>,
) -> impl IntoResponse {
    let logs = state.audit_manager.query_server_logs(
        params.server_id.as_deref(),
        params.operator_id.as_deref(),
        params.limit,
    );

    (StatusCode::OK, Json(ApiResponse::success(logs)))
}
