use super::worker::ReplicationWorker;
use crate::cluster::ClusterManager;
use artemis_core::config::ReplicationConfig;
use artemis_core::model::{Instance, InstanceKey};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tracing::info;

/// 复制事件类型
#[derive(Debug, Clone)]
pub enum ReplicationEvent {
    /// 注册事件
    Register(Instance),
    /// 注销事件
    Unregister(InstanceKey),
    /// 心跳事件
    Heartbeat(InstanceKey),
}

/// 数据复制管理器框架
///
/// 负责跨集群节点的数据复制：
/// - 事件采集
/// - 批量复制优化
/// - 目标节点选择
///
/// Note: 这是框架实现，完整功能待Phase 10详细实施
#[derive(Clone)]
pub struct ReplicationManager {
    event_tx: Arc<mpsc::UnboundedSender<ReplicationEvent>>,
}

impl ReplicationManager {
    pub fn new() -> (Self, mpsc::UnboundedReceiver<ReplicationEvent>) {
        let (event_tx, event_rx) = mpsc::unbounded_channel();

        (Self { event_tx: Arc::new(event_tx) }, event_rx)
    }

    /// 发布注册事件
    pub fn publish_register(&self, instance: Instance) {
        if let Err(e) = self.event_tx.send(ReplicationEvent::Register(instance)) {
            tracing::error!("Failed to publish register event: {}", e);
        }
    }

    /// 发布注销事件
    pub fn publish_unregister(&self, key: InstanceKey) {
        if let Err(e) = self.event_tx.send(ReplicationEvent::Unregister(key)) {
            tracing::error!("Failed to publish unregister event: {}", e);
        }
    }

    /// 发布心跳事件
    pub fn publish_heartbeat(&self, key: InstanceKey) {
        if let Err(e) = self.event_tx.send(ReplicationEvent::Heartbeat(key)) {
            tracing::error!("Failed to publish heartbeat event: {}", e);
        }
    }

    /// 启动复制工作器
    pub fn start_worker(
        event_rx: mpsc::UnboundedReceiver<ReplicationEvent>,
        cluster_manager: Arc<ClusterManager>,
        config: ReplicationConfig,
    ) -> JoinHandle<()> {
        info!("Starting replication worker");
        let worker = ReplicationWorker::new(event_rx, cluster_manager, config);
        worker.start()
    }

    /// 启动复制任务(已废弃,使用 start_worker)
    #[deprecated(note = "Use start_worker instead")]
    pub fn start_replication_task(mut event_rx: mpsc::UnboundedReceiver<ReplicationEvent>) {
        tokio::spawn(async move {
            info!("Replication task started");

            while let Some(event) = event_rx.recv().await {
                match event {
                    ReplicationEvent::Register(instance) => {
                        info!("Would replicate register: {}", instance.instance_id);
                    }
                    ReplicationEvent::Unregister(key) => {
                        info!("Would replicate unregister: {}", key.instance_id);
                    }
                    ReplicationEvent::Heartbeat(key) => {
                        info!("Would replicate heartbeat: {}", key.instance_id);
                    }
                }
            }
        });
    }
}

impl Default for ReplicationManager {
    fn default() -> Self {
        let (manager, _rx) = Self::new();
        manager
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use artemis_core::model::InstanceStatus;

    fn create_test_instance() -> Instance {
        Instance {
            region_id: "test".to_string(),
            zone_id: "zone".to_string(),
            service_id: "service".to_string(),
            group_id: None,
            instance_id: "inst-1".to_string(),
            machine_name: None,
            ip: "127.0.0.1".to_string(),
            port: 8080,
            protocol: None,
            url: "http://127.0.0.1:8080".to_string(),
            health_check_url: None,
            status: InstanceStatus::Up,
            metadata: None,
        }
    }

    #[tokio::test]
    async fn test_replication_events() {
        let (manager, mut rx) = ReplicationManager::new();

        let instance = create_test_instance();
        manager.publish_register(instance.clone());

        let event = rx.recv().await.unwrap();
        match event {
            ReplicationEvent::Register(inst) => {
                assert_eq!(inst.instance_id, "inst-1");
            }
            _ => panic!("Unexpected event type"),
        }
    }

    #[tokio::test]
    async fn test_publish_unregister() {
        let (manager, mut rx) = ReplicationManager::new();

        let key = InstanceKey {
            region_id: "test".to_string(),
            zone_id: "zone".to_string(),
            group_id: String::new(),
            service_id: "service".to_string(),
            instance_id: "inst-1".to_string(),
        };
        manager.publish_unregister(key.clone());

        let event = rx.recv().await.unwrap();
        match event {
            ReplicationEvent::Unregister(k) => {
                assert_eq!(k.instance_id, "inst-1");
                assert_eq!(k.service_id, "service");
            }
            _ => panic!("Expected Unregister event"),
        }
    }

    #[tokio::test]
    async fn test_publish_heartbeat() {
        let (manager, mut rx) = ReplicationManager::new();

        let key = InstanceKey {
            region_id: "test".to_string(),
            zone_id: "zone".to_string(),
            group_id: String::new(),
            service_id: "service".to_string(),
            instance_id: "inst-1".to_string(),
        };
        manager.publish_heartbeat(key.clone());

        let event = rx.recv().await.unwrap();
        match event {
            ReplicationEvent::Heartbeat(k) => {
                assert_eq!(k.instance_id, "inst-1");
                assert_eq!(k.service_id, "service");
            }
            _ => panic!("Expected Heartbeat event"),
        }
    }

    #[tokio::test]
    async fn test_multiple_events() {
        let (manager, mut rx) = ReplicationManager::new();

        let instance = create_test_instance();
        let key = InstanceKey {
            region_id: "test".to_string(),
            zone_id: "zone".to_string(),
            group_id: String::new(),
            service_id: "service".to_string(),
            instance_id: "inst-2".to_string(),
        };

        manager.publish_register(instance.clone());
        manager.publish_heartbeat(key.clone());
        manager.publish_unregister(key.clone());

        // 第一个事件: Register
        let event1 = rx.recv().await.unwrap();
        assert!(matches!(event1, ReplicationEvent::Register(_)));

        // 第二个事件: Heartbeat
        let event2 = rx.recv().await.unwrap();
        assert!(matches!(event2, ReplicationEvent::Heartbeat(_)));

        // 第三个事件: Unregister
        let event3 = rx.recv().await.unwrap();
        assert!(matches!(event3, ReplicationEvent::Unregister(_)));
    }

    #[tokio::test]
    async fn test_replication_manager_default() {
        let manager = ReplicationManager::default();

        let instance = create_test_instance();
        manager.publish_register(instance);

        // Default manager 丢弃了接收器,事件会被发送但不会被接收
        // 这个测试确保 default() 不会 panic
    }

    #[test]
    fn test_replication_event_clone() {
        let instance = create_test_instance();
        let event = ReplicationEvent::Register(instance.clone());
        let cloned = event.clone();

        match (event, cloned) {
            (ReplicationEvent::Register(i1), ReplicationEvent::Register(i2)) => {
                assert_eq!(i1.instance_id, i2.instance_id);
            }
            _ => panic!("Clone failed"),
        }
    }

    #[test]
    fn test_replication_event_debug() {
        let instance = create_test_instance();
        let event = ReplicationEvent::Register(instance);
        let debug_str = format!("{:?}", event);
        assert!(debug_str.contains("Register"));
    }

    #[tokio::test]
    async fn test_manager_clone() {
        let (manager1, mut rx) = ReplicationManager::new();
        let manager2 = manager1.clone();

        let instance = create_test_instance();
        manager2.publish_register(instance.clone());

        let event = rx.recv().await.unwrap();
        match event {
            ReplicationEvent::Register(inst) => {
                assert_eq!(inst.instance_id, "inst-1");
            }
            _ => panic!("Unexpected event type"),
        }
    }

    #[tokio::test]
    async fn test_start_replication_task_deprecated() {
        let (_manager, event_rx) = ReplicationManager::new();

        // 测试已废弃的 API (确保不会崩溃)
        #[allow(deprecated)]
        ReplicationManager::start_replication_task(event_rx);

        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    #[tokio::test]
    async fn test_publish_after_receiver_dropped() {
        let (manager, rx) = ReplicationManager::new();

        // 丢弃接收器
        drop(rx);

        let instance = create_test_instance();
        // 发送应该失败,但不会 panic
        manager.publish_register(instance);
    }

    #[tokio::test]
    async fn test_event_ordering() {
        let (manager, mut rx) = ReplicationManager::new();

        let instance1 = create_test_instance();
        let mut instance2 = create_test_instance();
        instance2.instance_id = "inst-2".to_string();
        let mut instance3 = create_test_instance();
        instance3.instance_id = "inst-3".to_string();

        manager.publish_register(instance1);
        manager.publish_register(instance2);
        manager.publish_register(instance3);

        // 验证事件顺序
        let event1 = rx.recv().await.unwrap();
        match event1 {
            ReplicationEvent::Register(i) => assert_eq!(i.instance_id, "inst-1"),
            _ => panic!("Expected Register event"),
        }

        let event2 = rx.recv().await.unwrap();
        match event2 {
            ReplicationEvent::Register(i) => assert_eq!(i.instance_id, "inst-2"),
            _ => panic!("Expected Register event"),
        }

        let event3 = rx.recv().await.unwrap();
        match event3 {
            ReplicationEvent::Register(i) => assert_eq!(i.instance_id, "inst-3"),
            _ => panic!("Expected Register event"),
        }
    }
}
