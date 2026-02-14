//! Zone management data models

use serde::{Deserialize, Serialize};
use std::fmt;

/// Zone 操作类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ZoneOperation {
    /// 拉入整个 Zone
    PullIn,
    /// 拉出整个 Zone
    PullOut,
}

impl fmt::Display for ZoneOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ZoneOperation::PullIn => write!(f, "pullin"),
            ZoneOperation::PullOut => write!(f, "pullout"),
        }
    }
}

/// Zone 操作记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoneOperationRecord {
    pub zone_id: String,
    pub region_id: String,
    pub operation: ZoneOperation,
    pub operator_id: String,
    pub operation_time: i64,
}

/// 操作 Zone 请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperateZoneRequest {
    pub zone_id: String,
    pub region_id: String,
    pub operation: ZoneOperation,
    pub operator_id: String,
}

/// 操作 Zone 响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperateZoneResponse {
    pub success: bool,
    pub message: Option<String>,
}

/// 查询 Zone 状态请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetZoneStatusRequest {
    pub zone_id: String,
    pub region_id: String,
}

/// 查询 Zone 状态响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetZoneStatusResponse {
    pub success: bool,
    pub zone_id: String,
    pub region_id: String,
    pub is_down: bool,
    pub operation: Option<ZoneOperation>,
    pub operator_id: Option<String>,
}

/// 查询所有 Zone 操作请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListZoneOperationsRequest {
    pub region_id: Option<String>,
}

/// 查询所有 Zone 操作响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListZoneOperationsResponse {
    pub success: bool,
    pub operations: Vec<ZoneOperationRecord>,
}
