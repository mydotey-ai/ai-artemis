//! 路由策略实现

use artemis_core::model::RouteRuleGroup;
use dashmap::DashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// 路由策略 trait
pub trait RouteStrategy: Send + Sync {
    /// 根据分组列表和路由上下文选择一个分组
    fn select_group(
        &self,
        groups: &[RouteRuleGroup],
        context: &super::context::RouteContext,
    ) -> Option<String>;
}

/// 加权轮询策略
#[derive(Clone)]
pub struct WeightedRoundRobinStrategy {
    /// 每个规则的计数器: route_rule_id -> AtomicUsize
    counters: Arc<DashMap<String, Arc<AtomicUsize>>>,
}

impl WeightedRoundRobinStrategy {
    pub fn new() -> Self {
        Self {
            counters: Arc::new(DashMap::new()),
        }
    }

    /// 获取或创建计数器
    fn get_counter(&self, route_rule_id: &str) -> Arc<AtomicUsize> {
        self.counters
            .entry(route_rule_id.to_string())
            .or_insert_with(|| Arc::new(AtomicUsize::new(0)))
            .value()
            .clone()
    }
}

impl Default for WeightedRoundRobinStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl RouteStrategy for WeightedRoundRobinStrategy {
    fn select_group(
        &self,
        groups: &[RouteRuleGroup],
        _context: &super::context::RouteContext,
    ) -> Option<String> {
        if groups.is_empty() {
            return None;
        }

        // 获取第一个分组的 route_rule_id 作为计数器键
        let route_rule_id = &groups[0].route_rule_id;
        let counter = self.get_counter(route_rule_id);

        // 计算总权重
        let total_weight: u32 = groups.iter().map(|g| g.weight).sum();
        if total_weight == 0 {
            return None;
        }

        // 原子递增并取模
        let count = counter.fetch_add(1, Ordering::Relaxed);
        let position = (count % total_weight as usize) as u32;

        // 根据权重选择分组
        let mut accumulated = 0u32;
        for group in groups {
            accumulated += group.weight;
            if position < accumulated {
                return Some(group.group_id.clone());
            }
        }

        // 理论上不会到达这里,但提供后备方案
        Some(groups[0].group_id.clone())
    }
}

// Placeholder for upcoming tasks
pub struct CloseByVisitStrategy;

#[cfg(test)]
mod tests {
    use super::*;
    use artemis_core::model::RouteRuleGroup;
    use std::collections::HashMap;

    #[test]
    fn test_weighted_round_robin_basic() {
        let strategy = WeightedRoundRobinStrategy::new();
        let context = super::super::context::RouteContext::new();

        let groups = vec![
            RouteRuleGroup::new("rule-1".to_string(), "group-a".to_string(), 50),
            RouteRuleGroup::new("rule-1".to_string(), "group-b".to_string(), 30),
            RouteRuleGroup::new("rule-1".to_string(), "group-c".to_string(), 20),
        ];

        // 执行 1000 次选择,统计分布
        let mut counts = HashMap::new();
        for _ in 0..1000 {
            let selected = strategy.select_group(&groups, &context).unwrap();
            *counts.entry(selected).or_insert(0) += 1;
        }

        // 验证分布接近预期比例 (50:30:20)
        let a_count = counts.get("group-a").unwrap_or(&0);
        let b_count = counts.get("group-b").unwrap_or(&0);
        let c_count = counts.get("group-c").unwrap_or(&0);

        // 允许 ±5% 的误差
        assert!(*a_count >= 450 && *a_count <= 550, "group-a: {}", a_count);
        assert!(*b_count >= 250 && *b_count <= 350, "group-b: {}", b_count);
        assert!(*c_count >= 150 && *c_count <= 250, "group-c: {}", c_count);
    }

    #[test]
    fn test_weighted_round_robin_empty_groups() {
        let strategy = WeightedRoundRobinStrategy::new();
        let context = super::super::context::RouteContext::new();

        let result = strategy.select_group(&[], &context);
        assert!(result.is_none());
    }
}
