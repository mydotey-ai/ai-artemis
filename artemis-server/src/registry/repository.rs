use artemis_core::model::{Instance, InstanceKey, Service};
use dashmap::DashMap;
use std::sync::Arc;
use std::collections::HashMap;

/// 内存中的注册表存储（高性能无锁）
#[derive(Clone)]
pub struct RegistryRepository {
    /// Instance存储: InstanceKey -> Instance
    instances: Arc<DashMap<InstanceKey, Instance>>,
}

impl RegistryRepository {
    pub fn new() -> Self {
        Self { instances: Arc::new(DashMap::new()) }
    }

    /// 注册实例
    pub fn register(&self, instance: Instance) {
        let key = instance.key();
        self.instances.insert(key, instance);
    }

    /// 获取实例
    pub fn get_instance(&self, key: &InstanceKey) -> Option<Instance> {
        self.instances.get(key).map(|entry| entry.value().clone())
    }

    /// 删除实例
    pub fn remove(&self, key: &InstanceKey) -> Option<Instance> {
        self.instances.remove(key).map(|(_, v)| v)
    }

    /// 获取某个服务的所有实例
    pub fn get_instances_by_service(&self, service_id: &str) -> Vec<Instance> {
        let service_id_lower = service_id.to_lowercase();
        self.instances
            .iter()
            .filter(|entry| entry.key().service_id == service_id_lower)
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// 获取所有实例
    pub fn get_all_instances(&self) -> Vec<Instance> {
        self.instances.iter().map(|entry| entry.value().clone()).collect()
    }

    /// 获取实例数量
    pub fn count(&self) -> usize {
        self.instances.len()
    }

    /// 获取所有服务(按 service_id 分组)
    pub fn get_all_services(&self) -> Vec<Service> {
        // 按 service_id 分组
        let mut services_map: HashMap<String, Vec<Instance>> = HashMap::new();

        for entry in self.instances.iter() {
            let instance = entry.value().clone();
            let service_id = instance.service_id.clone();

            services_map
                .entry(service_id)
                .or_insert_with(Vec::new)
                .push(instance);
        }

        // 转换为 Service 对象
        services_map
            .into_iter()
            .map(|(service_id, instances)| {
                Service {
                    service_id,
                    metadata: None,
                    instances,
                    logic_instances: None,
                    route_rules: None,
                }
            })
            .collect()
    }
}

impl Default for RegistryRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use artemis_core::model::InstanceStatus;

    fn create_test_instance(service_id: &str, instance_id: &str) -> Instance {
        Instance {
            region_id: "test-region".to_string(),
            zone_id: "test-zone".to_string(),
            group_id: None,
            service_id: service_id.to_string(),
            instance_id: instance_id.to_string(),
            machine_name: None,
            ip: "127.0.0.1".to_string(),
            port: 8080,
            protocol: Some("http".to_string()),
            url: "http://127.0.0.1:8080".to_string(),
            health_check_url: None,
            status: InstanceStatus::Up,
            metadata: None,
        }
    }

    #[test]
    fn test_register_and_get() {
        let repo = RegistryRepository::new();
        let instance = create_test_instance("my-service", "inst-1");
        let key = instance.key();

        repo.register(instance.clone());
        let retrieved = repo.get_instance(&key).unwrap();

        assert_eq!(retrieved.instance_id, "inst-1");
    }

    #[test]
    fn test_get_instances_by_service() {
        let repo = RegistryRepository::new();
        repo.register(create_test_instance("service-a", "inst-1"));
        repo.register(create_test_instance("service-a", "inst-2"));
        repo.register(create_test_instance("service-b", "inst-3"));

        let instances = repo.get_instances_by_service("service-a");
        assert_eq!(instances.len(), 2);
    }

    #[test]
    fn test_remove() {
        let repo = RegistryRepository::new();
        let instance = create_test_instance("my-service", "inst-1");
        let key = instance.key();

        repo.register(instance);
        let removed = repo.remove(&key);

        assert!(removed.is_some());
        assert!(repo.get_instance(&key).is_none());
    }
}
