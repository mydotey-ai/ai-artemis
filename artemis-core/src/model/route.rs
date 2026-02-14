use super::service::ServiceGroup;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 服务路由规则 (Service Route Rule)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteRule {
    /// 路由规则 ID (自动生成或手动指定)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route_rule_id: Option<i64>,
    /// 路由规则唯一标识 (service_id + name)
    pub route_id: String,
    /// 所属服务 ID
    pub service_id: String,
    /// 规则名称
    pub name: String,
    /// 规则描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// 规则状态 (active/inactive)
    pub status: RouteRuleStatus,
    /// 路由策略
    pub strategy: RouteStrategy,
    /// 关联的分组列表 (包含权重)
    pub groups: Vec<ServiceGroup>,
}

/// 路由规则状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RouteRuleStatus {
    Active,
    Inactive,
}

/// 路由规则分组关联 (带权重)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RouteRuleGroup {
    /// 路由规则 ID
    pub route_rule_id: String,
    /// 分组 ID
    pub group_id: String,
    /// 权重 (1-100)
    pub weight: u32,
    /// 是否可发布
    pub unreleasable: bool,
    /// 分组所在 Region (用于就近访问路由)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region_id: Option<String>,
    /// 分组所在 Zone (用于就近访问路由)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zone_id: Option<String>,
}

impl RouteRuleGroup {
    pub fn new(route_rule_id: String, group_id: String, weight: u32) -> Self {
        Self {
            route_rule_id,
            group_id,
            weight: weight.clamp(1, 100),
            unreleasable: false,
            region_id: None,
            zone_id: None,
        }
    }

    /// 创建带地理位置信息的分组
    pub fn with_location(
        route_rule_id: String,
        group_id: String,
        weight: u32,
        region_id: Option<String>,
        zone_id: Option<String>,
    ) -> Self {
        Self {
            route_rule_id,
            group_id,
            weight: weight.clamp(1, 100),
            unreleasable: false,
            region_id,
            zone_id,
        }
    }
}

/// 路由策略
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RouteStrategy {
    /// 加权轮询
    WeightedRoundRobin,
    /// 就近访问
    CloseByVisit,
}

/// 服务分组 (完整定义)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    /// 分组 ID (自动生成)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_id: Option<i64>,
    /// 所属服务 ID
    pub service_id: String,
    /// Region ID
    pub region_id: String,
    /// Zone ID
    pub zone_id: String,
    /// 分组名称
    pub name: String,
    /// 应用 ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_id: Option<String>,
    /// 分组描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// 分组状态
    pub status: GroupStatus,
    /// 分组元数据
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
}

/// 分组状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum GroupStatus {
    Active,
    Inactive,
}

impl Group {
    /// 生成分组唯一键: service_id:region_id:zone_id:name
    pub fn group_key(&self) -> String {
        format!("{}:{}:{}:{}", self.service_id, self.region_id, self.zone_id, self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_rule_group_weight_clamp() {
        let group = RouteRuleGroup::new("r1".to_string(), "g1".to_string(), 150);
        assert_eq!(group.weight, 100);

        let group = RouteRuleGroup::new("r1".to_string(), "g1".to_string(), 0);
        assert_eq!(group.weight, 1);
    }
}
