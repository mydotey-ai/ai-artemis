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

    // ========== 新增测试 (快速冲刺阶段 - Task 3) ==========

    #[test]
    fn test_clear_increments_version() {
        let manager = VersionedCacheManager::new();
        manager.update_service(create_test_service("service-1"));
        manager.update_service(create_test_service("service-2"));

        let version_before_clear = manager.get_version();
        manager.clear();

        assert_eq!(manager.get_all_services().len(), 0);
        assert_eq!(manager.get_version(), version_before_clear + 1);
    }

    #[test]
    fn test_service_id_case_insensitive() {
        let manager = VersionedCacheManager::new();
        manager.update_service(create_test_service("My-Service"));

        // 验证小写查询
        assert!(manager.get_service("my-service").is_some());
        assert!(manager.get_service("MY-SERVICE").is_some());
        assert!(manager.get_service("My-Service").is_some());

        // 验证删除也是大小写不敏感
        manager.remove_service("MY-SERVICE");
        assert!(manager.get_service("my-service").is_none());
    }

    #[test]
    fn test_remove_nonexistent_service() {
        let manager = VersionedCacheManager::new();
        let version_before = manager.get_version();

        manager.remove_service("nonexistent-service");

        // 删除不存在的服务仍然会递增版本
        assert_eq!(manager.get_version(), version_before + 1);
    }

    #[test]
    fn test_compute_delta_new_instances() {
        use artemis_core::model::Instance;

        let old_services = vec![create_test_service("service-1")];

        let mut new_service = create_test_service("service-1");
        new_service.instances = vec![Instance {
            region_id: "test-region".to_string(),
            zone_id: "test-zone".to_string(),
            group_id: None,
            service_id: "service-1".to_string(),
            instance_id: "inst-1".to_string(),
            machine_name: None,
            ip: "127.0.0.1".to_string(),
            port: 8080,
            protocol: None,
            url: "http://127.0.0.1:8080".to_string(),
            status: artemis_core::model::InstanceStatus::Up,
            metadata: None,
            health_check_url: None,
        }];

        let delta = VersionedCacheManager::compute_delta(&old_services, &[new_service]);

        assert_eq!(delta.len(), 1);
        assert!(delta.contains_key("service-1"));
        let changes = delta.get("service-1").unwrap();
        assert_eq!(changes.len(), 1);
        assert!(matches!(changes[0].change_type, ChangeType::New));
    }

    #[test]
    fn test_compute_delta_deleted_instances() {
        use artemis_core::model::Instance;

        let mut old_service = create_test_service("service-1");
        old_service.instances = vec![Instance {
            region_id: "test-region".to_string(),
            zone_id: "test-zone".to_string(),
            group_id: None,
            service_id: "service-1".to_string(),
            instance_id: "inst-1".to_string(),
            machine_name: None,
            ip: "127.0.0.1".to_string(),
            port: 8080,
            protocol: None,
            url: "http://127.0.0.1:8080".to_string(),
            status: artemis_core::model::InstanceStatus::Up,
            metadata: None,
            health_check_url: None,
        }];

        let new_services = vec![create_test_service("service-1")];

        let delta = VersionedCacheManager::compute_delta(&[old_service], &new_services);

        assert_eq!(delta.len(), 1);
        let changes = delta.get("service-1").unwrap();
        assert_eq!(changes.len(), 1);
        assert!(matches!(changes[0].change_type, ChangeType::Delete));
    }

    #[test]
    fn test_compute_delta_no_changes() {
        let services = vec![create_test_service("service-1")];
        let delta = VersionedCacheManager::compute_delta(&services, &services);

        // 没有变更,应该返回空 map
        assert_eq!(delta.len(), 0);
    }
}
