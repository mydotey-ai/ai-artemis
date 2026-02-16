use super::node::{ClusterNode, NodeStatus};
use dashmap::DashMap;
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, warn};

/// 集群管理器
///
/// 负责管理集群节点，包括：
/// - 节点注册和发现
/// - 主动健康检查
/// - 节点状态管理
#[derive(Clone)]
pub struct ClusterManager {
    /// 当前节点ID
    node_id: String,
    /// 集群节点映射: NodeId -> ClusterNode
    nodes: Arc<DashMap<String, ClusterNode>>,
    /// 心跳超时时间
    heartbeat_timeout: Duration,
}

impl ClusterManager {
    pub fn new(node_id: String, peers: Vec<String>) -> Self {
        let nodes = Arc::new(DashMap::new());

        // 初始化对等节点
        for peer_url in peers {
            let node = ClusterNode::new_from_url(peer_url);
            info!("Adding peer node: {} at {}", node.node_id, node.base_url());
            nodes.insert(node.node_id.clone(), node);
        }

        Self {
            node_id,
            nodes,
            heartbeat_timeout: Duration::from_secs(30),
        }
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

    /// 获取健康的对等节点(排除自己)
    pub fn get_healthy_peers(&self) -> Vec<ClusterNode> {
        self.nodes
            .iter()
            .filter(|entry| {
                entry.key() != &self.node_id && entry.value().is_healthy()
            })
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

    /// 启动健康检查任务
    pub fn start_health_check_task(self: Arc<Self>) {
        tokio::spawn(async move {
            info!("Health check task started (interval: 5s)");

            // 首次延迟,等待所有节点启动
            tokio::time::sleep(Duration::from_secs(10)).await;

            let mut interval = tokio::time::interval(Duration::from_secs(5));

            loop {
                interval.tick().await;

                // 并发执行所有节点的健康检查
                let mut tasks = vec![];

                for node_entry in self.nodes.iter() {
                    let node_id = node_entry.key().clone();
                    let base_url = node_entry.value().base_url();

                    // 为每个节点启动独立的健康检查任务
                    let task = tokio::spawn(async move {
                        let is_healthy = check_node_health(&base_url).await;
                        (node_id, is_healthy)
                    });

                    tasks.push(task);
                }

                // 等待所有健康检查完成
                for task in tasks {
                    if let Ok((node_id, is_healthy)) = task.await {
                        // 更新节点状态
                        if let Some(mut node) = self.nodes.get_mut(&node_id) {
                            let was_healthy = node.is_healthy();
                            node.update_status(is_healthy);

                            // 状态变化时记录日志
                            if was_healthy != is_healthy {
                                if is_healthy {
                                    info!("Node {} is now UP", node_id);
                                } else {
                                    warn!("Node {} is now DOWN", node_id);
                                }
                            }
                        }
                    }
                }
            }
        });
    }
}

/// 检查节点健康状态
async fn check_node_health(base_url: &str) -> bool {
    let health_url = format!("{}/health", base_url);

    // 创建带超时的客户端 (避免长时间阻塞)
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());

    match client.get(&health_url).send().await {
        Ok(response) => response.status().is_success(),
        Err(e) => {
            tracing::debug!("Health check failed for {}: {}", base_url, e);
            false
        }
    }
}

impl Default for ClusterManager {
    fn default() -> Self {
        Self::new("default-node".to_string(), vec![])
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

    #[test]
    fn test_cluster_manager_new_with_peers() {
        let peers = vec![
            "http://192.168.1.101:8080".to_string(),
            "http://192.168.1.102:8080".to_string(),
        ];
        let manager = ClusterManager::new("node-0".to_string(), peers);

        assert_eq!(manager.node_count(), 2);
        assert_eq!(manager.node_id, "node-0");
    }

    #[test]
    fn test_cluster_manager_default() {
        let manager = ClusterManager::default();

        assert_eq!(manager.node_id, "default-node");
        assert_eq!(manager.node_count(), 0);
    }

    #[test]
    fn test_get_healthy_peers() {
        let manager = ClusterManager::new("node-0".to_string(), vec![]);

        // Add self node (should be excluded from peers)
        let self_node = ClusterNode::new("node-0".to_string(), "127.0.0.1".to_string(), 8080);
        manager.register_node(self_node);

        // Add other nodes
        let node1 = ClusterNode::new("node-1".to_string(), "192.168.1.101".to_string(), 8080);
        manager.register_node(node1);

        let node2 = ClusterNode::new("node-2".to_string(), "192.168.1.102".to_string(), 8080);
        manager.register_node(node2);

        let peers = manager.get_healthy_peers();
        // Should not include self
        assert_eq!(peers.len(), 2);
        assert!(peers.iter().all(|n| n.node_id != "node-0"));
    }

    #[test]
    fn test_mark_node_down() {
        let manager = ClusterManager::default();

        let node = ClusterNode::new("node-1".to_string(), "192.168.1.100".to_string(), 8080);
        manager.register_node(node);

        manager.mark_node_down("node-1");

        let healthy = manager.get_healthy_nodes();
        assert_eq!(healthy.len(), 0); // Node should be marked down
    }

    #[test]
    fn test_mark_nonexistent_node_down() {
        let manager = ClusterManager::default();

        // Should not panic
        manager.mark_node_down("nonexistent-node");
    }

    #[test]
    fn test_check_expired_nodes_empty() {
        let manager = ClusterManager::default();

        let expired = manager.check_expired_nodes();
        assert_eq!(expired.len(), 0);
    }

    #[test]
    fn test_check_expired_nodes_with_fresh_nodes() {
        let manager = ClusterManager::default();

        let node = ClusterNode::new("node-1".to_string(), "192.168.1.100".to_string(), 8080);
        manager.register_node(node);

        // Just registered, should not be expired
        let expired = manager.check_expired_nodes();
        assert_eq!(expired.len(), 0);
    }

    #[test]
    fn test_node_count() {
        let manager = ClusterManager::default();

        assert_eq!(manager.node_count(), 0);

        manager.register_node(ClusterNode::new("node-1".to_string(), "127.0.0.1".to_string(), 8080));
        assert_eq!(manager.node_count(), 1);

        manager.register_node(ClusterNode::new("node-2".to_string(), "127.0.0.2".to_string(), 8080));
        assert_eq!(manager.node_count(), 2);
    }

    #[test]
    fn test_cluster_manager_clone() {
        let manager1 = ClusterManager::default();
        manager1.register_node(ClusterNode::new("node-1".to_string(), "127.0.0.1".to_string(), 8080));

        let manager2 = manager1.clone();
        assert_eq!(manager2.node_count(), 1);
        assert_eq!(manager2.node_id, manager1.node_id);
    }

    #[test]
    fn test_multiple_nodes_registration() {
        let manager = ClusterManager::default();

        for i in 1..=5 {
            let node = ClusterNode::new(
                format!("node-{}", i),
                format!("192.168.1.{}", 100 + i),
                8080,
            );
            manager.register_node(node);
        }

        assert_eq!(manager.node_count(), 5);
        let healthy = manager.get_healthy_nodes();
        assert_eq!(healthy.len(), 5);
    }

    #[test]
    fn test_heartbeat_update_nonexistent() {
        let manager = ClusterManager::default();

        let result = manager.update_heartbeat("nonexistent");
        assert!(!result);
    }

    #[test]
    fn test_get_healthy_nodes_with_down_nodes() {
        let manager = ClusterManager::default();

        let node1 = ClusterNode::new("node-1".to_string(), "192.168.1.101".to_string(), 8080);
        let node2 = ClusterNode::new("node-2".to_string(), "192.168.1.102".to_string(), 8080);
        manager.register_node(node1);
        manager.register_node(node2);

        manager.mark_node_down("node-1");

        let healthy = manager.get_healthy_nodes();
        assert_eq!(healthy.len(), 1);
        assert_eq!(healthy[0].node_id, "node-2");
    }

    #[tokio::test]
    async fn test_check_node_health_invalid_url() {
        let result = check_node_health("http://invalid-host-that-does-not-exist:9999").await;
        assert!(!result);
    }
}
