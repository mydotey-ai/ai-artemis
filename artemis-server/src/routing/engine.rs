//! 路由引擎 - 统一入口

use artemis_core::model::{Instance, RouteRule, RouteStrategy as RouteStrategyEnum};
use std::sync::Arc;
use tracing::{debug, warn};

use super::context::RouteContext;
use super::strategy::{CloseByVisitStrategy, RouteStrategy, WeightedRoundRobinStrategy};

/// 路由引擎 - 统一管理路由策略
#[derive(Clone)]
pub struct RouteEngine {
    weighted_rr: Arc<WeightedRoundRobinStrategy>,
    close_by: Arc<CloseByVisitStrategy>,
}

impl RouteEngine {
    pub fn new() -> Self {
        Self {
            weighted_rr: Arc::new(WeightedRoundRobinStrategy::new()),
            close_by: Arc::new(CloseByVisitStrategy::new()),
        }
    }

    /// 应用路由规则到实例列表
    ///
    /// 流程:
    /// 1. 使用策略选择目标分组ID
    /// 2. 根据分组ID过滤实例(从 Instance.group_id 字段读取)
    /// 3. 如果没有匹配实例,降级返回所有实例
    pub fn apply_route_rule(
        &self,
        instances: Vec<Instance>,
        rule: &RouteRule,
        context: &RouteContext,
    ) -> Vec<Instance> {
        if instances.is_empty() {
            return instances;
        }

        if rule.groups.is_empty() {
            warn!("Route rule {} has no groups, returning all instances", rule.route_id);
            return instances;
        }

        // 步骤1: 选择目标分组
        let selected_group = match rule.strategy {
            RouteStrategyEnum::WeightedRoundRobin => {
                // Note: RouteRule.groups is Vec<ServiceGroup>, need to extract RouteRuleGroup
                // For now, we'll create a temporary conversion
                let route_groups = self.convert_service_groups_to_route_groups(rule);
                self.weighted_rr.select_group(&route_groups, context)
            }
            RouteStrategyEnum::CloseByVisit => {
                let route_groups = self.convert_service_groups_to_route_groups(rule);
                self.close_by.select_group(&route_groups, context)
            }
        };

        let Some(group_id) = selected_group else {
            warn!("No group selected, returning all instances");
            return instances;
        };

        debug!("Selected group {} using strategy {:?}", group_id, rule.strategy);

        // 步骤2: 根据分组ID过滤实例 (使用 group_id 字段)
        let filtered: Vec<Instance> = instances
            .iter()
            .filter(|inst| inst.group_id.as_ref().map(|gid| gid == &group_id).unwrap_or(false))
            .cloned()
            .collect();

        // 步骤3: 降级处理
        if filtered.is_empty() {
            warn!("No instances found for group {}, returning all instances as fallback", group_id);
            instances
        } else {
            filtered
        }
    }

    /// Convert ServiceGroup to RouteRuleGroup for strategy selection
    /// This is a temporary bridge until we align the data models
    fn convert_service_groups_to_route_groups(
        &self,
        rule: &RouteRule,
    ) -> Vec<artemis_core::model::RouteRuleGroup> {
        rule.groups
            .iter()
            .map(|sg| {
                artemis_core::model::RouteRuleGroup {
                    route_rule_id: rule.route_id.clone(),
                    group_id: sg.group_key.clone(),
                    weight: sg.weight.unwrap_or(100),
                    unreleasable: false,
                    region_id: None, // ServiceGroup doesn't have region_id directly
                    zone_id: None,   // ServiceGroup doesn't have zone_id directly
                }
            })
            .collect()
    }
}

impl Default for RouteEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use artemis_core::model::{
        InstanceStatus, RouteRuleStatus, RouteStrategy as RouteStrategyEnum,
    };
    // Import ServiceGroup from service module (not group module)
    use artemis_core::model::service::ServiceGroup;

    fn create_test_instance(service_id: &str, instance_id: &str, group_id: &str) -> Instance {
        Instance {
            region_id: "us-east".to_string(),
            zone_id: "zone-1".to_string(),
            group_id: Some(group_id.to_string()),
            service_id: service_id.to_string(),
            instance_id: instance_id.to_string(),
            machine_name: None,
            ip: "192.168.1.100".to_string(),
            port: 8080,
            protocol: Some("http".to_string()),
            url: "http://192.168.1.100:8080".to_string(),
            health_check_url: None,
            status: InstanceStatus::Up,
            metadata: None,
        }
    }

    #[test]
    fn test_route_engine_weighted_round_robin() {
        let engine = RouteEngine::new();

        let instances = vec![
            create_test_instance("s1", "i1", "group-1"),
            create_test_instance("s1", "i2", "group-1"),
            create_test_instance("s1", "i3", "group-2"),
        ];

        let rule = RouteRule {
            route_rule_id: Some(1),
            route_id: "r1".to_string(),
            service_id: "s1".to_string(),
            name: "test-rule".to_string(),
            description: None,
            status: RouteRuleStatus::Active,
            strategy: RouteStrategyEnum::WeightedRoundRobin,
            groups: vec![
                ServiceGroup {
                    group_key: "group-1".to_string(),
                    weight: Some(70),
                    instance_ids: None,
                    instances: None,
                    metadata: None,
                },
                ServiceGroup {
                    group_key: "group-2".to_string(),
                    weight: Some(30),
                    instance_ids: None,
                    instances: None,
                    metadata: None,
                },
            ],
        };

        let context = RouteContext::new();

        let result = engine.apply_route_rule(instances, &rule, &context);

        // 应该返回 group-1 或 group-2 的实例
        assert!(!result.is_empty());

        // 验证所有返回的实例属于同一个分组
        if let Some(first_inst) = result.first() {
            let group_id = first_inst.group_id.as_ref().unwrap();
            assert!(result.iter().all(|inst| inst.group_id.as_ref().unwrap() == group_id));
        }
    }

    #[test]
    fn test_route_engine_close_by_visit() {
        let engine = RouteEngine::new();

        let instances = vec![
            Instance {
                region_id: "us-east".to_string(),
                zone_id: "zone-1".to_string(),
                group_id: Some("group-us-east".to_string()),
                service_id: "s1".to_string(),
                instance_id: "i1".to_string(),
                machine_name: None,
                ip: "192.168.1.100".to_string(),
                port: 8080,
                protocol: Some("http".to_string()),
                url: "http://192.168.1.100:8080".to_string(),
                health_check_url: None,
                status: InstanceStatus::Up,
                metadata: None,
            },
            Instance {
                region_id: "us-west".to_string(),
                zone_id: "zone-1".to_string(),
                group_id: Some("group-us-west".to_string()),
                service_id: "s1".to_string(),
                instance_id: "i2".to_string(),
                machine_name: None,
                ip: "192.168.1.101".to_string(),
                port: 8080,
                protocol: Some("http".to_string()),
                url: "http://192.168.1.101:8080".to_string(),
                health_check_url: None,
                status: InstanceStatus::Up,
                metadata: None,
            },
        ];

        // Note: ServiceGroup doesn't have region_id/zone_id, so CloseByVisit
        // will fall back to the first group. This test validates the fallback behavior.
        let rule = RouteRule {
            route_rule_id: Some(1),
            route_id: "r1".to_string(),
            service_id: "s1".to_string(),
            name: "test-rule".to_string(),
            description: None,
            status: RouteRuleStatus::Active,
            strategy: RouteStrategyEnum::CloseByVisit,
            groups: vec![
                ServiceGroup {
                    group_key: "group-us-east".to_string(),
                    weight: Some(50),
                    instance_ids: None,
                    instances: None,
                    metadata: None,
                },
                ServiceGroup {
                    group_key: "group-us-west".to_string(),
                    weight: Some(50),
                    instance_ids: None,
                    instances: None,
                    metadata: None,
                },
            ],
        };

        let context = RouteContext::new().with_region("us-east".to_string());

        let result = engine.apply_route_rule(instances, &rule, &context);

        // Since ServiceGroup doesn't have region_id, it will fall back to first group
        assert!(!result.is_empty());
        // Should return group-us-east instances (first group in the list)
        assert_eq!(result[0].instance_id, "i1");
    }

    #[test]
    fn test_route_engine_empty_groups() {
        let engine = RouteEngine::new();

        let instances = vec![create_test_instance("s1", "i1", "group-1")];

        let rule = RouteRule {
            route_rule_id: Some(1),
            route_id: "r1".to_string(),
            service_id: "s1".to_string(),
            name: "test-rule".to_string(),
            description: None,
            status: RouteRuleStatus::Active,
            strategy: RouteStrategyEnum::WeightedRoundRobin,
            groups: vec![],
        };

        let context = RouteContext::new();

        let result = engine.apply_route_rule(instances.clone(), &rule, &context);

        // 空分组应该返回所有实例
        assert_eq!(result.len(), instances.len());
    }

    #[test]
    fn test_route_engine_empty_instances() {
        let engine = RouteEngine::new();

        let instances = vec![];

        let rule = RouteRule {
            route_rule_id: Some(1),
            route_id: "r1".to_string(),
            service_id: "s1".to_string(),
            name: "test-rule".to_string(),
            description: None,
            status: RouteRuleStatus::Active,
            strategy: RouteStrategyEnum::WeightedRoundRobin,
            groups: vec![ServiceGroup {
                group_key: "group-1".to_string(),
                weight: Some(100),
                instance_ids: None,
                instances: None,
                metadata: None,
            }],
        };

        let context = RouteContext::new();

        let result = engine.apply_route_rule(instances, &rule, &context);
        assert!(result.is_empty());
    }
}
