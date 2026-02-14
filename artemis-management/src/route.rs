//! Route rule management framework (Phase 11)
//!
//! This module provides route rule management:
//! - Route rule CRUD operations
//! - Rule priority management
//! - Target configuration
//!
//! Status: Framework only, full implementation pending

use artemis_core::model::RouteRule;
use dashmap::DashMap;
use std::sync::Arc;
use tracing::info;

/// 路由规则管理器框架
#[derive(Clone)]
pub struct RouteManager {
    /// 路由规则映射: RuleId -> RouteRule
    rules: Arc<DashMap<String, RouteRule>>,
}

impl RouteManager {
    pub fn new() -> Self {
        Self { rules: Arc::new(DashMap::new()) }
    }

    /// 创建路由规则
    pub fn create_rule(&self, rule: RouteRule) {
        info!("Creating route rule: {}", rule.route_id);
        self.rules.insert(rule.route_id.clone(), rule);
    }

    /// 获取路由规则
    pub fn get_rule(&self, rule_id: &str) -> Option<RouteRule> {
        self.rules.get(rule_id).map(|entry| entry.value().clone())
    }

    /// 更新路由规则
    pub fn update_rule(&self, rule: RouteRule) -> bool {
        if self.rules.contains_key(&rule.route_id) {
            self.rules.insert(rule.route_id.clone(), rule);
            true
        } else {
            false
        }
    }

    /// 删除路由规则
    pub fn delete_rule(&self, rule_id: &str) -> bool {
        self.rules.remove(rule_id).is_some()
    }

    /// 列出所有路由规则（框架方法，按服务过滤待实现）
    pub fn get_rules_by_service(&self, _service_id: &str) -> Vec<RouteRule> {
        // TODO: 需要在RouteRule中添加service_id字段或其他关联方式
        vec![]
    }

    /// 列出所有路由规则
    pub fn list_rules(&self) -> Vec<RouteRule> {
        self.rules.iter().map(|entry| entry.value().clone()).collect()
    }

    /// 获取规则数量
    pub fn rule_count(&self) -> usize {
        self.rules.len()
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

    fn create_test_rule(rule_id: &str) -> RouteRule {
        RouteRule {
            route_rule_id: None,
            route_id: rule_id.to_string(),
            service_id: "test-service".to_string(),
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
        let rule = create_test_rule("rule-1");

        manager.create_rule(rule.clone());
        let retrieved = manager.get_rule("rule-1").unwrap();

        assert_eq!(retrieved.route_id, "rule-1");
    }

    #[test]
    fn test_update_rule() {
        let manager = RouteManager::new();
        let rule = create_test_rule("rule-1");

        manager.create_rule(rule.clone());

        let mut updated = rule.clone();
        updated.strategy = RouteStrategy::CloseByVisit;

        assert!(manager.update_rule(updated));
        let retrieved = manager.get_rule("rule-1").unwrap();
        assert!(matches!(retrieved.strategy, RouteStrategy::CloseByVisit));
    }

    #[test]
    fn test_delete_rule() {
        let manager = RouteManager::new();
        let rule = create_test_rule("rule-1");

        manager.create_rule(rule);
        assert_eq!(manager.rule_count(), 1);

        assert!(manager.delete_rule("rule-1"));
        assert_eq!(manager.rule_count(), 0);
    }
}
