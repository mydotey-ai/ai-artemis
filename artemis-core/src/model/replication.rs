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
