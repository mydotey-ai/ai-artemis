//! Canary release HTTP API

use crate::state::AppState;
use artemis_core::model::{CanaryConfig, EnableCanaryRequest, SetCanaryConfigRequest};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
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
        Self { success: true, data: Some(data), message: None }
    }

    pub fn error(message: String) -> Self {
        Self { success: false, data: None, message: Some(message) }
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
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::<String>::error(e.to_string())))
        }
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
            Json(ApiResponse::<CanaryConfig>::error("Canary config not found".to_string())),
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
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::<String>::error(e.to_string())))
        }
    }
}

/// DELETE /api/management/canary/config/:service_id - 删除金丝雀配置
pub async fn delete_canary_config(
    State(state): State<AppState>,
    Path(service_id): Path<String>,
) -> impl IntoResponse {
    match state.canary_manager.remove_config(&service_id) {
        Ok(_) => (StatusCode::OK, Json(ApiResponse::success("Canary config removed".to_string()))),
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::<String>::error(e.to_string())))
        }
    }
}

/// GET /api/management/canary/configs - 列出所有金丝雀配置
pub async fn list_canary_configs(State(state): State<AppState>) -> impl IntoResponse {
    let configs = state.canary_manager.list_configs();
    (StatusCode::OK, Json(ApiResponse::success(configs)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_response_success() {
        let response: ApiResponse<String> = ApiResponse::success("test".to_string());
        assert!(response.success);
        assert_eq!(response.data, Some("test".to_string()));
        assert!(response.message.is_none());
    }

    #[test]
    fn test_api_response_error() {
        let response: ApiResponse<String> = ApiResponse::error("error".to_string());
        assert!(!response.success);
        assert!(response.data.is_none());
        assert_eq!(response.message, Some("error".to_string()));
    }

    #[test]
    fn test_set_canary_config_request() {
        use artemis_core::model::SetCanaryConfigRequest;
        let req = SetCanaryConfigRequest {
            service_id: "service1".to_string(),
            ip_whitelist: vec!["192.168.1.1".to_string()],
        };
        assert_eq!(req.service_id, "service1");
        assert_eq!(req.ip_whitelist.len(), 1);
    }

    #[test]
    fn test_enable_canary_request() {
        use artemis_core::model::EnableCanaryRequest;
        let req = EnableCanaryRequest { service_id: "service1".to_string(), enabled: true };
        assert_eq!(req.service_id, "service1");
        assert!(req.enabled);
    }

    #[test]
    fn test_canary_config() {
        use artemis_core::model::CanaryConfig;
        let config = CanaryConfig {
            service_id: "service1".to_string(),
            ip_whitelist: vec!["192.168.1.1".to_string(), "10.0.0.1".to_string()],
            enabled: true,
        };
        assert_eq!(config.service_id, "service1");
        assert_eq!(config.ip_whitelist.len(), 2);
        assert!(config.enabled);
    }

    #[test]
    fn test_canary_config_disabled() {
        use artemis_core::model::CanaryConfig;
        let config = CanaryConfig {
            service_id: "service2".to_string(),
            ip_whitelist: vec![],
            enabled: false,
        };
        assert!(!config.enabled);
        assert_eq!(config.ip_whitelist.len(), 0);
    }

    #[test]
    fn test_enable_canary_request_disabled() {
        use artemis_core::model::EnableCanaryRequest;
        let req = EnableCanaryRequest { service_id: "service2".to_string(), enabled: false };
        assert_eq!(req.service_id, "service2");
        assert!(!req.enabled);
    }

    #[test]
    fn test_set_canary_config_request_empty_whitelist() {
        use artemis_core::model::SetCanaryConfigRequest;
        let req =
            SetCanaryConfigRequest { service_id: "service3".to_string(), ip_whitelist: vec![] };
        assert_eq!(req.service_id, "service3");
        assert!(req.ip_whitelist.is_empty());
    }

    #[test]
    fn test_set_canary_config_request_multiple_ips() {
        use artemis_core::model::SetCanaryConfigRequest;
        let ips =
            vec!["192.168.1.1".to_string(), "192.168.1.2".to_string(), "10.0.0.1".to_string()];
        let req = SetCanaryConfigRequest {
            service_id: "service4".to_string(),
            ip_whitelist: ips.clone(),
        };
        assert_eq!(req.ip_whitelist.len(), 3);
        assert_eq!(req.ip_whitelist[0], "192.168.1.1");
        assert_eq!(req.ip_whitelist[2], "10.0.0.1");
    }

    #[test]
    fn test_api_response_success_with_message() {
        let response: ApiResponse<String> = ApiResponse::success("Success message".to_string());
        assert!(response.success);
        assert_eq!(response.data, Some("Success message".to_string()));
        assert!(response.message.is_none());
    }

    #[test]
    fn test_api_response_error_with_details() {
        let error_msg = "Configuration not found".to_string();
        let response: ApiResponse<CanaryConfig> = ApiResponse::error(error_msg.clone());
        assert!(!response.success);
        assert!(response.data.is_none());
        assert_eq!(response.message, Some(error_msg));
    }
}
