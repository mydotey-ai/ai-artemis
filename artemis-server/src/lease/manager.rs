use artemis_core::model::{InstanceKey, Lease};
use dashmap::DashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::time;
use tracing::info;

/// 租约管理器 - 管理实例租约和过期清理
#[derive(Clone)]
pub struct LeaseManager {
    leases: Arc<DashMap<InstanceKey, Arc<Lease>>>,
    ttl: Duration,
}

impl LeaseManager {
    pub fn new(ttl: Duration) -> Self {
        Self { leases: Arc::new(DashMap::new()), ttl }
    }

    /// 创建租约
    pub fn create_lease(&self, key: InstanceKey) -> Arc<Lease> {
        let lease = Arc::new(Lease::new(key.clone(), self.ttl));
        self.leases.insert(key, lease.clone());
        lease
    }

    /// 续约
    pub fn renew(&self, key: &InstanceKey) -> bool {
        if let Some(lease) = self.leases.get(key) {
            lease.renew();
            true
        } else {
            false
        }
    }

    /// 删除租约
    pub fn remove_lease(&self, key: &InstanceKey) -> Option<Arc<Lease>> {
        self.leases.remove(key).map(|(_, v)| v)
    }

    /// 检查租约是否存在且未过期
    pub fn is_valid(&self, key: &InstanceKey) -> bool {
        self.leases.get(key).map(|lease| !lease.is_expired()).unwrap_or(false)
    }

    /// 获取所有过期的租约key
    pub fn get_expired_keys(&self) -> Vec<InstanceKey> {
        self.leases
            .iter()
            .filter(|entry| entry.value().is_expired())
            .map(|entry| entry.key().clone())
            .collect()
    }

    /// 启动后台清理任务
    pub fn start_eviction_task(
        self,
        eviction_interval: Duration,
        on_evict: impl Fn(InstanceKey) + Send + Sync + 'static,
    ) {
        tokio::spawn(async move {
            let mut interval = time::interval(eviction_interval);
            loop {
                interval.tick().await;
                let expired_keys = self.get_expired_keys();

                if !expired_keys.is_empty() {
                    info!("Evicting {} expired leases", expired_keys.len());
                    for key in expired_keys {
                        if let Some(lease) = self.remove_lease(&key) {
                            lease.mark_evicted();
                            on_evict(key);
                        }
                    }
                }
            }
        });
    }

    /// 获取租约数量
    pub fn count(&self) -> usize {
        self.leases.len()
    }

    /// 获取所有租约 (用于状态查询)
    pub fn get_all_leases(&self) -> Vec<(InstanceKey, Arc<Lease>)> {
        self.leases.iter().map(|entry| (entry.key().clone(), entry.value().clone())).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_key(id: &str) -> InstanceKey {
        InstanceKey {
            region_id: "test".to_string(),
            zone_id: "zone".to_string(),
            service_id: "service".to_string(),
            group_id: String::new(),
            instance_id: id.to_string(),
        }
    }

    #[test]
    fn test_create_and_renew() {
        let manager = LeaseManager::new(Duration::from_secs(30));
        let key = create_test_key("inst-1");

        let _lease = manager.create_lease(key.clone());
        assert!(manager.is_valid(&key));

        assert!(manager.renew(&key));
    }

    #[test]
    fn test_remove_lease() {
        let manager = LeaseManager::new(Duration::from_secs(30));
        let key = create_test_key("inst-1");

        manager.create_lease(key.clone());
        assert!(manager.remove_lease(&key).is_some());
        assert!(!manager.is_valid(&key));
    }

    #[tokio::test]
    async fn test_lease_expiration() {
        let manager = LeaseManager::new(Duration::from_millis(100));
        let key = create_test_key("inst-1");

        manager.create_lease(key.clone());
        assert!(manager.is_valid(&key));

        tokio::time::sleep(Duration::from_millis(150)).await;
        assert!(!manager.is_valid(&key));

        let expired = manager.get_expired_keys();
        assert_eq!(expired.len(), 1);
    }
}
