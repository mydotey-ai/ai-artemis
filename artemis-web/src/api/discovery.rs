use crate::state::AppState;
use artemis_core::{model::*, traits::DiscoveryService};
use artemis_server::discovery::LoadBalanceStrategy;
use axum::{http::StatusCode, response::IntoResponse, Json, extract::{State, Query}};
use serde::{Deserialize, Serialize};

// ===== GET Service API (POST 版本) =====

pub async fn get_service(
    State(state): State<AppState>,
    Json(request): Json<GetServiceRequest>,
) -> Json<GetServiceResponse> {
    Json(state.discovery_service.get_service(request).await)
}

// ===== GET Service API (GET 版本 - Phase 22 新增) =====

#[derive(Debug, Deserialize)]
pub struct GetServiceQuery {
    #[serde(rename = "serviceId")]
    pub service_id: String,
    #[serde(rename = "regionId")]
    pub region_id: Option<String>,
    #[serde(rename = "zoneId")]
    pub zone_id: Option<String>,
}

pub async fn get_service_by_query(
    State(state): State<AppState>,
    Query(query): Query<GetServiceQuery>,
) -> Json<GetServiceResponse> {
    let request = GetServiceRequest {
        discovery_config: DiscoveryConfig {
            service_id: query.service_id,
            region_id: query.region_id.unwrap_or_else(|| "default".to_string()),
            zone_id: query.zone_id.unwrap_or_else(|| "default".to_string()),
            discovery_data: None,
        },
    };
    Json(state.discovery_service.get_service(request).await)
}

// ===== GET Services API (POST 版本) =====

pub async fn get_services(
    State(state): State<AppState>,
    Json(request): Json<GetServicesRequest>,
) -> Json<GetServicesResponse> {
    Json(state.discovery_service.get_services(request).await)
}

// ===== GET Services API (GET 版本 - Phase 22 新增) =====

#[derive(Debug, Deserialize)]
pub struct GetServicesQuery {
    #[serde(rename = "regionId")]
    pub region_id: Option<String>,
    #[serde(rename = "zoneId")]
    pub zone_id: Option<String>,
}

pub async fn get_services_by_query(
    State(state): State<AppState>,
    Query(query): Query<GetServicesQuery>,
) -> Json<GetServicesResponse> {
    let request = GetServicesRequest {
        region_id: query.region_id.unwrap_or_else(|| "default".to_string()),
        zone_id: query.zone_id.unwrap_or_else(|| "default".to_string()),
    };
    Json(state.discovery_service.get_services(request).await)
}

// ===== Discovery Lookup API (Phase 20 新增) =====

/// Lookup 请求 (查询单个实例)
#[derive(Debug, Serialize, Deserialize)]
pub struct LookupRequest {
    pub discovery_config: DiscoveryConfig,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strategy: Option<String>, // "random" or "round-robin"
}

/// Lookup 响应
#[derive(Debug, Serialize)]
pub struct LookupResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance: Option<Instance>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// POST /api/discovery/lookup.json - 查询单个实例 (负载均衡选择)
pub async fn lookup_instance(
    State(state): State<AppState>,
    Json(request): Json<LookupRequest>,
) -> impl IntoResponse {
    // 1. 先获取服务的所有实例
    let get_service_request = GetServiceRequest {
        discovery_config: request.discovery_config.clone(),
    };

    let response = state.discovery_service.get_service(get_service_request).await;

    // 2. 检查是否有可用服务和实例
    let service = match response.service {
        Some(s) => s,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(LookupResponse {
                    success: false,
                    instance: None,
                    message: Some(format!(
                        "Service not found: {}",
                        request.discovery_config.service_id
                    )),
                }),
            );
        }
    };

    if service.instances.is_empty() {
        return (
            StatusCode::NOT_FOUND,
            Json(LookupResponse {
                success: false,
                instance: None,
                message: Some(format!(
                    "No instances found for service: {}",
                    request.discovery_config.service_id
                )),
            }),
        );
    }

    // 3. 使用负载均衡策略选择一个实例
    let strategy = match request.strategy.as_deref() {
        Some("round-robin") => LoadBalanceStrategy::RoundRobin,
        _ => LoadBalanceStrategy::Random, // 默认随机
    };

    let selected_instance = state
        .load_balancer
        .select_instance(&service.instances, strategy);

    match selected_instance {
        Some(instance) => (
            StatusCode::OK,
            Json(LookupResponse {
                success: true,
                instance: Some(instance),
                message: None,
            }),
        ),
        None => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(LookupResponse {
                success: false,
                instance: None,
                message: Some("Failed to select instance".to_string()),
            }),
        ),
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use artemis_core::model::{DiscoveryConfig, GetServiceRequest, GetServicesRequest};

    #[test]
    fn test_get_service_request() {
        let config = DiscoveryConfig {
            service_id: "test-service".to_string(),
            region_id: "us-east".to_string(),
            zone_id: "zone1".to_string(),
            discovery_data: None,
        };
        let req = GetServiceRequest {
            discovery_config: config.clone(),
        };
        assert_eq!(req.discovery_config.service_id, "test-service");
        assert_eq!(req.discovery_config.region_id, "us-east");
    }

    #[test]
    fn test_get_services_request() {
        let req = GetServicesRequest {
            region_id: "us-west".to_string(),
            zone_id: "zone2".to_string(),
        };
        assert_eq!(req.region_id, "us-west");
        assert_eq!(req.zone_id, "zone2");
    }

    #[test]
    fn test_discovery_config_with_data() {
        use std::collections::HashMap;
        let mut data = HashMap::new();
        data.insert("key1".to_string(), "value1".to_string());
        
        let config = DiscoveryConfig {
            service_id: "service1".to_string(),
            region_id: "us-east".to_string(),
            zone_id: "zone1".to_string(),
            discovery_data: Some(data),
        };
        assert!(config.discovery_data.is_some());
        assert_eq!(config.discovery_data.unwrap().get("key1"), Some(&"value1".to_string()));
    }

    #[test]
    fn test_discovery_config_without_data() {
        let config = DiscoveryConfig {
            service_id: "service2".to_string(),
            region_id: "eu-west".to_string(),
            zone_id: "zone3".to_string(),
            discovery_data: None,
        };
        assert_eq!(config.service_id, "service2");
        assert!(config.discovery_data.is_none());
    }
}
