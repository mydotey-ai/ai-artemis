use artemis_core::model::{
    GetClusterNodeStatusRequest, GetClusterNodeStatusResponse, GetClusterStatusRequest,
    GetClusterStatusResponse, GetConfigStatusRequest, GetConfigStatusResponse,
    GetDeploymentStatusRequest, GetDeploymentStatusResponse, GetLeasesStatusRequest,
    GetLeasesStatusResponse, LeaseStatus, ResponseStatus, ServiceNode, ServiceNodeStatus,
};
use artemis_core::model::node_status;
use std::collections::HashMap;
use std::sync::Arc;

use crate::cluster::ClusterManager;
use crate::lease::LeaseManager;

/// StatusService 提供系统状态查询功能
pub struct StatusService {
    cluster_manager: Option<Arc<ClusterManager>>,
    lease_manager: Arc<LeaseManager>,
    node_id: String,
    region_id: String,
    zone_id: String,
    server_url: String,
    app_id: String,
}

impl StatusService {
    pub fn new(
        cluster_manager: Option<Arc<ClusterManager>>,
        lease_manager: Arc<LeaseManager>,
        node_id: String,
        region_id: String,
        zone_id: String,
        server_url: String,
        app_id: String,
    ) -> Self {
        Self {
            cluster_manager,
            lease_manager,
            node_id,
            region_id,
            zone_id,
            server_url,
            app_id,
        }
    }

    // ==================== Node Status ====================

    pub async fn get_cluster_node_status(
        &self,
        _request: GetClusterNodeStatusRequest,
    ) -> GetClusterNodeStatusResponse {
        let node = ServiceNode {
            node_id: self.node_id.clone(),
            url: self.server_url.clone(),
            region_id: self.region_id.clone(),
            zone_id: self.zone_id.clone(),
        };

        let node_status = ServiceNodeStatus {
            node,
            status: node_status::UP.to_string(),
            can_service_discovery: true,
            can_service_registry: true,
            allow_registry_from_other_zone: true,
            allow_discovery_from_other_zone: true,
        };

        GetClusterNodeStatusResponse {
            node_status: Some(node_status),
            response_status: ResponseStatus::success(),
        }
    }

    // ==================== Cluster Status ====================

    pub async fn get_cluster_status(
        &self,
        _request: GetClusterStatusRequest,
    ) -> GetClusterStatusResponse {
        let mut nodes_status = Vec::new();

        // 添加当前节点
        let current_node = ServiceNode {
            node_id: self.node_id.clone(),
            url: self.server_url.clone(),
            region_id: self.region_id.clone(),
            zone_id: self.zone_id.clone(),
        };

        nodes_status.push(ServiceNodeStatus {
            node: current_node,
            status: node_status::UP.to_string(),
            can_service_discovery: true,
            can_service_registry: true,
            allow_registry_from_other_zone: true,
            allow_discovery_from_other_zone: true,
        });

        // 添加集群节点
        if let Some(ref cluster_mgr) = self.cluster_manager {
            let cluster_nodes = cluster_mgr.get_healthy_nodes();
            for cluster_node in cluster_nodes.iter() {
                let node = ServiceNode {
                    node_id: cluster_node.node_id.clone(),
                    url: cluster_node.base_url(),
                    region_id: self.region_id.clone(), // 集群节点使用相同的 region
                    zone_id: self.zone_id.clone(),     // 集群节点使用相同的 zone
                };

                // get_healthy_nodes() 已经过滤了健康节点,所以状态都是 UP
                nodes_status.push(ServiceNodeStatus {
                    node,
                    status: node_status::UP.to_string(),
                    can_service_discovery: true,
                    can_service_registry: true,
                    allow_registry_from_other_zone: true,
                    allow_discovery_from_other_zone: true,
                });
            }
        }

        let node_count = nodes_status.len();

        GetClusterStatusResponse {
            nodes_status,
            node_count,
            response_status: ResponseStatus::success(),
        }
    }

    // ==================== Leases Status ====================

    pub async fn get_leases_status(
        &self,
        request: GetLeasesStatusRequest,
    ) -> GetLeasesStatusResponse {
        let all_leases = self.lease_manager.get_all_leases();

        // 过滤服务 (如果提供了 serviceIds)
        let mut leases_status: HashMap<String, Vec<LeaseStatus>> = HashMap::new();

        for (key, lease) in all_leases.iter() {
            // 如果有 service_ids 过滤条件,检查是否匹配
            if let Some(ref service_ids) = request.service_ids {
                if !service_ids.contains(&key.service_id) {
                    continue;
                }
            }

            // 使用 Instant 计算时间差(秒)
            let now = std::time::Instant::now();
            let creation = lease.creation_time();
            let renewal = lease.renewal_time();

            let creation_elapsed = if let Some(elapsed) = now.checked_duration_since(creation) {
                elapsed.as_secs() as i64
            } else {
                0
            };

            let renewal_elapsed = if let Some(elapsed) = now.checked_duration_since(renewal) {
                elapsed.as_secs() as i64
            } else {
                0
            };

            let ttl_secs = lease.ttl_secs();
            let eviction_in = ttl_secs - renewal_elapsed;

            let lease_status = LeaseStatus {
                instance: key.instance_id.clone(),
                creation_time: format!("{} seconds ago", creation_elapsed),
                renewal_time: format!("{} seconds ago", renewal_elapsed),
                evition_time: if eviction_in > 0 {
                    format!("in {} seconds", eviction_in)
                } else {
                    "expired".to_string()
                },
                ttl: ttl_secs,
            };

            leases_status
                .entry(key.service_id.clone())
                .or_insert_with(Vec::new)
                .push(lease_status);
        }

        let lease_count = leases_status.values().map(|v| v.len()).sum();

        GetLeasesStatusResponse {
            lease_update_max_count: 0, // TODO: 实现统计逻辑
            lease_update_max_count_last_update_time: 0,
            lease_update_count_last_time_window: 0,
            is_safe: true, // 默认安全
            is_safe_check_enabled: false,
            lease_count,
            leases_status,
            response_status: ResponseStatus::success(),
        }
    }

    pub async fn get_legacy_leases_status(
        &self,
        request: GetLeasesStatusRequest,
    ) -> GetLeasesStatusResponse {
        // Legacy API 与 get_leases_status 相同实现
        self.get_leases_status(request).await
    }

    // ==================== Config Status ====================

    pub async fn get_config_status(
        &self,
        _request: GetConfigStatusRequest,
    ) -> GetConfigStatusResponse {
        // 返回基本配置信息
        let mut sources = HashMap::new();
        let mut properties = HashMap::new();

        sources.insert("default".to_string(), 1);

        properties.insert("node_id".to_string(), self.node_id.clone());
        properties.insert("region_id".to_string(), self.region_id.clone());
        properties.insert("zone_id".to_string(), self.zone_id.clone());
        properties.insert("app_id".to_string(), self.app_id.clone());

        GetConfigStatusResponse {
            sources,
            properties,
            response_status: ResponseStatus::success(),
        }
    }

    // ==================== Deployment Status ====================

    pub async fn get_deployment_status(
        &self,
        _request: GetDeploymentStatusRequest,
    ) -> GetDeploymentStatusResponse {
        let mut sources = HashMap::new();
        let mut properties = HashMap::new();

        sources.insert("default".to_string(), 1);

        properties.insert("node_id".to_string(), self.node_id.clone());
        properties.insert("region_id".to_string(), self.region_id.clone());
        properties.insert("zone_id".to_string(), self.zone_id.clone());
        properties.insert("server_url".to_string(), self.server_url.clone());

        // 从 server_url 中提取 IP 和 端口
        let (ip, port, protocol, path) = Self::parse_url(&self.server_url);

        GetDeploymentStatusResponse {
            region_id: self.region_id.clone(),
            zone_id: self.zone_id.clone(),
            app_id: self.app_id.clone(),
            machine_name: hostname::get()
                .ok()
                .and_then(|h| h.into_string().ok())
                .unwrap_or_else(|| "unknown".to_string()),
            ip,
            port,
            protocol,
            path,
            sources,
            properties,
            response_status: ResponseStatus::success(),
        }
    }

    // ==================== Helper Functions ====================

    #[allow(dead_code)]
    fn format_timestamp(ts: i64) -> String {
        let datetime = chrono::DateTime::from_timestamp(ts, 0);
        datetime
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_else(|| "unknown".to_string())
    }

    fn parse_url(url: &str) -> (String, u16, String, String) {
        // 简单的 URL 解析,支持 http://ip:port/path 格式
        let protocol = if url.starts_with("https://") {
            "https"
        } else {
            "http"
        }
        .to_string();

        let without_protocol = url.trim_start_matches("http://").trim_start_matches("https://");

        let parts: Vec<&str> = without_protocol.splitn(2, '/').collect();
        let host_port = parts[0];
        let path = if parts.len() > 1 {
            format!("/{}", parts[1])
        } else {
            "/".to_string()
        };

        let (ip, port) = if let Some(pos) = host_port.rfind(':') {
            let ip = host_port[..pos].to_string();
            let port = host_port[pos + 1..]
                .parse::<u16>()
                .unwrap_or(8080);
            (ip, port)
        } else {
            (host_port.to_string(), 8080)
        };

        (ip, port, protocol, path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_url() {
        let (ip, port, protocol, path) = StatusService::parse_url("http://10.0.0.1:8080/api");
        assert_eq!(ip, "10.0.0.1");
        assert_eq!(port, 8080);
        assert_eq!(protocol, "http");
        assert_eq!(path, "/api");

        let (ip, port, protocol, path) =
            StatusService::parse_url("https://example.com:9090");
        assert_eq!(ip, "example.com");
        assert_eq!(port, 9090);
        assert_eq!(protocol, "https");
        assert_eq!(path, "/");
    }
}
