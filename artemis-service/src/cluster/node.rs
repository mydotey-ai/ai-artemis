use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 集群节点状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeStatus {
    /// 节点在线且健康
    Up,
    /// 节点下线或不可达
    Down,
    /// 节点状态未知
    Unknown,
}

/// 集群节点信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterNode {
    /// 节点ID
    pub node_id: String,
    /// 节点地址
    pub address: String,
    /// 节点端口
    pub port: u16,
    /// 节点状态
    pub status: NodeStatus,
    /// 最后心跳时间
    pub last_heartbeat: DateTime<Utc>,
    /// 元数据
    pub metadata: Option<std::collections::HashMap<String, String>>,
}

impl ClusterNode {
    pub fn new(node_id: String, address: String, port: u16) -> Self {
        Self {
            node_id,
            address,
            port,
            status: NodeStatus::Up,
            last_heartbeat: Utc::now(),
            metadata: None,
        }
    }

    /// 从 URL 创建节点(格式: "host:port" 或 "http://host:port")
    pub fn new_from_url(url: String) -> Self {
        // 移除协议前缀
        let url_clean =
            url.strip_prefix("http://").or_else(|| url.strip_prefix("https://")).unwrap_or(&url);

        // 解析 host:port
        let parts: Vec<&str> = url_clean.split(':').collect();
        let address = parts.first().unwrap_or(&"127.0.0.1").to_string();
        let port = parts.get(1).and_then(|p| p.parse().ok()).unwrap_or(8080);

        // 生成节点 ID (使用地址:端口作为 ID)
        let node_id = format!("{}:{}", address, port);

        Self::new(node_id, address, port)
    }

    /// 获取节点的基础 URL
    pub fn base_url(&self) -> String {
        format!("http://{}:{}", self.address, self.port)
    }

    pub fn is_healthy(&self) -> bool {
        matches!(self.status, NodeStatus::Up)
    }

    pub fn update_heartbeat(&mut self) {
        self.last_heartbeat = Utc::now();
        self.status = NodeStatus::Up;
    }

    pub fn update_status(&mut self, is_healthy: bool) {
        self.status = if is_healthy { NodeStatus::Up } else { NodeStatus::Down };
        if is_healthy {
            self.last_heartbeat = Utc::now();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== 基本构造测试 =====

    #[test]
    fn test_node_creation() {
        let node = ClusterNode::new("node-1".to_string(), "192.168.1.100".to_string(), 8080);

        assert_eq!(node.node_id, "node-1");
        assert_eq!(node.address, "192.168.1.100");
        assert_eq!(node.port, 8080);
        assert!(node.is_healthy());
    }

    #[test]
    fn test_new_node_defaults() {
        let node = ClusterNode::new("test".to_string(), "localhost".to_string(), 9090);

        assert_eq!(node.status, NodeStatus::Up, "新节点应该是 Up 状态");
        assert!(node.metadata.is_none(), "元数据应该是 None");
    }

    // ===== URL 解析测试 =====

    #[test]
    fn test_new_from_url_with_http() {
        let node = ClusterNode::new_from_url("http://192.168.1.100:8080".to_string());

        assert_eq!(node.address, "192.168.1.100");
        assert_eq!(node.port, 8080);
        assert_eq!(node.node_id, "192.168.1.100:8080");
    }

    #[test]
    fn test_new_from_url_with_https() {
        let node = ClusterNode::new_from_url("https://example.com:9090".to_string());

        assert_eq!(node.address, "example.com");
        assert_eq!(node.port, 9090);
        assert_eq!(node.node_id, "example.com:9090");
    }

    #[test]
    fn test_new_from_url_without_protocol() {
        let node = ClusterNode::new_from_url("localhost:8080".to_string());

        assert_eq!(node.address, "localhost");
        assert_eq!(node.port, 8080);
        assert_eq!(node.node_id, "localhost:8080");
    }

    #[test]
    fn test_new_from_url_without_port() {
        let node = ClusterNode::new_from_url("http://192.168.1.100".to_string());

        assert_eq!(node.address, "192.168.1.100");
        assert_eq!(node.port, 8080, "缺少端口时应使用默认端口 8080");
    }

    #[test]
    fn test_new_from_url_only_hostname() {
        let node = ClusterNode::new_from_url("example.com".to_string());

        assert_eq!(node.address, "example.com");
        assert_eq!(node.port, 8080, "缺少端口时应使用默认端口 8080");
    }

    #[test]
    fn test_new_from_url_empty_string() {
        let node = ClusterNode::new_from_url("".to_string());

        // 空字符串分割后 parts[0] 是 "",unwrap_or 不会触发
        assert_eq!(node.address, "", "空字符串解析后地址为空");
        assert_eq!(node.port, 8080, "空字符串应使用默认端口");
    }

    #[test]
    fn test_new_from_url_invalid_port() {
        let node = ClusterNode::new_from_url("localhost:invalid".to_string());

        assert_eq!(node.address, "localhost");
        assert_eq!(node.port, 8080, "无效端口应使用默认端口 8080");
    }

    // ===== 基础 URL 生成测试 =====

    #[test]
    fn test_base_url() {
        let node = ClusterNode::new("node-1".to_string(), "192.168.1.100".to_string(), 8080);

        assert_eq!(node.base_url(), "http://192.168.1.100:8080");
    }

    #[test]
    fn test_base_url_with_domain() {
        let node = ClusterNode::new("node-1".to_string(), "example.com".to_string(), 9090);

        assert_eq!(node.base_url(), "http://example.com:9090");
    }

    // ===== 健康状态测试 =====

    #[test]
    fn test_is_healthy_with_up_status() {
        let mut node = ClusterNode::new("node-1".to_string(), "localhost".to_string(), 8080);
        node.status = NodeStatus::Up;

        assert!(node.is_healthy());
    }

    #[test]
    fn test_is_healthy_with_down_status() {
        let mut node = ClusterNode::new("node-1".to_string(), "localhost".to_string(), 8080);
        node.status = NodeStatus::Down;

        assert!(!node.is_healthy());
    }

    #[test]
    fn test_is_healthy_with_unknown_status() {
        let mut node = ClusterNode::new("node-1".to_string(), "localhost".to_string(), 8080);
        node.status = NodeStatus::Unknown;

        assert!(!node.is_healthy());
    }

    // ===== 心跳更新测试 =====

    #[test]
    fn test_heartbeat_update() {
        let mut node = ClusterNode::new("node-1".to_string(), "192.168.1.100".to_string(), 8080);

        node.status = NodeStatus::Down;
        assert!(!node.is_healthy());

        node.update_heartbeat();
        assert!(node.is_healthy());
    }

    #[test]
    fn test_heartbeat_update_from_unknown() {
        let mut node = ClusterNode::new("node-1".to_string(), "localhost".to_string(), 8080);
        node.status = NodeStatus::Unknown;

        let old_heartbeat = node.last_heartbeat;
        std::thread::sleep(std::time::Duration::from_millis(10));

        node.update_heartbeat();

        assert_eq!(node.status, NodeStatus::Up);
        assert!(node.last_heartbeat > old_heartbeat, "心跳时间应该更新");
    }

    // ===== 状态更新测试 =====

    #[test]
    fn test_update_status_to_healthy() {
        let mut node = ClusterNode::new("node-1".to_string(), "localhost".to_string(), 8080);
        node.status = NodeStatus::Down;

        let old_heartbeat = node.last_heartbeat;
        std::thread::sleep(std::time::Duration::from_millis(10));

        node.update_status(true);

        assert_eq!(node.status, NodeStatus::Up);
        assert!(node.last_heartbeat > old_heartbeat, "健康时应更新心跳时间");
    }

    #[test]
    fn test_update_status_to_unhealthy() {
        let mut node = ClusterNode::new("node-1".to_string(), "localhost".to_string(), 8080);

        let old_heartbeat = node.last_heartbeat;
        std::thread::sleep(std::time::Duration::from_millis(10));

        node.update_status(false);

        assert_eq!(node.status, NodeStatus::Down);
        assert_eq!(node.last_heartbeat, old_heartbeat, "不健康时不应更新心跳时间");
    }

    // ===== NodeStatus 枚举测试 =====

    #[test]
    fn test_node_status_equality() {
        assert_eq!(NodeStatus::Up, NodeStatus::Up);
        assert_ne!(NodeStatus::Up, NodeStatus::Down);
        assert_ne!(NodeStatus::Down, NodeStatus::Unknown);
    }

    #[test]
    fn test_node_status_clone() {
        let status = NodeStatus::Up;
        let cloned = status;
        assert_eq!(status, cloned);
    }

    // ===== Clone 测试 =====

    #[test]
    fn test_cluster_node_clone() {
        let node = ClusterNode::new("node-1".to_string(), "192.168.1.100".to_string(), 8080);
        let cloned = node.clone();

        assert_eq!(node.node_id, cloned.node_id);
        assert_eq!(node.address, cloned.address);
        assert_eq!(node.port, cloned.port);
        assert_eq!(node.status, cloned.status);
    }
}
