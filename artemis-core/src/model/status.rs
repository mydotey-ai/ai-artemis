use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::ResponseStatus;

// ==================== Node Status ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetClusterNodeStatusRequest {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetClusterNodeStatusResponse {
    #[serde(rename = "nodeStatus")]
    pub node_status: Option<ServiceNodeStatus>,

    #[serde(rename = "responseStatus")]
    pub response_status: ResponseStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceNodeStatus {
    pub node: ServiceNode,
    pub status: String,  // "starting" | "up" | "down" | "unknown"

    #[serde(rename = "canServiceDiscovery")]
    pub can_service_discovery: bool,

    #[serde(rename = "canServiceRegistry")]
    pub can_service_registry: bool,

    #[serde(rename = "allowRegistryFromOtherZone")]
    pub allow_registry_from_other_zone: bool,

    #[serde(rename = "allowDiscoveryFromOtherZone")]
    pub allow_discovery_from_other_zone: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceNode {
    #[serde(rename = "nodeId")]
    pub node_id: String,

    pub url: String,

    #[serde(rename = "regionId")]
    pub region_id: String,

    #[serde(rename = "zoneId")]
    pub zone_id: String,
}

// ==================== Cluster Status ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetClusterStatusRequest {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetClusterStatusResponse {
    #[serde(rename = "nodesStatus")]
    pub nodes_status: Vec<ServiceNodeStatus>,

    #[serde(rename = "nodeCount")]
    pub node_count: usize,

    #[serde(rename = "responseStatus")]
    pub response_status: ResponseStatus,
}

// ==================== Leases Status ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetLeasesStatusRequest {
    #[serde(rename = "serviceIds")]
    pub service_ids: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetLeasesStatusResponse {
    #[serde(rename = "leaseUpdateMaxCount")]
    pub lease_update_max_count: u64,

    #[serde(rename = "leaseUpdateMaxCountLastUpdateTime")]
    pub lease_update_max_count_last_update_time: i64,

    #[serde(rename = "leaseUpdateCountLastTimeWindow")]
    pub lease_update_count_last_time_window: u64,

    #[serde(rename = "isSafe")]
    pub is_safe: bool,

    #[serde(rename = "isSafeCheckEnabled")]
    pub is_safe_check_enabled: bool,

    #[serde(rename = "leaseCount")]
    pub lease_count: usize,

    // Map<Service, List<LeaseStatus>>
    // For simplicity, use ServiceId as key
    #[serde(rename = "leasesStatus")]
    pub leases_status: HashMap<String, Vec<LeaseStatus>>,

    #[serde(rename = "responseStatus")]
    pub response_status: ResponseStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaseStatus {
    pub instance: String,  // instance_id

    #[serde(rename = "creationTime")]
    pub creation_time: String,

    #[serde(rename = "renewalTime")]
    pub renewal_time: String,

    #[serde(rename = "evitionTime")]
    pub evition_time: String,

    pub ttl: i64,
}

// ==================== Config Status ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetConfigStatusRequest {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetConfigStatusResponse {
    pub sources: HashMap<String, i32>,
    pub properties: HashMap<String, String>,

    #[serde(rename = "responseStatus")]
    pub response_status: ResponseStatus,
}

// ==================== Deployment Status ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetDeploymentStatusRequest {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetDeploymentStatusResponse {
    #[serde(rename = "regionId")]
    pub region_id: String,

    #[serde(rename = "zoneId")]
    pub zone_id: String,

    #[serde(rename = "appId")]
    pub app_id: String,

    #[serde(rename = "machineName")]
    pub machine_name: String,

    pub ip: String,
    pub port: u16,
    pub protocol: String,
    pub path: String,

    pub sources: HashMap<String, i32>,
    pub properties: HashMap<String, String>,

    #[serde(rename = "responseStatus")]
    pub response_status: ResponseStatus,
}


// ==================== Node Status Constants ====================

pub mod node_status {
    pub const STARTING: &str = "starting";
    pub const UP: &str = "up";
    pub const DOWN: &str = "down";
    pub const UNKNOWN: &str = "unknown";
}
