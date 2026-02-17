//! Canary release configuration models

use serde::{Deserialize, Serialize};

/// 金丝雀配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanaryConfig {
    pub service_id: String,
    pub ip_whitelist: Vec<String>,
    pub enabled: bool,
}

/// 设置金丝雀配置请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetCanaryConfigRequest {
    pub service_id: String,
    pub ip_whitelist: Vec<String>,
}

/// 设置金丝雀配置响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetCanaryConfigResponse {
    pub success: bool,
    pub message: Option<String>,
}

/// 获取金丝雀配置请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetCanaryConfigRequest {
    pub service_id: String,
}

/// 获取金丝雀配置响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetCanaryConfigResponse {
    pub success: bool,
    pub config: Option<CanaryConfig>,
}

/// 启用/禁用金丝雀配置请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnableCanaryRequest {
    pub service_id: String,
    pub enabled: bool,
}

/// 启用/禁用金丝雀配置响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnableCanaryResponse {
    pub success: bool,
    pub message: Option<String>,
}
