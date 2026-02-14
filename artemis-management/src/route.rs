//! Route rule management
//!
//! This module provides comprehensive route rule management:
//! - Route rule CRUD operations
//! - Rule-group associations
//! - Rule publishing/unpublishing

use artemis_core::model::{RouteRule, RouteRuleGroup, RouteRuleStatus};
use dashmap::DashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};
use tracing::info;
use crate::db::Database;
use crate::dao::RouteRuleDao;

/// 路由规则管理器
#[derive(Clone)]
pub struct RouteManager {
    /// 路由规则映射: route_id -> RouteRule
    rules: Arc<DashMap<String, RouteRule>>,

    /// 数字ID映射: route_rule_id -> route_id
    rule_id_map: Arc<DashMap<i64, String>>,

    /// 路由规则分组: (route_id, group_id) -> RouteRuleGroup
    rule_groups: Arc<DashMap<(String, String), RouteRuleGroup>>,

    /// ID 生成器
    next_id: Arc<AtomicI64>,

    /// 可选数据库支持 - 用于持久化
    database: Option<Arc<Database>>,
}

impl RouteManager {
    pub fn new() -> Self {
        Self::with_database(None)
    }

    pub fn with_database(database: Option<Arc<Database>>) -> Self {
        Self {
            rules: Arc::new(DashMap::new()),
            rule_id_map: Arc::new(DashMap::new()),
            rule_groups: Arc::new(DashMap::new()),
            next_id: Arc::new(AtomicI64::new(1)),
            database,
        }
    }

    // === 路由规则 CRUD ===

    pub fn create_rule(&self, mut rule: RouteRule) -> Result<(), String> {
        if self.rules.contains_key(&rule.route_id) {
            return Err(format!("Route rule {} already exists", rule.route_id));
        }

        // 自动分配数字ID
        if rule.route_rule_id.is_none() {
            let id = self.next_id.fetch_add(1, Ordering::SeqCst);
            rule.route_rule_id = Some(id);
            self.rule_id_map.insert(id, rule.route_id.clone());
        }

        info!("Creating route rule: {}", rule.route_id);
        self.rules.insert(rule.route_id.clone(), rule.clone());

        // 持久化到数据库
        if let Some(db) = &self.database {
            let dao = RouteRuleDao::new(db.pool().clone());
            let rule_clone = rule.clone();
            tokio::spawn(async move {
                if let Err(e) = dao.insert_rule(&rule_clone).await {
                    tracing::error!("Failed to persist route rule to database: {}", e);
                }
            });
        }

        Ok(())
    }

    pub fn get_rule(&self, rule_id: &str) -> Option<RouteRule> {
        self.rules.get(rule_id).map(|entry| entry.value().clone())
    }

    pub fn get_rule_by_id(&self, route_rule_id: i64) -> Option<RouteRule> {
        let route_id = self.rule_id_map.get(&route_rule_id)?.value().clone();
        self.get_rule(&route_id)
    }

    pub fn update_rule(&self, rule: RouteRule) -> Result<(), String> {
        if !self.rules.contains_key(&rule.route_id) {
            return Err(format!("Route rule {} not found", rule.route_id));
        }

        info!("Updating route rule: {}", rule.route_id);
        self.rules.insert(rule.route_id.clone(), rule.clone());

        // 持久化到数据库
        if let Some(db) = &self.database {
            let dao = RouteRuleDao::new(db.pool().clone());
            let rule_clone = rule.clone();
            tokio::spawn(async move {
                if let Err(e) = dao.update_rule(&rule_clone).await {
                    tracing::error!("Failed to update route rule in database: {}", e);
                }
            });
        }

        Ok(())
    }

    pub fn delete_rule(&self, rule_id: &str) -> Result<(), String> {
        if !self.rules.contains_key(rule_id) {
            return Err(format!("Route rule {} not found", rule_id));
        }

        info!("Deleting route rule: {}", rule_id);

        // 删除关联的分组
        self.rule_groups.retain(|(rid, _), _| rid != rule_id);

        // 删除ID映射
        if let Some(rule) = self.rules.get(rule_id)
            && let Some(id) = rule.route_rule_id
        {
            self.rule_id_map.remove(&id);
        }

        self.rules.remove(rule_id);

        // 从数据库删除
        if let Some(db) = &self.database {
            let dao = RouteRuleDao::new(db.pool().clone());
            let rule_id_owned = rule_id.to_string();
            tokio::spawn(async move {
                if let Err(e) = dao.delete_rule(&rule_id_owned).await {
                    tracing::error!("Failed to delete route rule from database: {}", e);
                }
            });
        }

        Ok(())
    }

    pub fn list_rules(&self) -> Vec<RouteRule> {
        self.rules.iter().map(|entry| entry.value().clone()).collect()
    }

    pub fn get_rules_by_service(&self, service_id: &str) -> Vec<RouteRule> {
        self.rules
            .iter()
            .filter(|entry| entry.value().service_id == service_id)
            .map(|entry| entry.value().clone())
            .collect()
    }

    // === 路由规则分组关联 ===

    pub fn add_rule_group(&self, rule_id: &str, group: RouteRuleGroup) -> Result<(), String> {
        if !self.rules.contains_key(rule_id) {
            return Err(format!("Route rule {} not found", rule_id));
        }

        let key = (rule_id.to_string(), group.group_id.clone());
        self.rule_groups.insert(key, group);
        Ok(())
    }

    pub fn remove_rule_group(&self, rule_id: &str, group_id: &str) -> Result<(), String> {
        let key = (rule_id.to_string(), group_id.to_string());
        if self.rule_groups.remove(&key).is_none() {
            return Err(format!("Group {} not found in rule {}", group_id, rule_id));
        }
        Ok(())
    }

    pub fn get_rule_groups(&self, rule_id: &str) -> Vec<RouteRuleGroup> {
        self.rule_groups
            .iter()
            .filter(|entry| entry.key().0 == rule_id)
            .map(|entry| entry.value().clone())
            .collect()
    }

    pub fn update_rule_group(&self, rule_id: &str, group: RouteRuleGroup) -> Result<(), String> {
        let key = (rule_id.to_string(), group.group_id.clone());

        if !self.rule_groups.contains_key(&key) {
            return Err(format!("Group {} not found in rule {}", group.group_id, rule_id));
        }

        self.rule_groups.insert(key, group);
        Ok(())
    }

    // === 规则发布管理 ===

    pub fn publish_rule(&self, rule_id: &str) -> Result<(), String> {
        let mut rule = self.get_rule(rule_id)
            .ok_or_else(|| format!("Route rule {} not found", rule_id))?;

        rule.status = RouteRuleStatus::Active;
        self.rules.insert(rule_id.to_string(), rule);

        info!("Published route rule: {}", rule_id);
        Ok(())
    }

    pub fn unpublish_rule(&self, rule_id: &str) -> Result<(), String> {
        let mut rule = self.get_rule(rule_id)
            .ok_or_else(|| format!("Route rule {} not found", rule_id))?;

        rule.status = RouteRuleStatus::Inactive;
        self.rules.insert(rule_id.to_string(), rule);

        info!("Unpublished route rule: {}", rule_id);
        Ok(())
    }

    pub fn get_active_rules(&self, service_id: &str) -> Vec<RouteRule> {
        self.rules
            .iter()
            .filter(|entry| {
                let rule = entry.value();
                rule.service_id == service_id && matches!(rule.status, RouteRuleStatus::Active)
            })
            .map(|entry| entry.value().clone())
            .collect()
    }

    // === 辅助方法 ===

    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }

    pub fn rule_exists(&self, rule_id: &str) -> bool {
        self.rules.contains_key(rule_id)
    }
}

impl Default for RouteManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use artemis_core::model::{RouteRuleStatus, RouteStrategy};

    fn create_test_rule(rule_id: &str, service_id: &str) -> RouteRule {
        RouteRule {
            route_rule_id: None,
            route_id: rule_id.to_string(),
            service_id: service_id.to_string(),
            name: rule_id.to_string(),
            description: None,
            status: RouteRuleStatus::Active,
            strategy: RouteStrategy::WeightedRoundRobin,
            groups: vec![],
        }
    }

    #[test]
    fn test_create_and_get_rule() {
        let manager = RouteManager::new();
        let rule = create_test_rule("rule-1", "service-a");

        assert!(manager.create_rule(rule.clone()).is_ok());

        let retrieved = manager.get_rule("rule-1").unwrap();
        assert_eq!(retrieved.route_id, "rule-1");
        assert!(retrieved.route_rule_id.is_some());
    }

    #[test]
    fn test_create_duplicate_rule() {
        let manager = RouteManager::new();
        let rule = create_test_rule("rule-1", "service-a");

        assert!(manager.create_rule(rule.clone()).is_ok());
        assert!(manager.create_rule(rule).is_err());
    }

    #[test]
    fn test_get_rule_by_id() {
        let manager = RouteManager::new();
        let rule = create_test_rule("rule-1", "service-a");

        manager.create_rule(rule).unwrap();

        let retrieved_by_string = manager.get_rule("rule-1").unwrap();
        let id = retrieved_by_string.route_rule_id.unwrap();

        let retrieved_by_id = manager.get_rule_by_id(id).unwrap();
        assert_eq!(retrieved_by_id.route_id, "rule-1");
    }

    #[test]
    fn test_update_rule() {
        let manager = RouteManager::new();
        let mut rule = create_test_rule("rule-1", "service-a");

        manager.create_rule(rule.clone()).unwrap();

        rule.strategy = RouteStrategy::CloseByVisit;
        assert!(manager.update_rule(rule).is_ok());

        let retrieved = manager.get_rule("rule-1").unwrap();
        assert!(matches!(retrieved.strategy, RouteStrategy::CloseByVisit));
    }

    #[test]
    fn test_delete_rule() {
        let manager = RouteManager::new();
        let rule = create_test_rule("rule-1", "service-a");

        manager.create_rule(rule).unwrap();
        assert_eq!(manager.rule_count(), 1);

        assert!(manager.delete_rule("rule-1").is_ok());
        assert_eq!(manager.rule_count(), 0);
    }

    #[test]
    fn test_get_rules_by_service() {
        let manager = RouteManager::new();

        manager.create_rule(create_test_rule("rule-1", "service-a")).unwrap();
        manager.create_rule(create_test_rule("rule-2", "service-b")).unwrap();
        manager.create_rule(create_test_rule("rule-3", "service-a")).unwrap();

        let rules = manager.get_rules_by_service("service-a");
        assert_eq!(rules.len(), 2);
    }

    #[test]
    fn test_add_and_get_rule_groups() {
        let manager = RouteManager::new();
        manager.create_rule(create_test_rule("rule-1", "service-a")).unwrap();

        let group = RouteRuleGroup::new("rule-1".to_string(), "group-1".to_string(), 50);
        assert!(manager.add_rule_group("rule-1", group).is_ok());

        let groups = manager.get_rule_groups("rule-1");
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].group_id, "group-1");
    }

    #[test]
    fn test_update_rule_group() {
        let manager = RouteManager::new();
        manager.create_rule(create_test_rule("rule-1", "service-a")).unwrap();

        let group = RouteRuleGroup::new("rule-1".to_string(), "group-1".to_string(), 50);
        manager.add_rule_group("rule-1", group).unwrap();

        let updated_group = RouteRuleGroup::new("rule-1".to_string(), "group-1".to_string(), 80);
        assert!(manager.update_rule_group("rule-1", updated_group).is_ok());

        let groups = manager.get_rule_groups("rule-1");
        assert_eq!(groups[0].weight, 80);
    }

    #[test]
    fn test_remove_rule_group() {
        let manager = RouteManager::new();
        manager.create_rule(create_test_rule("rule-1", "service-a")).unwrap();

        let group = RouteRuleGroup::new("rule-1".to_string(), "group-1".to_string(), 50);
        manager.add_rule_group("rule-1", group).unwrap();

        assert!(manager.remove_rule_group("rule-1", "group-1").is_ok());
        assert_eq!(manager.get_rule_groups("rule-1").len(), 0);
    }

    #[test]
    fn test_publish_unpublish_rule() {
        let manager = RouteManager::new();
        let mut rule = create_test_rule("rule-1", "service-a");
        rule.status = RouteRuleStatus::Inactive;

        manager.create_rule(rule).unwrap();

        // Publish
        assert!(manager.publish_rule("rule-1").is_ok());
        let rule = manager.get_rule("rule-1").unwrap();
        assert!(matches!(rule.status, RouteRuleStatus::Active));

        // Unpublish
        assert!(manager.unpublish_rule("rule-1").is_ok());
        let rule = manager.get_rule("rule-1").unwrap();
        assert!(matches!(rule.status, RouteRuleStatus::Inactive));
    }

    #[test]
    fn test_get_active_rules() {
        let manager = RouteManager::new();

        let mut rule1 = create_test_rule("rule-1", "service-a");
        rule1.status = RouteRuleStatus::Active;

        let mut rule2 = create_test_rule("rule-2", "service-a");
        rule2.status = RouteRuleStatus::Inactive;

        manager.create_rule(rule1).unwrap();
        manager.create_rule(rule2).unwrap();

        let active = manager.get_active_rules("service-a");
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].route_id, "rule-1");
    }

    #[test]
    fn test_delete_rule_cascades() {
        let manager = RouteManager::new();
        manager.create_rule(create_test_rule("rule-1", "service-a")).unwrap();

        let group = RouteRuleGroup::new("rule-1".to_string(), "group-1".to_string(), 50);
        manager.add_rule_group("rule-1", group).unwrap();

        manager.delete_rule("rule-1").unwrap();

        assert_eq!(manager.get_rule_groups("rule-1").len(), 0);
    }
}
