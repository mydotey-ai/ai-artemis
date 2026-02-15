use super::instance::{Instance, InstanceKey};
use super::service::Service;
use super::request::ResponseStatus;
use serde::{Deserialize, Serialize};

// ===== 复制-注册 =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicateRegisterRequest {
    pub instances: Vec<Instance>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReplicateRegisterResponse {
    pub response_status: ResponseStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failed_instances: Option<Vec<Instance>>,
}

// ===== 复制-心跳 =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicateHeartbeatRequest {
    pub instance_keys: Vec<InstanceKey>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReplicateHeartbeatResponse {
    pub response_status: ResponseStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failed_instance_keys: Option<Vec<InstanceKey>>,
}

// ===== 复制-注销 =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicateUnregisterRequest {
    pub instance_keys: Vec<InstanceKey>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReplicateUnregisterResponse {
    pub response_status: ResponseStatus,
}

// ===== 获取所有服务(用于启动同步) =====

#[derive(Debug, Serialize, Deserialize)]
pub struct GetAllServicesResponse {
    pub response_status: ResponseStatus,
    pub services: Vec<Service>,
}

// ===== 批量复制 API (Phase 23) =====

/// 批量注册请求 - 用于节点间批量数据同步
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchRegisterRequest {
    pub instances: Vec<Instance>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BatchRegisterResponse {
    pub response_status: ResponseStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failed_instances: Option<Vec<Instance>>,
}

/// 批量心跳请求 - 优化网络请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchHeartbeatRequest {
    pub instance_keys: Vec<InstanceKey>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BatchHeartbeatResponse {
    pub response_status: ResponseStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failed_instance_keys: Option<Vec<InstanceKey>>,
}

/// 批量注销请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchUnregisterRequest {
    pub instance_keys: Vec<InstanceKey>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BatchUnregisterResponse {
    pub response_status: ResponseStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failed_instance_keys: Option<Vec<InstanceKey>>,
}

/// 增量同步请求 - 获取指定时间戳之后的变更
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServicesDeltaRequest {
    pub region_id: String,
    pub zone_id: String,
    pub since_timestamp: i64,  // Unix timestamp in milliseconds
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServicesDeltaResponse {
    pub response_status: ResponseStatus,
    pub services: Vec<Service>,
    pub current_timestamp: i64,
}

/// 全量同步请求 - 新节点加入时的完整数据同步
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncFullDataRequest {
    pub region_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zone_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncFullDataResponse {
    pub response_status: ResponseStatus,
    pub services: Vec<Service>,
    pub sync_timestamp: i64,
}
