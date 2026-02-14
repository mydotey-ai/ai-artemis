use artemis_core::model::Service;
use dashmap::DashMap;
use parking_lot::RwLock;
use std::sync::Arc;

/// 服务数据的版本化缓存
#[derive(Clone)]
pub struct VersionedCacheManager {
    /// 服务缓存: service_id -> Service
    cache: Arc<DashMap<String, Service>>,
    /// 全局版本号
    version: Arc<RwLock<i64>>,
}

impl VersionedCacheManager {
    pub fn new() -> Self {
        Self { cache: Arc::new(DashMap::new()), version: Arc::new(RwLock::new(0)) }
    }

    /// 更新服务缓存并递增版本
    pub fn update_service(&self, service: Service) {
        let service_id = service.service_id.clone().to_lowercase();
        self.cache.insert(service_id, service);
        self.increment_version();
    }

    /// 删除服务
    pub fn remove_service(&self, service_id: &str) {
        self.cache.remove(&service_id.to_lowercase());
        self.increment_version();
    }

    /// 获取服务
    pub fn get_service(&self, service_id: &str) -> Option<Service> {
        self.cache.get(&service_id.to_lowercase()).map(|entry| entry.value().clone())
    }

    /// 获取所有服务
    pub fn get_all_services(&self) -> Vec<Service> {
        self.cache.iter().map(|entry| entry.value().clone()).collect()
    }

    /// 获取当前版本号
    pub fn get_version(&self) -> i64 {
        *self.version.read()
    }

    /// 递增版本号
    fn increment_version(&self) {
        let mut version = self.version.write();
        *version += 1;
    }

    /// 清空缓存
    pub fn clear(&self) {
        self.cache.clear();
        self.increment_version();
    }
}

impl Default for VersionedCacheManager {
    fn default() -> Self {
        Self::new()
    }
}

// Task 3.8: 增量差异计算
use artemis_core::model::{ChangeType, Instance, InstanceChange};
use chrono::Utc;
use std::collections::{HashMap, HashSet};

impl VersionedCacheManager {
    pub fn compute_delta(
        old_services: &[Service],
        new_services: &[Service],
    ) -> HashMap<String, Vec<InstanceChange>> {
        let mut delta: HashMap<String, Vec<InstanceChange>> = HashMap::new();

        let old_map: HashMap<String, &Service> =
            old_services.iter().map(|s| (s.service_id.clone(), s)).collect();
        let new_map: HashMap<String, &Service> =
            new_services.iter().map(|s| (s.service_id.clone(), s)).collect();

        let mut all_service_ids: HashSet<String> = HashSet::new();
        all_service_ids.extend(old_map.keys().cloned());
        all_service_ids.extend(new_map.keys().cloned());

        for service_id in all_service_ids {
            let old_instances = old_map.get(&service_id).map(|s| &s.instances[..]).unwrap_or(&[]);
            let new_instances = new_map.get(&service_id).map(|s| &s.instances[..]).unwrap_or(&[]);
            let changes = Self::compute_instance_changes(old_instances, new_instances);

            if !changes.is_empty() {
                delta.insert(service_id, changes);
            }
        }

        delta
    }

    fn compute_instance_changes(
        old_instances: &[Instance],
        new_instances: &[Instance],
    ) -> Vec<InstanceChange> {
        let mut changes = Vec::new();
        let now = Utc::now();

        let old_map: HashMap<String, &Instance> =
            old_instances.iter().map(|inst| (inst.instance_id.clone(), inst)).collect();
        let new_map: HashMap<String, &Instance> =
            new_instances.iter().map(|inst| (inst.instance_id.clone(), inst)).collect();

        for (instance_id, new_inst) in &new_map {
            match old_map.get(instance_id) {
                Some(old_inst) => {
                    if **old_inst != **new_inst {
                        changes.push(InstanceChange {
                            instance: (*new_inst).clone(),
                            change_type: ChangeType::Change,
                            change_time: now,
                        });
                    }
                }
                None => {
                    changes.push(InstanceChange {
                        instance: (*new_inst).clone(),
                        change_type: ChangeType::New,
                        change_time: now,
                    });
                }
            }
        }

        for (instance_id, old_inst) in &old_map {
            if !new_map.contains_key(instance_id) {
                changes.push(InstanceChange {
                    instance: (*old_inst).clone(),
                    change_type: ChangeType::Delete,
                    change_time: now,
                });
            }
        }

        changes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_service(id: &str) -> Service {
        Service {
            service_id: id.to_string(),
            metadata: None,
            instances: vec![],
            logic_instances: None,
            route_rules: None,
        }
    }

    #[test]
    fn test_update_and_get() {
        let manager = VersionedCacheManager::new();
        let service = create_test_service("my-service");

        let initial_version = manager.get_version();
        manager.update_service(service.clone());

        assert_eq!(manager.get_version(), initial_version + 1);
        assert!(manager.get_service("my-service").is_some());
    }

    #[test]
    fn test_version_increment() {
        let manager = VersionedCacheManager::new();
        let v0 = manager.get_version();

        manager.update_service(create_test_service("service-1"));
        assert_eq!(manager.get_version(), v0 + 1);

        manager.update_service(create_test_service("service-2"));
        assert_eq!(manager.get_version(), v0 + 2);

        manager.remove_service("service-1");
        assert_eq!(manager.get_version(), v0 + 3);
    }

    #[test]
    fn test_get_all_services() {
        let manager = VersionedCacheManager::new();
        manager.update_service(create_test_service("service-1"));
        manager.update_service(create_test_service("service-2"));

        let services = manager.get_all_services();
        assert_eq!(services.len(), 2);
    }
}
