//! Canary release HTTP API

use crate::state::AppState;
use artemis_core::model::{CanaryConfig, EnableCanaryRequest, SetCanaryConfigRequest};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Serialize;

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

/// POST /api/management/canary/config - 设置金丝雀配置
pub async fn set_canary_config(
    State(state): State<AppState>,
    Json(req): Json<SetCanaryConfigRequest>,
) -> impl IntoResponse {
    let config = CanaryConfig {
        service_id: req.service_id.clone(),
        ip_whitelist: req.ip_whitelist.clone(),
        enabled: true,
    };

    match state.canary_manager.set_config(config) {
        Ok(_) => (
            StatusCode::OK,
            Json(ApiResponse::success("Canary config set successfully".to_string())),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<String>::error(e.to_string())),
        ),
    }
}

/// GET /api/management/canary/config/:service_id - 获取金丝雀配置
pub async fn get_canary_config(
    State(state): State<AppState>,
    Path(service_id): Path<String>,
) -> impl IntoResponse {
    match state.canary_manager.get_config(&service_id) {
        Some(config) => (StatusCode::OK, Json(ApiResponse::success(config))),
        None => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<CanaryConfig>::error(
                "Canary config not found".to_string(),
            )),
        ),
    }
}

/// POST /api/management/canary/enable - 启用/禁用金丝雀配置
pub async fn enable_canary(
    State(state): State<AppState>,
    Json(req): Json<EnableCanaryRequest>,
) -> impl IntoResponse {
    match state.canary_manager.set_enabled(&req.service_id, req.enabled) {
        Ok(_) => (
            StatusCode::OK,
            Json(ApiResponse::success(format!(
                "Canary {} for service {}",
                if req.enabled { "enabled" } else { "disabled" },
                req.service_id
            ))),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<String>::error(e.to_string())),
        ),
    }
}

/// DELETE /api/management/canary/config/:service_id - 删除金丝雀配置
pub async fn delete_canary_config(
    State(state): State<AppState>,
    Path(service_id): Path<String>,
) -> impl IntoResponse {
    match state.canary_manager.remove_config(&service_id) {
        Ok(_) => (
            StatusCode::OK,
            Json(ApiResponse::success("Canary config removed".to_string())),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<String>::error(e.to_string())),
        ),
    }
}

/// GET /api/management/canary/configs - 列出所有金丝雀配置
pub async fn list_canary_configs(State(state): State<AppState>) -> impl IntoResponse {
    let configs = state.canary_manager.list_configs();
    (StatusCode::OK, Json(ApiResponse::success(configs)))
}
