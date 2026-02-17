use artemis_core::model::node_status;
use artemis_core::model::{
    GetClusterNodeStatusRequest, GetClusterNodeStatusResponse, GetClusterStatusRequest,
    GetClusterStatusResponse, GetConfigStatusRequest, GetConfigStatusResponse,
    GetDeploymentStatusRequest, GetDeploymentStatusResponse, GetLeasesStatusRequest,
    GetLeasesStatusResponse, LeaseStatus, ResponseStatus, ServiceNode, ServiceNodeStatus,
};
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
        Self { cluster_manager, lease_manager, node_id, region_id, zone_id, server_url, app_id }
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
            if let Some(ref service_ids) = request.service_ids
                && !service_ids.contains(&key.service_id)
            {
                continue;
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

            leases_status.entry(key.service_id.clone()).or_default().push(lease_status);
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

        GetConfigStatusResponse { sources, properties, response_status: ResponseStatus::success() }
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
        let protocol = if url.starts_with("https://") { "https" } else { "http" }.to_string();

        let without_protocol = url.trim_start_matches("http://").trim_start_matches("https://");

        let parts: Vec<&str> = without_protocol.splitn(2, '/').collect();
        let host_port = parts[0];
        let path = if parts.len() > 1 { format!("/{}", parts[1]) } else { "/".to_string() };

        let (ip, port) = if let Some(pos) = host_port.rfind(':') {
            let ip = host_port[..pos].to_string();
            let port = host_port[pos + 1..].parse::<u16>().unwrap_or(8080);
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

    fn create_test_service() -> StatusService {
        let lease_manager = Arc::new(LeaseManager::new(std::time::Duration::from_secs(30)));
        StatusService::new(
            None,
            lease_manager,
            "test-node".to_string(),
            "test-region".to_string(),
            "test-zone".to_string(),
            "http://localhost:8080".to_string(),
            "test-app".to_string(),
        )
    }

    fn create_test_service_with_cluster() -> StatusService {
        let cluster_manager = Arc::new(ClusterManager::default());
        let lease_manager = Arc::new(LeaseManager::new(std::time::Duration::from_secs(30)));
        StatusService::new(
            Some(cluster_manager),
            lease_manager,
            "test-node".to_string(),
            "test-region".to_string(),
            "test-zone".to_string(),
            "http://localhost:8080".to_string(),
            "test-app".to_string(),
        )
    }

    // ========== URL 解析测试 ==========

    #[test]
    fn test_parse_url() {
        let (ip, port, protocol, path) = StatusService::parse_url("http://10.0.0.1:8080/api");
        assert_eq!(ip, "10.0.0.1");
        assert_eq!(port, 8080);
        assert_eq!(protocol, "http");
        assert_eq!(path, "/api");

        let (ip, port, protocol, path) = StatusService::parse_url("https://example.com:9090");
        assert_eq!(ip, "example.com");
        assert_eq!(port, 9090);
        assert_eq!(protocol, "https");
        assert_eq!(path, "/");
    }

    // ========== 新增测试 (Phase 1.1 - StatusService 覆盖提升) ==========

    #[test]
    fn test_parse_url_with_ipv4() {
        let (ip, port, protocol, path) =
            StatusService::parse_url("http://192.168.1.100:9090/api/v1");
        assert_eq!(ip, "192.168.1.100");
        assert_eq!(port, 9090);
        assert_eq!(protocol, "http");
        assert_eq!(path, "/api/v1");
    }

    #[test]
    fn test_parse_url_without_port() {
        let (ip, port, protocol, path) = StatusService::parse_url("http://example.com/path");
        assert_eq!(ip, "example.com");
        assert_eq!(port, 8080); // 默认端口
        assert_eq!(protocol, "http");
        assert_eq!(path, "/path");
    }

    #[test]
    fn test_parse_url_https_without_path() {
        let (ip, port, protocol, path) = StatusService::parse_url("https://secure.com:443");
        assert_eq!(ip, "secure.com");
        assert_eq!(port, 443);
        assert_eq!(protocol, "https");
        assert_eq!(path, "/");
    }

    #[test]
    fn test_parse_url_localhost() {
        let (ip, port, protocol, path) = StatusService::parse_url("http://localhost:8080");
        assert_eq!(ip, "localhost");
        assert_eq!(port, 8080);
        assert_eq!(protocol, "http");
        assert_eq!(path, "/");
    }

    // ========== StatusService 创建测试 ==========

    #[test]
    fn test_status_service_creation() {
        let service = create_test_service();
        assert_eq!(service.node_id, "test-node");
        assert_eq!(service.region_id, "test-region");
        assert_eq!(service.zone_id, "test-zone");
        assert_eq!(service.server_url, "http://localhost:8080");
        assert_eq!(service.app_id, "test-app");
    }

    #[test]
    fn test_status_service_with_cluster_manager() {
        let service = create_test_service_with_cluster();
        assert!(service.cluster_manager.is_some());
        assert_eq!(service.node_id, "test-node");
    }

    #[test]
    fn test_status_service_without_cluster_manager() {
        let service = create_test_service();
        assert!(service.cluster_manager.is_none());
    }

    // ========== Node Status 测试 ==========

    #[tokio::test]
    async fn test_get_cluster_node_status() {
        let service = create_test_service();
        let request = GetClusterNodeStatusRequest {};

        let response = service.get_cluster_node_status(request).await;

        assert!(response.node_status.is_some());
        let node_status = response.node_status.unwrap();
        assert_eq!(node_status.node.node_id, "test-node");
        assert_eq!(node_status.node.region_id, "test-region");
        assert_eq!(node_status.node.zone_id, "test-zone");
        assert_eq!(node_status.status, node_status::UP);
        assert!(node_status.can_service_discovery);
        assert!(node_status.can_service_registry);
    }

    // ========== Cluster Status 测试 ==========

    #[tokio::test]
    async fn test_get_cluster_status_without_cluster() {
        let service = create_test_service();
        let request = GetClusterStatusRequest {};

        let response = service.get_cluster_status(request).await;

        // 应该只包含当前节点
        assert_eq!(response.node_count, 1);
        assert_eq!(response.nodes_status.len(), 1);
        assert_eq!(response.nodes_status[0].node.node_id, "test-node");
    }

    #[tokio::test]
    async fn test_get_cluster_status_with_cluster() {
        let service = create_test_service_with_cluster();
        let request = GetClusterStatusRequest {};

        let response = service.get_cluster_status(request).await;

        // 至少包含当前节点
        assert!(response.node_count >= 1);
        assert!(!response.nodes_status.is_empty());
        assert!(matches!(
            response.response_status.error_code,
            artemis_core::model::ErrorCode::Success
        ));
    }

    // ========== Config Status 测试 ==========

    #[tokio::test]
    async fn test_get_config_status() {
        let service = create_test_service();
        let request = GetConfigStatusRequest {};

        let response = service.get_config_status(request).await;

        assert!(response.properties.contains_key("node_id"));
        assert_eq!(response.properties.get("node_id").unwrap(), "test-node");
        assert_eq!(response.properties.get("region_id").unwrap(), "test-region");
        assert_eq!(response.properties.get("zone_id").unwrap(), "test-zone");
        assert_eq!(response.properties.get("app_id").unwrap(), "test-app");
        assert!(response.sources.contains_key("default"));
    }

    // ========== Deployment Status 测试 ==========

    #[tokio::test]
    async fn test_get_deployment_status() {
        let service = create_test_service();
        let request = GetDeploymentStatusRequest {};

        let response = service.get_deployment_status(request).await;

        assert_eq!(response.region_id, "test-region");
        assert_eq!(response.zone_id, "test-zone");
        assert_eq!(response.app_id, "test-app");
        assert_eq!(response.ip, "localhost");
        assert_eq!(response.port, 8080);
        assert_eq!(response.protocol, "http");
        assert!(!response.machine_name.is_empty());
        assert!(response.sources.contains_key("default"));
    }
}
