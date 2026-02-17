//! Cluster Manager 测试
//!
//! 测试覆盖:
//! - 节点注册和管理
//! - 心跳更新机制
//! - 健康节点过滤
//! - 节点过期检查
//! - 并发操作

use artemis_server::cluster::{ClusterManager, ClusterNode, NodeStatus};
use std::sync::Arc;

// ===== 节点注册和管理测试 =====

#[test]
fn test_new_cluster_manager_with_peers() {
    let peers =
        vec!["http://192.168.1.101:8080".to_string(), "http://192.168.1.102:8080".to_string()];

    let manager = ClusterManager::new("node-0".to_string(), peers);

    assert_eq!(manager.node_count(), 2, "应该有 2 个对等节点");
}

#[test]
fn test_new_cluster_manager_without_peers() {
    let manager = ClusterManager::new("node-0".to_string(), vec![]);

    assert_eq!(manager.node_count(), 0, "没有对等节点时应为 0");
}

#[test]
fn test_register_node() {
    let manager = ClusterManager::default();

    let node = ClusterNode::new("node-1".to_string(), "192.168.1.100".to_string(), 8080);
    manager.register_node(node);

    assert_eq!(manager.node_count(), 1, "注册后应该有 1 个节点");
}

#[test]
fn test_register_multiple_nodes() {
    let manager = ClusterManager::default();

    for i in 1..=5 {
        let node = ClusterNode::new(format!("node-{}", i), format!("192.168.1.{}", 100 + i), 8080);
        manager.register_node(node);
    }

    assert_eq!(manager.node_count(), 5, "应该注册 5 个节点");
}

#[test]
fn test_register_duplicate_node_replaces() {
    let manager = ClusterManager::default();

    let node1 = ClusterNode::new("node-1".to_string(), "192.168.1.100".to_string(), 8080);
    manager.register_node(node1);

    let node2 = ClusterNode::new("node-1".to_string(), "192.168.1.200".to_string(), 9090);
    manager.register_node(node2);

    assert_eq!(manager.node_count(), 1, "重复节点 ID 应该替换");

    let nodes = manager.get_healthy_nodes();
    assert_eq!(nodes[0].address, "192.168.1.200", "应该使用最新的地址");
}

// ===== 心跳更新测试 =====

#[test]
fn test_update_heartbeat_existing_node() {
    let manager = ClusterManager::default();

    let node = ClusterNode::new("node-1".to_string(), "192.168.1.100".to_string(), 8080);
    manager.register_node(node);

    let result = manager.update_heartbeat("node-1");
    assert!(result, "更新已存在节点的心跳应该成功");
}

#[test]
fn test_update_heartbeat_nonexistent_node() {
    let manager = ClusterManager::default();

    let result = manager.update_heartbeat("nonexistent");
    assert!(!result, "更新不存在节点的心跳应该失败");
}

#[test]
fn test_update_heartbeat_revives_down_node() {
    let manager = ClusterManager::default();

    let mut node = ClusterNode::new("node-1".to_string(), "192.168.1.100".to_string(), 8080);
    node.status = NodeStatus::Down;
    manager.register_node(node);

    assert_eq!(manager.get_healthy_nodes().len(), 0, "DOWN 节点不应该在健康列表");

    manager.update_heartbeat("node-1");

    assert_eq!(manager.get_healthy_nodes().len(), 1, "心跳更新应该恢复节点为 UP");
}

// ===== 健康节点过滤测试 =====

#[test]
fn test_get_healthy_nodes_all_up() {
    let manager = ClusterManager::default();

    for i in 1..=3 {
        let node = ClusterNode::new(format!("node-{}", i), format!("192.168.1.{}", 100 + i), 8080);
        manager.register_node(node);
    }

    let healthy = manager.get_healthy_nodes();
    assert_eq!(healthy.len(), 3, "所有节点都是 UP 时应返回全部");
}

#[test]
fn test_get_healthy_nodes_mixed_status() {
    let manager = ClusterManager::default();

    let mut node1 = ClusterNode::new("node-1".to_string(), "192.168.1.101".to_string(), 8080);
    node1.status = NodeStatus::Up;
    manager.register_node(node1);

    let mut node2 = ClusterNode::new("node-2".to_string(), "192.168.1.102".to_string(), 8080);
    node2.status = NodeStatus::Down;
    manager.register_node(node2);

    let mut node3 = ClusterNode::new("node-3".to_string(), "192.168.1.103".to_string(), 8080);
    node3.status = NodeStatus::Unknown;
    manager.register_node(node3);

    let healthy = manager.get_healthy_nodes();
    assert_eq!(healthy.len(), 1, "只有 UP 节点应该在健康列表");
    assert_eq!(healthy[0].node_id, "node-1");
}

#[test]
fn test_get_healthy_nodes_empty() {
    let manager = ClusterManager::default();

    let healthy = manager.get_healthy_nodes();
    assert_eq!(healthy.len(), 0, "没有节点时应返回空列表");
}

#[test]
fn test_get_healthy_peers_excludes_self() {
    let manager = ClusterManager::new("node-1".to_string(), vec![]);

    let node1 = ClusterNode::new("node-1".to_string(), "192.168.1.101".to_string(), 8080);
    manager.register_node(node1);

    let node2 = ClusterNode::new("node-2".to_string(), "192.168.1.102".to_string(), 8080);
    manager.register_node(node2);

    let peers = manager.get_healthy_peers();
    assert_eq!(peers.len(), 1, "应该排除自己,只返回对等节点");
    assert_eq!(peers[0].node_id, "node-2");
}

#[test]
fn test_get_healthy_peers_only_self() {
    let manager = ClusterManager::new("node-1".to_string(), vec![]);

    let node1 = ClusterNode::new("node-1".to_string(), "192.168.1.101".to_string(), 8080);
    manager.register_node(node1);

    let peers = manager.get_healthy_peers();
    assert_eq!(peers.len(), 0, "只有自己时应返回空列表");
}

// ===== 节点过期检查测试 =====

#[tokio::test]
async fn test_check_expired_nodes_recent_heartbeat() {
    let manager = ClusterManager::default();

    let node = ClusterNode::new("node-1".to_string(), "192.168.1.100".to_string(), 8080);
    manager.register_node(node);

    let expired = manager.check_expired_nodes();
    assert_eq!(expired.len(), 0, "最近心跳的节点不应该过期");
}

#[tokio::test]
async fn test_check_expired_nodes_old_heartbeat() {
    let manager = ClusterManager::default();

    let mut node = ClusterNode::new("node-1".to_string(), "192.168.1.100".to_string(), 8080);
    // 设置一个旧的心跳时间
    node.last_heartbeat = chrono::Utc::now() - chrono::Duration::seconds(60);
    manager.register_node(node);

    let expired = manager.check_expired_nodes();
    assert_eq!(expired.len(), 1, "超过 30 秒的节点应该过期");
    assert_eq!(expired[0], "node-1");
}

#[tokio::test]
async fn test_check_expired_nodes_mixed() {
    let manager = ClusterManager::default();

    // 新节点
    let node1 = ClusterNode::new("node-1".to_string(), "192.168.1.101".to_string(), 8080);
    manager.register_node(node1);

    // 旧节点
    let mut node2 = ClusterNode::new("node-2".to_string(), "192.168.1.102".to_string(), 8080);
    node2.last_heartbeat = chrono::Utc::now() - chrono::Duration::seconds(60);
    manager.register_node(node2);

    let expired = manager.check_expired_nodes();
    assert_eq!(expired.len(), 1, "只有旧节点应该过期");
    assert_eq!(expired[0], "node-2");
}

// ===== 节点状态管理测试 =====

#[test]
fn test_mark_node_down() {
    let manager = ClusterManager::default();

    let node = ClusterNode::new("node-1".to_string(), "192.168.1.100".to_string(), 8080);
    manager.register_node(node);

    assert_eq!(manager.get_healthy_nodes().len(), 1, "初始应该是健康的");

    manager.mark_node_down("node-1");

    assert_eq!(manager.get_healthy_nodes().len(), 0, "标记为 DOWN 后应该不健康");
}

#[test]
fn test_mark_nonexistent_node_down_is_safe() {
    let manager = ClusterManager::default();

    // 标记不存在的节点应该安全,不应该 panic
    manager.mark_node_down("nonexistent");
}

// ===== Default 和 Clone 测试 =====

#[test]
fn test_default_constructor() {
    let manager = ClusterManager::default();

    assert_eq!(manager.node_count(), 0, "默认构造器应该没有节点");
}

#[test]
fn test_clone_shares_state() {
    let manager1 = ClusterManager::default();

    let node = ClusterNode::new("node-1".to_string(), "192.168.1.100".to_string(), 8080);
    manager1.register_node(node);

    let manager2 = manager1.clone();

    assert_eq!(manager2.node_count(), 1, "克隆应该共享节点状态");

    let node2 = ClusterNode::new("node-2".to_string(), "192.168.1.101".to_string(), 8080);
    manager2.register_node(node2);

    assert_eq!(manager1.node_count(), 2, "manager1 应该看到 manager2 的变更");
}

// ===== 并发操作测试 =====

#[test]
fn test_concurrent_node_registration() {
    use std::thread;

    let manager = Arc::new(ClusterManager::default());
    let mut handles = vec![];

    for i in 0..10 {
        let mgr = manager.clone();
        let handle = thread::spawn(move || {
            let node =
                ClusterNode::new(format!("node-{}", i), format!("192.168.1.{}", 100 + i), 8080);
            mgr.register_node(node);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(manager.node_count(), 10, "并发注册应该成功添加所有节点");
}

#[test]
fn test_concurrent_heartbeat_updates() {
    use std::thread;

    let manager = Arc::new(ClusterManager::default());

    // 先注册 5 个节点
    for i in 0..5 {
        let node = ClusterNode::new(format!("node-{}", i), format!("192.168.1.{}", 100 + i), 8080);
        manager.register_node(node);
    }

    let mut handles = vec![];

    // 并发更新心跳
    for i in 0..5 {
        let mgr = manager.clone();
        let handle = thread::spawn(move || {
            for _ in 0..10 {
                mgr.update_heartbeat(&format!("node-{}", i));
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(manager.get_healthy_nodes().len(), 5, "所有节点应该仍然健康");
}

#[test]
fn test_concurrent_read_and_write() {
    use std::thread;

    let manager = Arc::new(ClusterManager::default());
    let mut handles = vec![];

    // 5 个写线程
    for i in 0..5 {
        let mgr = manager.clone();
        let handle = thread::spawn(move || {
            for j in 0..5 {
                let node = ClusterNode::new(
                    format!("node-{}-{}", i, j),
                    format!("192.168.{}.{}", i, j),
                    8080,
                );
                mgr.register_node(node);
            }
        });
        handles.push(handle);
    }

    // 5 个读线程
    for _ in 0..5 {
        let mgr = manager.clone();
        let handle = thread::spawn(move || {
            for _ in 0..10 {
                let _ = mgr.get_healthy_nodes();
                let _ = mgr.node_count();
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(manager.node_count(), 25, "应该注册 25 个节点");
}
