use super::node::{ClusterNode, NodeStatus};
use dashmap::DashMap;
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, warn};

/// 集群管理器框架
///
/// 负责管理集群节点，包括：
/// - 节点注册和发现
/// - 健康检查
/// - 节点状态管理
///
/// Note: 这是框架实现，完整功能待Phase 10详细实施
#[derive(Clone)]
pub struct ClusterManager {
    /// 集群节点映射: NodeId -> ClusterNode
    nodes: Arc<DashMap<String, ClusterNode>>,
    /// 心跳超时时间
    heartbeat_timeout: Duration,
}

impl ClusterManager {
    pub fn new(heartbeat_timeout: Duration) -> Self {
        Self { nodes: Arc::new(DashMap::new()), heartbeat_timeout }
    }

    /// 注册新节点
    pub fn register_node(&self, node: ClusterNode) {
        info!("Registering cluster node: {}", node.node_id);
        self.nodes.insert(node.node_id.clone(), node);
    }

    /// 更新节点心跳
    pub fn update_heartbeat(&self, node_id: &str) -> bool {
        if let Some(mut node) = self.nodes.get_mut(node_id) {
            node.update_heartbeat();
            true
        } else {
            warn!("Node not found: {}", node_id);
            false
        }
    }

    /// 获取所有健康节点
    pub fn get_healthy_nodes(&self) -> Vec<ClusterNode> {
        self.nodes
            .iter()
            .filter(|entry| entry.value().is_healthy())
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// 获取节点数量
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// 检查过期节点（框架方法，完整实现待补充）
    pub fn check_expired_nodes(&self) -> Vec<String> {
        let now = chrono::Utc::now();
        self.nodes
            .iter()
            .filter(|entry| {
                let elapsed = now.signed_duration_since(entry.value().last_heartbeat);
                elapsed.num_seconds() as u64 > self.heartbeat_timeout.as_secs()
            })
            .map(|entry| entry.key().clone())
            .collect()
    }

    /// 标记节点为下线
    pub fn mark_node_down(&self, node_id: &str) {
        if let Some(mut node) = self.nodes.get_mut(node_id) {
            node.status = NodeStatus::Down;
            warn!("Node marked as down: {}", node_id);
        }
    }

    /// 启动健康检查任务（框架方法）
    pub fn start_health_check_task(self, check_interval: Duration) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(check_interval);
            loop {
                interval.tick().await;

                let expired = self.check_expired_nodes();
                if !expired.is_empty() {
                    info!("Found {} expired nodes", expired.len());
                    for node_id in expired {
                        self.mark_node_down(&node_id);
                    }
                }
            }
        });
    }
}

impl Default for ClusterManager {
    fn default() -> Self {
        Self::new(Duration::from_secs(30))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_node() {
        let manager = ClusterManager::default();
        let node = ClusterNode::new("node-1".to_string(), "192.168.1.100".to_string(), 8080);

        manager.register_node(node);
        assert_eq!(manager.node_count(), 1);
    }

    #[test]
    fn test_get_healthy_nodes() {
        let manager = ClusterManager::default();

        let node1 = ClusterNode::new("node-1".to_string(), "192.168.1.100".to_string(), 8080);
        manager.register_node(node1);

        let healthy = manager.get_healthy_nodes();
        assert_eq!(healthy.len(), 1);
    }

    #[test]
    fn test_heartbeat_update() {
        let manager = ClusterManager::default();

        let node = ClusterNode::new("node-1".to_string(), "192.168.1.100".to_string(), 8080);
        manager.register_node(node);

        assert!(manager.update_heartbeat("node-1"));
        assert!(!manager.update_heartbeat("non-existent"));
    }
}
