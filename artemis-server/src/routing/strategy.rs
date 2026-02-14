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

/// 就近访问策略
///
/// 根据客户端的地理位置(Region/Zone)选择最近的分组:
/// - 优先级1: 匹配相同 Region 的分组
/// - 优先级2: 匹配相同 Zone 的分组
/// - 降级: 返回第一个分组
#[derive(Clone)]
pub struct CloseByVisitStrategy;

impl CloseByVisitStrategy {
    pub fn new() -> Self {
        Self
    }
}

impl Default for CloseByVisitStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl RouteStrategy for CloseByVisitStrategy {
    fn select_group(
        &self,
        groups: &[RouteRuleGroup],
        context: &super::context::RouteContext,
    ) -> Option<String> {
        if groups.is_empty() {
            return None;
        }

        // 优先级1: 匹配相同 Region
        if let Some(client_region) = &context.client_region {
            for group in groups {
                if let Some(group_region) = &group.region_id {
                    if group_region == client_region {
                        return Some(group.group_id.clone());
                    }
                }
            }
        }

        // 优先级2: 匹配相同 Zone
        if let Some(client_zone) = &context.client_zone {
            for group in groups {
                if let Some(group_zone) = &group.zone_id {
                    if group_zone == client_zone {
                        return Some(group.group_id.clone());
                    }
                }
            }
        }

        // 降级: 返回第一个分组
        Some(groups[0].group_id.clone())
    }
}

#[cfg(test)]
mod weighted_round_robin_tests {
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

#[cfg(test)]
mod close_by_visit_tests {
    use super::*;
    use artemis_core::model::RouteRuleGroup;

    #[test]
    fn test_close_by_visit_same_region() {
        let strategy = CloseByVisitStrategy::new();

        let groups = vec![
            RouteRuleGroup::with_location(
                "rule-1".to_string(),
                "group-us-east".to_string(),
                50,
                Some("us-east".to_string()),
                Some("zone-1".to_string()),
            ),
            RouteRuleGroup::with_location(
                "rule-1".to_string(),
                "group-us-west".to_string(),
                30,
                Some("us-west".to_string()),
                Some("zone-1".to_string()),
            ),
            RouteRuleGroup::with_location(
                "rule-1".to_string(),
                "group-eu".to_string(),
                20,
                Some("eu-central".to_string()),
                None,
            ),
        ];

        let context = super::super::context::RouteContext::new()
            .with_region("us-east".to_string());

        let selected = strategy.select_group(&groups, &context).unwrap();
        assert_eq!(selected, "group-us-east");
    }

    #[test]
    fn test_close_by_visit_same_zone() {
        let strategy = CloseByVisitStrategy::new();

        let groups = vec![
            RouteRuleGroup::with_location(
                "rule-1".to_string(),
                "group-zone1".to_string(),
                50,
                None,
                Some("zone-1".to_string()),
            ),
            RouteRuleGroup::with_location(
                "rule-1".to_string(),
                "group-zone2".to_string(),
                50,
                None,
                Some("zone-2".to_string()),
            ),
        ];

        let context = super::super::context::RouteContext::new()
            .with_zone("zone-2".to_string());

        let selected = strategy.select_group(&groups, &context).unwrap();
        assert_eq!(selected, "group-zone2");
    }

    #[test]
    fn test_close_by_visit_fallback() {
        let strategy = CloseByVisitStrategy::new();

        let groups = vec![
            RouteRuleGroup::with_location(
                "rule-1".to_string(),
                "group-default".to_string(),
                100,
                Some("us-east".to_string()),
                None,
            ),
        ];

        // 客户端位置不匹配,降级到第一个分组
        let context = super::super::context::RouteContext::new()
            .with_region("eu-west".to_string());

        let selected = strategy.select_group(&groups, &context).unwrap();
        assert_eq!(selected, "group-default");
    }

    #[test]
    fn test_close_by_visit_empty_groups() {
        let strategy = CloseByVisitStrategy::new();
        let context = super::super::context::RouteContext::new();

        let result = strategy.select_group(&[], &context);
        assert!(result.is_none());
    }
}
