//! Service group management framework (Phase 11)
//!
//! This module provides group management capabilities:
//! - Group CRUD operations
//! - Group-based service discovery
//! - Weight routing
//!
//! Status: Framework only, full implementation pending

use artemis_core::model::RouteRule;
use dashmap::DashMap;
use std::sync::Arc;
use tracing::info;

/// 服务分组管理器框架
#[derive(Clone)]
pub struct GroupManager {
    /// 分组映射: GroupId -> RouteRule
    groups: Arc<DashMap<String, RouteRule>>,
}

impl GroupManager {
    pub fn new() -> Self {
        Self {
            groups: Arc::new(DashMap::new()),
        }
    }

    /// 创建分组
    pub fn create_group(&self, group: RouteRule) {
        info!("Creating group: {}", group.route_id);
        self.groups.insert(group.route_id.clone(), group);
    }

    /// 获取分组
    pub fn get_group(&self, group_id: &str) -> Option<RouteRule> {
        self.groups.get(group_id).map(|entry| entry.value().clone())
    }

    /// 更新分组
    pub fn update_group(&self, group: RouteRule) -> bool {
        if self.groups.contains_key(&group.route_id) {
            self.groups.insert(group.route_id.clone(), group);
            true
        } else {
            false
        }
    }

    /// 删除分组
    pub fn delete_group(&self, group_id: &str) -> bool {
        self.groups.remove(group_id).is_some()
    }

    /// 列出所有分组
    pub fn list_groups(&self) -> Vec<RouteRule> {
        self.groups
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// 获取分组数量
    pub fn group_count(&self) -> usize {
        self.groups.len()
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
    use artemis_core::model::RouteStrategy;

    fn create_test_rule() -> RouteRule {
        RouteRule {
            route_id: "rule-1".to_string(),
            strategy: RouteStrategy::WeightedRoundRobin,
            groups: vec![],
        }
    }

    #[test]
    fn test_create_and_get_group() {
        let manager = GroupManager::new();
        let rule = create_test_rule();

        manager.create_group(rule.clone());
        let retrieved = manager.get_group("rule-1").unwrap();

        assert_eq!(retrieved.route_id, "rule-1");
    }

    #[test]
    fn test_update_group() {
        let manager = GroupManager::new();
        let rule = create_test_rule();

        manager.create_group(rule.clone());

        let mut updated_rule = rule.clone();
        updated_rule.strategy = RouteStrategy::CloseByVisit;

        assert!(manager.update_group(updated_rule));
        let retrieved = manager.get_group("rule-1").unwrap();
        assert!(matches!(retrieved.strategy, RouteStrategy::CloseByVisit));
    }

    #[test]
    fn test_delete_group() {
        let manager = GroupManager::new();
        let rule = create_test_rule();

        manager.create_group(rule);
        assert_eq!(manager.group_count(), 1);

        assert!(manager.delete_group("rule-1"));
        assert_eq!(manager.group_count(), 0);
    }
}
