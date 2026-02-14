use artemis_core::model::{ChangeType, Instance, InstanceChange, InstanceKey};
use chrono::Utc;
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::mpsc;

type ChangeReceiver = mpsc::UnboundedReceiver<InstanceChange>;
type ChangeSender = mpsc::UnboundedSender<InstanceChange>;

/// 实例变更管理器
#[derive(Clone)]
pub struct InstanceChangeManager {
    /// 服务变更通道: ServiceId -> Sender
    channels: Arc<DashMap<String, ChangeSender>>,
}

impl InstanceChangeManager {
    pub fn new() -> Self {
        Self { channels: Arc::new(DashMap::new()) }
    }

    /// 订阅服务变更
    pub fn subscribe(&self, service_id: &str) -> ChangeReceiver {
        let (tx, rx) = mpsc::unbounded_channel();
        self.channels.insert(service_id.to_string(), tx);
        rx
    }

    /// 发布实例变更
    pub fn publish(&self, service_id: &str, change: InstanceChange) {
        if let Some(sender) = self.channels.get(service_id)
            && let Err(e) = sender.send(change)
        {
            tracing::error!("Failed to publish change: {}", e);
        }
    }

    /// 发布实例注册事件
    pub fn publish_register(&self, instance: &Instance) {
        let change = InstanceChange {
            instance: instance.clone(),
            change_type: ChangeType::New,
            change_time: Utc::now(),
        };

        self.publish(&instance.service_id, change);
    }

    /// 发布实例注销事件
    pub fn publish_unregister(&self, key: &InstanceKey, instance: &Instance) {
        let change = InstanceChange {
            instance: instance.clone(),
            change_type: ChangeType::Delete,
            change_time: Utc::now(),
        };

        self.publish(&key.service_id, change);
    }

    /// 发布实例状态变更事件
    pub fn publish_update(&self, instance: &Instance) {
        let change = InstanceChange {
            instance: instance.clone(),
            change_type: ChangeType::Change,
            change_time: Utc::now(),
        };

        self.publish(&instance.service_id, change);
    }

    /// 获取订阅数量
    pub fn subscription_count(&self) -> usize {
        self.channels.len()
    }
}

impl Default for InstanceChangeManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use artemis_core::model::InstanceStatus;

    fn create_test_instance(id: &str) -> Instance {
        Instance {
            region_id: "test-region".to_string(),
            zone_id: "test-zone".to_string(),
            service_id: "test-service".to_string(),
            group_id: None,
            instance_id: id.to_string(),
            machine_name: None,
            ip: "127.0.0.1".to_string(),
            port: 8080,
            protocol: None,
            url: format!("http://127.0.0.1:8080/{}", id),
            health_check_url: None,
            status: InstanceStatus::Up,
            metadata: None,
        }
    }

    #[tokio::test]
    async fn test_subscribe_and_publish() {
        let manager = InstanceChangeManager::new();
        let mut rx = manager.subscribe("test-service");

        let instance = create_test_instance("inst-1");
        manager.publish_register(&instance);

        let change = rx.recv().await.unwrap();
        assert_eq!(change.change_type, ChangeType::New);
        assert_eq!(change.instance.instance_id, "inst-1");
    }

    #[test]
    fn test_subscription_count() {
        let manager = InstanceChangeManager::new();
        assert_eq!(manager.subscription_count(), 0);

        let _rx1 = manager.subscribe("service-1");
        assert_eq!(manager.subscription_count(), 1);

        let _rx2 = manager.subscribe("service-2");
        assert_eq!(manager.subscription_count(), 2);
    }
}
