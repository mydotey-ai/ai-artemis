use artemis_common::model::ResponseStatus;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ==================== Node Status ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetClusterNodeStatusRequest {}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetClusterNodeStatusResponse {
    pub node_status: Option<ServiceNodeStatus>,
    pub response_status: ResponseStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceNodeStatus {
    pub node: ServiceNode,
    pub status: String, // "starting" | "up" | "down" | "unknown"
    pub can_service_discovery: bool,
    pub can_service_registry: bool,
    pub allow_registry_from_other_zone: bool,
    pub allow_discovery_from_other_zone: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceNode {
    pub node_id: String,
    pub url: String,
    pub region_id: String,
    pub zone_id: String,
}

// ==================== Cluster Status ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetClusterStatusRequest {}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetClusterStatusResponse {
    pub nodes_status: Vec<ServiceNodeStatus>,
    pub node_count: usize,
    pub response_status: ResponseStatus,
}

// ==================== Leases Status ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetLeasesStatusRequest {
    pub service_ids: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetLeasesStatusResponse {
    pub lease_update_max_count: u64,
    pub lease_update_max_count_last_update_time: i64,
    pub lease_update_count_last_time_window: u64,
    pub is_safe: bool,
    pub is_safe_check_enabled: bool,
    pub lease_count: usize,
    // Map<Service, List<LeaseStatus>>
    // For simplicity, use ServiceId as key
    pub leases_status: HashMap<String, Vec<LeaseStatus>>,
    pub response_status: ResponseStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeaseStatus {
    pub instance: String, // instance_id
    pub creation_time: String,
    pub renewal_time: String,
    pub evition_time: String,
    pub ttl: i64,
}

// ==================== Config Status ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetConfigStatusRequest {}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetConfigStatusResponse {
    pub sources: HashMap<String, i32>,
    pub properties: HashMap<String, String>,
    pub response_status: ResponseStatus,
}

// ==================== Deployment Status ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetDeploymentStatusRequest {}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetDeploymentStatusResponse {
    pub region_id: String,
    pub zone_id: String,
    pub app_id: String,
    pub machine_name: String,
    pub ip: String,
    pub port: u16,
    pub protocol: String,
    pub path: String,
    pub sources: HashMap<String, i32>,
    pub properties: HashMap<String, String>,
    pub response_status: ResponseStatus,
}

// ==================== Node Status Constants ====================

pub mod node_status {
    pub const STARTING: &str = "starting";
    pub const UP: &str = "up";
    pub const DOWN: &str = "down";
    pub const UNKNOWN: &str = "unknown";
}
