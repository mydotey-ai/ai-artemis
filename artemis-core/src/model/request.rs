use super::instance::{Instance, InstanceKey};
use super::service::Service;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ===== 注册 =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub instances: Vec<Instance>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterResponse {
    pub response_status: ResponseStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failed_instances: Option<Vec<Instance>>,
}

// ===== 心跳 =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatRequest {
    pub instance_keys: Vec<InstanceKey>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatResponse {
    pub response_status: ResponseStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failed_instance_keys: Option<Vec<InstanceKey>>,
}

// ===== 注销 =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnregisterRequest {
    pub instance_keys: Vec<InstanceKey>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnregisterResponse {
    pub response_status: ResponseStatus,
}

// ===== 发现 =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetServiceRequest {
    pub discovery_config: DiscoveryConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryConfig {
    pub service_id: String,
    pub region_id: String,
    pub zone_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discovery_data: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetServiceResponse {
    pub response_status: ResponseStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<Service>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetServicesRequest {
    pub region_id: String,
    pub zone_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetServicesResponse {
    pub response_status: ResponseStatus,
    pub services: Vec<Service>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetServicesDeltaRequest {
    pub region_id: String,
    pub zone_id: String,
    pub since_timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetServicesDeltaResponse {
    pub response_status: ResponseStatus,
    pub services: Vec<Service>,
    pub current_timestamp: i64,
}

// ===== 通用响应状态 =====

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResponseStatus {
    pub error_code: ErrorCode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

impl ResponseStatus {
    pub fn success() -> Self {
        Self { error_code: ErrorCode::Success, error_message: None }
    }

    pub fn error(code: ErrorCode, message: impl Into<String>) -> Self {
        Self { error_code: code, error_message: Some(message.into()) }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ErrorCode {
    Success,
    BadRequest,
    ServiceUnavailable,
    RateLimited,
    InternalError,
}
