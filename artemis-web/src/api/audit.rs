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

// ===== Phase 24: 审计日志细分 API =====

#[derive(Debug, Deserialize)]
pub struct QueryGroupLogsParams {
    pub group_id: Option<String>,
    pub operator_id: Option<String>,
    pub limit: Option<usize>,
}

/// POST /api/management/log/group-logs.json - 查询分组操作日志
pub async fn query_group_logs(
    State(state): State<AppState>,
    Query(params): Query<QueryGroupLogsParams>,
) -> impl IntoResponse {
    let logs = state.audit_manager.query_group_logs(
        params.group_id.as_deref(),
        params.operator_id.as_deref(),
        params.limit,
    );

    (StatusCode::OK, Json(ApiResponse::success(logs)))
}

#[derive(Debug, Deserialize)]
pub struct QueryRouteRuleLogsParams {
    pub rule_id: Option<String>,
    pub operator_id: Option<String>,
    pub limit: Option<usize>,
}

/// POST /api/management/log/route-rule-logs.json - 查询路由规则操作日志
pub async fn query_route_rule_logs(
    State(state): State<AppState>,
    Query(params): Query<QueryRouteRuleLogsParams>,
) -> impl IntoResponse {
    let logs = state.audit_manager.query_route_rule_logs(
        params.rule_id.as_deref(),
        params.operator_id.as_deref(),
        params.limit,
    );

    (StatusCode::OK, Json(ApiResponse::success(logs)))
}

#[derive(Debug, Deserialize)]
pub struct QueryRouteRuleGroupLogsParams {
    pub rule_id: Option<String>,
    pub group_id: Option<String>,
    pub operator_id: Option<String>,
    pub limit: Option<usize>,
}

/// POST /api/management/log/route-rule-group-logs.json - 查询路由规则分组操作日志
pub async fn query_route_rule_group_logs(
    State(state): State<AppState>,
    Query(params): Query<QueryRouteRuleGroupLogsParams>,
) -> impl IntoResponse {
    let logs = state.audit_manager.query_route_rule_group_logs(
        params.rule_id.as_deref(),
        params.group_id.as_deref(),
        params.operator_id.as_deref(),
        params.limit,
    );

    (StatusCode::OK, Json(ApiResponse::success(logs)))
}

#[derive(Debug, Deserialize)]
pub struct QueryZoneLogsParams {
    pub zone_id: Option<String>,
    pub region_id: Option<String>,
    pub operator_id: Option<String>,
    pub limit: Option<usize>,
}

/// POST /api/management/log/zone-operation-logs.json - 查询 Zone 操作日志
pub async fn query_zone_operation_logs(
    State(state): State<AppState>,
    Query(params): Query<QueryZoneLogsParams>,
) -> impl IntoResponse {
    let logs = state.audit_manager.query_zone_logs(
        params.zone_id.as_deref(),
        params.region_id.as_deref(),
        params.operator_id.as_deref(),
        params.limit,
    );

    (StatusCode::OK, Json(ApiResponse::success(logs)))
}

#[derive(Debug, Deserialize)]
pub struct QueryGroupInstanceLogsParams {
    pub group_id: Option<String>,
    pub instance_id: Option<String>,
    pub operator_id: Option<String>,
    pub limit: Option<usize>,
}

/// POST /api/management/log/group-instance-logs.json - 查询分组实例绑定日志
pub async fn query_group_instance_logs(
    State(state): State<AppState>,
    Query(params): Query<QueryGroupInstanceLogsParams>,
) -> impl IntoResponse {
    let logs = state.audit_manager.query_group_instance_logs(
        params.group_id.as_deref(),
        params.instance_id.as_deref(),
        params.operator_id.as_deref(),
        params.limit,
    );

    (StatusCode::OK, Json(ApiResponse::success(logs)))
}

#[derive(Debug, Deserialize)]
pub struct QueryServiceInstanceLogsParams {
    pub service_id: Option<String>,
    pub region_id: Option<String>,
    pub operator_id: Option<String>,
    pub limit: Option<usize>,
}

/// POST /api/management/log/service-instance-logs.json - 查询服务实例日志
pub async fn query_service_instance_logs(
    State(state): State<AppState>,
    Query(params): Query<QueryServiceInstanceLogsParams>,
) -> impl IntoResponse {
    let logs = state.audit_manager.query_service_instance_logs(
        params.service_id.as_deref(),
        params.region_id.as_deref(),
        params.operator_id.as_deref(),
        params.limit,
    );

    (StatusCode::OK, Json(ApiResponse::success(logs)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_response_success() {
        let response: ApiResponse<Vec<String>> = ApiResponse::success(vec!["test".to_string()]);
        assert!(response.success);
        assert!(response.data.is_some());
        assert_eq!(response.data.unwrap().len(), 1);
        assert!(response.message.is_none());
    }

    #[test]
    fn test_query_logs_params_deserialize() {
        // 测试参数结构体可以正常反序列化
        let params = QueryLogsParams {
            operation_type: Some("group".to_string()),
            operator_id: Some("admin".to_string()),
            limit: Some(10),
        };
        assert_eq!(params.operation_type, Some("group".to_string()));
        assert_eq!(params.operator_id, Some("admin".to_string()));
        assert_eq!(params.limit, Some(10));
    }

    #[test]
    fn test_query_instance_logs_params() {
        let params = QueryInstanceLogsParams {
            service_id: Some("service1".to_string()),
            operator_id: None,
            limit: Some(5),
        };
        assert!(params.service_id.is_some());
        assert!(params.operator_id.is_none());
    }

    #[test]
    fn test_query_server_logs_params() {
        let params = QueryServerLogsParams {
            server_id: Some("server1".to_string()),
            operator_id: Some("admin".to_string()),
            limit: Some(10),
        };
        assert_eq!(params.server_id.as_deref(), Some("server1"));
    }

    #[test]
    fn test_query_group_logs_params() {
        let params = QueryGroupLogsParams {
            group_id: Some("g1".to_string()),
            operator_id: None,
            limit: None,
        };
        assert!(params.group_id.is_some());
        assert!(params.limit.is_none());
    }

    #[test]
    fn test_query_route_rule_logs_params() {
        let params = QueryRouteRuleLogsParams {
            rule_id: Some("r1".to_string()),
            operator_id: Some("user1".to_string()),
            limit: Some(20),
        };
        assert_eq!(params.rule_id, Some("r1".to_string()));
        assert_eq!(params.limit, Some(20));
    }

    #[test]
    fn test_query_route_rule_group_logs_params() {
        let params = QueryRouteRuleGroupLogsParams {
            rule_id: Some("r1".to_string()),
            group_id: Some("g1".to_string()),
            operator_id: None,
            limit: None,
        };
        assert!(params.rule_id.is_some());
        assert!(params.group_id.is_some());
    }

    #[test]
    fn test_query_zone_logs_params() {
        let params = QueryZoneLogsParams {
            zone_id: Some("zone1".to_string()),
            region_id: Some("us-east".to_string()),
            operator_id: Some("admin".to_string()),
            limit: Some(15),
        };
        assert_eq!(params.zone_id, Some("zone1".to_string()));
        assert_eq!(params.region_id, Some("us-east".to_string()));
    }

    #[test]
    fn test_query_group_instance_logs_params() {
        let params = QueryGroupInstanceLogsParams {
            group_id: Some("g1".to_string()),
            instance_id: Some("inst1".to_string()),
            operator_id: None,
            limit: Some(10),
        };
        assert!(params.group_id.is_some());
        assert!(params.instance_id.is_some());
    }

    #[test]
    fn test_query_service_instance_logs_params() {
        let params = QueryServiceInstanceLogsParams {
            service_id: Some("service1".to_string()),
            region_id: Some("us-east".to_string()),
            operator_id: Some("admin".to_string()),
            limit: Some(25),
        };
        assert_eq!(params.service_id, Some("service1".to_string()));
        assert_eq!(params.limit, Some(25));
    }

    #[test]
    fn test_query_logs_params_all_none() {
        let params = QueryLogsParams {
            operation_type: None,
            operator_id: None,
            limit: None,
        };
        assert!(params.operation_type.is_none());
        assert!(params.operator_id.is_none());
        assert!(params.limit.is_none());
    }

    #[test]
    fn test_query_instance_logs_params_partial() {
        let params = QueryInstanceLogsParams {
            service_id: Some("service2".to_string()),
            operator_id: Some("user2".to_string()),
            limit: None,
        };
        assert!(params.service_id.is_some());
        assert!(params.limit.is_none());
    }

    #[test]
    fn test_query_server_logs_params_no_limit() {
        let params = QueryServerLogsParams {
            server_id: None,
            operator_id: Some("system".to_string()),
            limit: None,
        };
        assert!(params.server_id.is_none());
        assert_eq!(params.operator_id, Some("system".to_string()));
    }

    #[test]
    fn test_query_route_rule_group_logs_params_full() {
        let params = QueryRouteRuleGroupLogsParams {
            rule_id: Some("rule-123".to_string()),
            group_id: Some("group-456".to_string()),
            operator_id: Some("admin".to_string()),
            limit: Some(100),
        };
        assert_eq!(params.rule_id, Some("rule-123".to_string()));
        assert_eq!(params.group_id, Some("group-456".to_string()));
        assert_eq!(params.limit, Some(100));
    }

    #[test]
    fn test_query_zone_logs_params_minimal() {
        let params = QueryZoneLogsParams {
            zone_id: None,
            region_id: None,
            operator_id: None,
            limit: Some(50),
        };
        assert!(params.zone_id.is_none());
        assert!(params.region_id.is_none());
        assert_eq!(params.limit, Some(50));
    }

    #[test]
    fn test_query_group_instance_logs_params_no_operator() {
        let params = QueryGroupInstanceLogsParams {
            group_id: Some("group-789".to_string()),
            instance_id: Some("instance-abc".to_string()),
            operator_id: None,
            limit: Some(30),
        };
        assert!(params.group_id.is_some());
        assert!(params.operator_id.is_none());
        assert_eq!(params.limit, Some(30));
    }

    #[test]
    fn test_api_response_with_vec() {
        let data = vec!["log1".to_string(), "log2".to_string(), "log3".to_string()];
        let response: ApiResponse<Vec<String>> = ApiResponse::success(data.clone());
        assert!(response.success);
        assert_eq!(response.data.unwrap().len(), 3);
    }

    #[test]
    fn test_query_logs_params_with_limit() {
        let params = QueryLogsParams {
            operation_type: Some("zone".to_string()),
            operator_id: None,
            limit: Some(1000),
        };
        assert_eq!(params.operation_type, Some("zone".to_string()));
        assert_eq!(params.limit, Some(1000));
    }

    #[test]
    fn test_query_service_instance_logs_params_no_region() {
        let params = QueryServiceInstanceLogsParams {
            service_id: Some("my-service".to_string()),
            region_id: None,
            operator_id: None,
            limit: None,
        };
        assert_eq!(params.service_id, Some("my-service".to_string()));
        assert!(params.region_id.is_none());
    }
}
