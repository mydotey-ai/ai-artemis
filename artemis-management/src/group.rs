//! Service group management
//!
//! This module provides comprehensive group management:
//! - Group CRUD operations
//! - Group tagging
//! - Group-instance associations
//! - Operation history tracking

use artemis_core::model::{GroupOperation, GroupTag, ServiceGroup};
use dashmap::DashMap;
use std::sync::Arc;
use tracing::info;

/// 服务分组管理器
#[derive(Clone)]
pub struct GroupManager {
    /// 分组映射: group_key -> ServiceGroup
    /// group_key = service_id:region_id:zone_id:name
    groups: Arc<DashMap<String, ServiceGroup>>,

    /// 分组 ID 映射: group_id -> group_key
    group_id_map: Arc<DashMap<i64, String>>,

    /// 分组标签: (group_id, tag_key) -> GroupTag
    tags: Arc<DashMap<(i64, String), GroupTag>>,

    /// 分组实例关联: (group_id, instance_id) -> ()
    group_instances: Arc<DashMap<(i64, String), ()>>,

    /// 操作历史: operation_id -> GroupOperation
    operations: Arc<DashMap<i64, GroupOperation>>,

    /// 下一个分组 ID
    next_group_id: Arc<DashMap<(), i64>>,

    /// 下一个操作 ID
    next_operation_id: Arc<DashMap<(), i64>>,
}

impl GroupManager {
    pub fn new() -> Self {
        let next_group_id = Arc::new(DashMap::new());
        next_group_id.insert((), 1);

        let next_operation_id = Arc::new(DashMap::new());
        next_operation_id.insert((), 1);

        Self {
            groups: Arc::new(DashMap::new()),
            group_id_map: Arc::new(DashMap::new()),
            tags: Arc::new(DashMap::new()),
            group_instances: Arc::new(DashMap::new()),
            operations: Arc::new(DashMap::new()),
            next_group_id,
            next_operation_id,
        }
    }

    /// 生成新的分组 ID
    fn allocate_group_id(&self) -> i64 {
        let mut entry = self.next_group_id.get_mut(&()).unwrap();
        let id = *entry;
        *entry += 1;
        id
    }

    /// 生成新的操作 ID
    fn allocate_operation_id(&self) -> i64 {
        let mut entry = self.next_operation_id.get_mut(&()).unwrap();
        let id = *entry;
        *entry += 1;
        id
    }

    // === 分组 CRUD ===

    pub fn create_group(&self, mut group: ServiceGroup) -> Result<(), String> {
        let group_key = group.group_key();

        if self.groups.contains_key(&group_key) {
            return Err(format!("Group already exists: {}", group_key));
        }

        // 分配分组 ID
        let group_id = self.allocate_group_id();
        group.group_id = Some(group_id);

        // 设置创建时间
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        group.created_at = Some(now);
        group.updated_at = Some(now);

        info!("Creating group: {} (ID: {})", group_key, group_id);
        self.group_id_map.insert(group_id, group_key.clone());
        self.groups.insert(group_key, group);
        Ok(())
    }

    pub fn get_group(&self, group_key: &str) -> Option<ServiceGroup> {
        self.groups.get(group_key).map(|entry| entry.value().clone())
    }

    pub fn get_group_by_id(&self, group_id: i64) -> Option<ServiceGroup> {
        let group_key = self.group_id_map.get(&group_id)?.value().clone();
        self.get_group(&group_key)
    }

    pub fn update_group(&self, group: ServiceGroup) -> Result<(), String> {
        let group_key = group.group_key();

        if !self.groups.contains_key(&group_key) {
            return Err(format!("Group not found: {}", group_key));
        }

        // 更新时间
        let mut updated_group = group;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        updated_group.updated_at = Some(now);

        info!("Updating group: {}", group_key);
        self.groups.insert(group_key, updated_group);
        Ok(())
    }

    pub fn delete_group(&self, group_key: &str) -> Result<(), String> {
        // Extract group_id first and drop the reference
        let group_id = {
            let group = self
                .groups
                .get(group_key)
                .ok_or_else(|| format!("Group not found: {}", group_key))?;

            group
                .value()
                .group_id
                .ok_or_else(|| "Group has no ID".to_string())?
        }; // group reference dropped here

        info!("Deleting group: {} (ID: {})", group_key, group_id);

        // 删除关联的标签 - collect keys first to avoid deadlock
        let tag_keys: Vec<_> = self
            .tags
            .iter()
            .filter(|entry| entry.key().0 == group_id)
            .map(|entry| entry.key().clone())
            .collect();
        for key in tag_keys {
            self.tags.remove(&key);
        }

        // 删除关联的实例 - collect keys first to avoid deadlock
        let instance_keys: Vec<_> = self
            .group_instances
            .iter()
            .filter(|entry| entry.key().0 == group_id)
            .map(|entry| entry.key().clone())
            .collect();
        for key in instance_keys {
            self.group_instances.remove(&key);
        }

        // 删除分组 ID 映射
        self.group_id_map.remove(&group_id);

        // 删除分组
        self.groups.remove(group_key);
        Ok(())
    }

    pub fn list_groups(&self) -> Vec<ServiceGroup> {
        self.groups
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    pub fn list_groups_by_service(&self, service_id: &str) -> Vec<ServiceGroup> {
        self.groups
            .iter()
            .filter(|entry| entry.value().service_id == service_id)
            .map(|entry| entry.value().clone())
            .collect()
    }

    pub fn list_groups_by_region(&self, region_id: &str) -> Vec<ServiceGroup> {
        self.groups
            .iter()
            .filter(|entry| entry.value().region_id == region_id)
            .map(|entry| entry.value().clone())
            .collect()
    }

    // === 标签管理 ===

    pub fn add_tag(&self, group_id: i64, tag: GroupTag) -> Result<(), String> {
        if !self.group_id_map.contains_key(&group_id) {
            return Err(format!("Group not found: {}", group_id));
        }

        let key = (group_id, tag.key.clone());
        self.tags.insert(key, tag);
        Ok(())
    }

    pub fn remove_tag(&self, group_id: i64, tag_key: &str) -> Result<(), String> {
        let key = (group_id, tag_key.to_string());
        if self.tags.remove(&key).is_none() {
            return Err(format!("Tag not found: {}:{}", group_id, tag_key));
        }
        Ok(())
    }

    pub fn get_tags(&self, group_id: i64) -> Vec<GroupTag> {
        self.tags
            .iter()
            .filter(|entry| entry.key().0 == group_id)
            .map(|entry| entry.value().clone())
            .collect()
    }

    pub fn find_groups_by_tag(&self, tag_key: &str, tag_value: &str) -> Vec<ServiceGroup> {
        let group_ids: Vec<i64> = self
            .tags
            .iter()
            .filter(|entry| {
                let tag = entry.value();
                tag.key == tag_key && tag.value == tag_value
            })
            .map(|entry| entry.key().0)
            .collect();

        group_ids
            .into_iter()
            .filter_map(|gid| self.get_group_by_id(gid))
            .collect()
    }

    // === 实例管理 ===

    pub fn add_instance(&self, group_id: i64, instance_id: &str) -> Result<(), String> {
        if !self.group_id_map.contains_key(&group_id) {
            return Err(format!("Group not found: {}", group_id));
        }

        let key = (group_id, instance_id.to_string());
        self.group_instances.insert(key, ());
        Ok(())
    }

    pub fn remove_instance(&self, group_id: i64, instance_id: &str) -> Result<(), String> {
        let key = (group_id, instance_id.to_string());
        if self.group_instances.remove(&key).is_none() {
            return Err(format!(
                "Instance {} not in group {}",
                instance_id, group_id
            ));
        }
        Ok(())
    }

    pub fn get_instances(&self, group_id: i64) -> Vec<String> {
        self.group_instances
            .iter()
            .filter(|entry| entry.key().0 == group_id)
            .map(|entry| entry.key().1.clone())
            .collect()
    }

    pub fn get_instance_groups(&self, instance_id: &str) -> Vec<i64> {
        self.group_instances
            .iter()
            .filter(|entry| entry.key().1 == instance_id)
            .map(|entry| entry.key().0)
            .collect()
    }

    // === 操作记录 ===

    pub fn record_operation(&self, mut operation: GroupOperation) {
        let operation_id = self.allocate_operation_id();
        operation.operation_id = Some(operation_id);
        self.operations.insert(operation_id, operation);
    }

    pub fn get_operations(&self, group_id: i64) -> Vec<GroupOperation> {
        self.operations
            .iter()
            .filter(|entry| entry.value().group_id == group_id)
            .map(|entry| entry.value().clone())
            .collect()
    }

    // === 辅助方法 ===

    pub fn group_count(&self) -> usize {
        self.groups.len()
    }

    pub fn group_exists(&self, group_key: &str) -> bool {
        self.groups.contains_key(group_key)
    }
}

impl Default for GroupManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use artemis_core::model::{GroupStatus, GroupType};

    fn create_test_group(name: &str) -> ServiceGroup {
        ServiceGroup {
            group_id: None,
            service_id: "test-service".to_string(),
            region_id: "us-east".to_string(),
            zone_id: "zone-1".to_string(),
            name: name.to_string(),
            group_type: GroupType::Physical,
            status: GroupStatus::Active,
            description: Some("Test group".to_string()),
            tags: None,
            metadata: None,
            created_at: None,
            updated_at: None,
        }
    }

    #[test]
    fn test_create_and_get_group() {
        let manager = GroupManager::new();
        let group = create_test_group("group-1");

        assert!(manager.create_group(group.clone()).is_ok());

        let group_key = group.group_key();
        let retrieved = manager.get_group(&group_key).unwrap();
        assert_eq!(retrieved.name, "group-1");
        assert_eq!(retrieved.service_id, "test-service");
        assert!(retrieved.group_id.is_some());
        assert_eq!(retrieved.group_id.unwrap(), 1);
    }

    #[test]
    fn test_create_duplicate_group() {
        let manager = GroupManager::new();
        let group = create_test_group("group-1");

        assert!(manager.create_group(group.clone()).is_ok());
        assert!(manager.create_group(group).is_err());
    }

    #[test]
    fn test_update_group() {
        let manager = GroupManager::new();
        let group = create_test_group("group-1");

        manager.create_group(group.clone()).unwrap();

        let mut updated_group = manager.get_group(&group.group_key()).unwrap();
        updated_group.status = GroupStatus::Inactive;
        assert!(manager.update_group(updated_group).is_ok());

        let retrieved = manager.get_group(&group.group_key()).unwrap();
        assert!(matches!(retrieved.status, GroupStatus::Inactive));
    }

    #[test]
    fn test_delete_group() {
        let manager = GroupManager::new();
        let group = create_test_group("group-1");

        manager.create_group(group.clone()).unwrap();
        assert_eq!(manager.group_count(), 1);

        let group_key = group.group_key();
        assert!(manager.delete_group(&group_key).is_ok());
        assert_eq!(manager.group_count(), 0);
    }

    #[test]
    fn test_list_groups_by_service() {
        let manager = GroupManager::new();

        let mut group1 = create_test_group("group-1");
        group1.service_id = "service-a".to_string();

        let mut group2 = create_test_group("group-2");
        group2.service_id = "service-b".to_string();

        manager.create_group(group1).unwrap();
        manager.create_group(group2).unwrap();

        let groups = manager.list_groups_by_service("service-a");
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].name, "group-1");
    }

    #[test]
    fn test_add_and_get_tags() {
        let manager = GroupManager::new();
        let group = create_test_group("group-1");
        manager.create_group(group.clone()).unwrap();

        let group_id = manager.get_group(&group.group_key()).unwrap().group_id.unwrap();

        let tag = GroupTag {
            key: "env".to_string(),
            value: "production".to_string(),
        };

        assert!(manager.add_tag(group_id, tag).is_ok());

        let tags = manager.get_tags(group_id);
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].key, "env");
    }

    #[test]
    fn test_find_groups_by_tag() {
        let manager = GroupManager::new();

        manager.create_group(create_test_group("group-1")).unwrap();
        manager.create_group(create_test_group("group-2")).unwrap();

        let group1 = manager.get_group(&create_test_group("group-1").group_key()).unwrap();
        let group_id1 = group1.group_id.unwrap();

        manager
            .add_tag(
                group_id1,
                GroupTag {
                    key: "env".to_string(),
                    value: "prod".to_string(),
                },
            )
            .unwrap();

        let groups = manager.find_groups_by_tag("env", "prod");
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].name, "group-1");
    }

    #[test]
    fn test_add_and_get_instances() {
        let manager = GroupManager::new();
        manager.create_group(create_test_group("group-1")).unwrap();

        let group = manager.get_group(&create_test_group("group-1").group_key()).unwrap();
        let group_id = group.group_id.unwrap();

        assert!(manager.add_instance(group_id, "inst-1").is_ok());
        assert!(manager.add_instance(group_id, "inst-2").is_ok());

        let instances = manager.get_instances(group_id);
        assert_eq!(instances.len(), 2);
        assert!(instances.contains(&"inst-1".to_string()));
    }

    #[test]
    fn test_get_instance_groups() {
        let manager = GroupManager::new();
        manager.create_group(create_test_group("group-1")).unwrap();
        manager.create_group(create_test_group("group-2")).unwrap();

        let group1 = manager.get_group(&create_test_group("group-1").group_key()).unwrap();
        let group2 = manager.get_group(&create_test_group("group-2").group_key()).unwrap();
        let group_id1 = group1.group_id.unwrap();
        let group_id2 = group2.group_id.unwrap();

        manager.add_instance(group_id1, "inst-1").unwrap();
        manager.add_instance(group_id2, "inst-1").unwrap();

        let groups = manager.get_instance_groups("inst-1");
        assert_eq!(groups.len(), 2);
    }

    #[test]
    fn test_delete_group_cascades() {
        let manager = GroupManager::new();
        manager.create_group(create_test_group("group-1")).unwrap();

        let group = manager.get_group(&create_test_group("group-1").group_key()).unwrap();
        let group_id = group.group_id.unwrap();

        manager
            .add_tag(
                group_id,
                GroupTag {
                    key: "env".to_string(),
                    value: "prod".to_string(),
                },
            )
            .unwrap();

        manager.add_instance(group_id, "inst-1").unwrap();

        let group_key = group.group_key();
        manager.delete_group(&group_key).unwrap();

        assert_eq!(manager.get_tags(group_id).len(), 0);
        assert_eq!(manager.get_instances(group_id).len(), 0);
    }

    #[test]
    fn test_record_operations() {
        let manager = GroupManager::new();

        let operation = GroupOperation {
            operation_id: None,
            group_id: 1,
            operation_type: "CREATE".to_string(),
            operator_id: "admin".to_string(),
            description: Some("Created group".to_string()),
            timestamp: 1234567890,
        };

        manager.record_operation(operation);

        let ops = manager.get_operations(1);
        assert_eq!(ops.len(), 1);
        assert_eq!(ops[0].operation_type, "CREATE");
        assert!(ops[0].operation_id.is_some());
    }
}
