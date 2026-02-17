//! Zone management HTTP API

use crate::state::AppState;
use artemis_management::model::OperateZoneRequest;
use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
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
        Self { success: true, data: Some(data), message: None }
    }

    pub fn error(message: String) -> Self {
        Self { success: false, data: None, message: Some(message) }
    }
}

/// POST /api/management/zone/pull-out - 拉出整个 Zone
pub async fn pull_out_zone(
    State(state): State<AppState>,
    Json(req): Json<OperateZoneRequest>,
) -> impl IntoResponse {
    match state.zone_manager.pull_out_zone(&req.zone_id, &req.region_id, req.operator_id.clone()) {
        Ok(_) => {
            (StatusCode::OK, Json(ApiResponse::success("Zone pulled out successfully".to_string())))
        }
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::<String>::error(e.to_string())))
        }
    }
}

/// POST /api/management/zone/pull-in - 拉入整个 Zone
pub async fn pull_in_zone(
    State(state): State<AppState>,
    Json(req): Json<OperateZoneRequest>,
) -> impl IntoResponse {
    match state.zone_manager.pull_in_zone(&req.zone_id, &req.region_id, req.operator_id.clone()) {
        Ok(_) => {
            (StatusCode::OK, Json(ApiResponse::success("Zone pulled in successfully".to_string())))
        }
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::<String>::error(e.to_string())))
        }
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
    let operations = state.zone_manager.list_operations(query.region_id.as_deref());

    (StatusCode::OK, Json(ApiResponse::success(operations)))
}

/// DELETE /api/management/zone/:zone_id/:region_id - 移除 Zone 操作记录
pub async fn delete_zone_operation(
    State(state): State<AppState>,
    Path((zone_id, region_id)): Path<(String, String)>,
) -> impl IntoResponse {
    // Pull in is equivalent to deleting pull-out record
    match state.zone_manager.pull_in_zone(&zone_id, &region_id, "system".to_string()) {
        Ok(_) => (StatusCode::OK, Json(ApiResponse::success("Zone operation removed".to_string()))),
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::<String>::error(e.to_string())))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use artemis_management::model::OperateZoneRequest;

    #[test]
    fn test_api_response_success() {
        let response: ApiResponse<String> = ApiResponse::success("test".to_string());
        assert!(response.success);
        assert!(response.data.is_some());
        assert!(response.message.is_none());
    }

    #[test]
    fn test_api_response_error() {
        let response: ApiResponse<String> = ApiResponse::error("error".to_string());
        assert!(!response.success);
        assert!(response.data.is_none());
        assert!(response.message.is_some());
    }

    #[test]
    fn test_api_response_success_with_data() {
        let data = vec!["zone1".to_string(), "zone2".to_string()];
        let response: ApiResponse<Vec<String>> = ApiResponse::success(data.clone());
        assert!(response.success);
        assert_eq!(response.data, Some(data));
        assert!(response.message.is_none());
    }

    #[test]
    fn test_api_response_error_with_message() {
        let error_msg = "Zone not found".to_string();
        let response: ApiResponse<()> = ApiResponse::error(error_msg.clone());
        assert!(!response.success);
        assert!(response.data.is_none());
        assert_eq!(response.message, Some(error_msg));
    }

    #[test]
    fn test_operate_zone_request() {
        use artemis_management::model::ZoneOperation;
        let request = OperateZoneRequest {
            zone_id: "zone-1".to_string(),
            region_id: "us-east".to_string(),
            operation: ZoneOperation::PullOut,
            operator_id: "admin".to_string(),
        };
        assert_eq!(request.zone_id, "zone-1");
        assert_eq!(request.region_id, "us-east");
        assert_eq!(request.operator_id, "admin");
    }

    #[test]
    fn test_operate_zone_request_pull_in() {
        use artemis_management::model::ZoneOperation;
        let request = OperateZoneRequest {
            zone_id: "zone-2".to_string(),
            region_id: "eu-west".to_string(),
            operation: ZoneOperation::PullIn,
            operator_id: "system".to_string(),
        };
        assert_eq!(request.zone_id, "zone-2");
        assert_eq!(request.operator_id, "system");
    }

    #[test]
    fn test_list_zone_ops_query() {
        let query = ListZoneOpsQuery { region_id: Some("us-west".to_string()) };
        assert_eq!(query.region_id, Some("us-west".to_string()));
    }

    #[test]
    fn test_list_zone_ops_query_no_region() {
        let query = ListZoneOpsQuery { region_id: None };
        assert!(query.region_id.is_none());
    }
}
