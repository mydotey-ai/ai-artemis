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

    pub fn is_healthy(&self) -> bool {
        matches!(self.status, NodeStatus::Up)
    }

    pub fn update_heartbeat(&mut self) {
        self.last_heartbeat = Utc::now();
        self.status = NodeStatus::Up;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_creation() {
        let node = ClusterNode::new("node-1".to_string(), "192.168.1.100".to_string(), 8080);

        assert_eq!(node.node_id, "node-1");
        assert_eq!(node.address, "192.168.1.100");
        assert_eq!(node.port, 8080);
        assert!(node.is_healthy());
    }

    #[test]
    fn test_heartbeat_update() {
        let mut node = ClusterNode::new("node-1".to_string(), "192.168.1.100".to_string(), 8080);

        node.status = NodeStatus::Down;
        assert!(!node.is_healthy());

        node.update_heartbeat();
        assert!(node.is_healthy());
    }
}
