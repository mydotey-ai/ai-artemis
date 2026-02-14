use super::instance::InstanceKey;
use super::request::ResponseStatus;
use serde::{Deserialize, Serialize};

/// 实例操作类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum InstanceOperation {
    /// 拉入 (恢复服务)
    PullIn,
    /// 拉出 (下线服务)
    PullOut,
}

/// 实例操作记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceOperationRecord {
    /// 实例键
    pub instance_key: InstanceKey,
    /// 操作类型
    pub operation: InstanceOperation,
    /// 操作是否完成
    pub operation_complete: bool,
    /// 操作人 ID
    pub operator_id: String,
    /// 操作 Token
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
}

/// 操作实例请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperateInstanceRequest {
    /// 实例键
    pub instance_key: InstanceKey,
    /// 操作类型
    pub operation: InstanceOperation,
    /// 操作是否完成 (true=完成拉出, false=开始拉出)
    #[serde(default)]
    pub operation_complete: bool,
    /// 操作人 ID
    pub operator_id: String,
    /// 操作 Token
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
}

/// 操作实例响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperateInstanceResponse {
    /// 响应状态
    pub status: ResponseStatus,
}

/// 查询实例操作请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetInstanceOperationsRequest {
    /// 实例键
    pub instance_key: InstanceKey,
}

/// 查询实例操作响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetInstanceOperationsResponse {
    /// 响应状态
    pub status: ResponseStatus,
    /// 操作列表
    pub operations: Vec<InstanceOperation>,
}

/// 查询实例是否被拉出请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsInstanceDownRequest {
    /// 实例键
    pub instance_key: InstanceKey,
}

/// 查询实例是否被拉出响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsInstanceDownResponse {
    /// 响应状态
    pub status: ResponseStatus,
    /// 是否被拉出
    pub is_down: bool,
}

/// 服务器操作类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ServerOperation {
    /// 拉入整台服务器
    PullIn,
    /// 拉出整台服务器
    PullOut,
}

/// 操作服务器请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperateServerRequest {
    /// 服务器 ID (IP 地址)
    pub server_id: String,
    /// Region ID
    pub region_id: String,
    /// 操作类型
    pub operation: ServerOperation,
    /// 操作是否完成
    #[serde(default)]
    pub operation_complete: bool,
    /// 操作人 ID
    pub operator_id: String,
    /// 操作 Token
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
}

/// 操作服务器响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperateServerResponse {
    /// 响应状态
    pub status: ResponseStatus,
}

/// 查询服务器是否被拉出请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsServerDownRequest {
    /// 服务器 ID
    pub server_id: String,
    /// Region ID
    pub region_id: String,
}

/// 查询服务器是否被拉出响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsServerDownResponse {
    /// 响应状态
    pub status: ResponseStatus,
    /// 是否被拉出
    pub is_down: bool,
}

// ResponseStatus 使用 super::request::ResponseStatus
