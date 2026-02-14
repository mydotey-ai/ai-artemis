use artemis_core::model::{RouteRule, RouteRuleStatus, RouteStrategy};
use artemis_core::model::service::ServiceGroup;  // service.rs中的ServiceGroup,有weight字段
use sqlx::{Pool, Any, Row};

pub struct RouteRuleDao {
    pool: Pool<Any>,
}

impl RouteRuleDao {
    pub fn new(pool: Pool<Any>) -> Self {
        Self { pool }
    }

    /// 插入路由规则
    pub async fn insert_rule(&self, rule: &RouteRule) -> anyhow::Result<()> {
        let status_str = match rule.status {
            RouteRuleStatus::Active => "active",
            RouteRuleStatus::Inactive => "inactive",
        };

        let strategy_str = match rule.strategy {
            RouteStrategy::WeightedRoundRobin => "weighted-round-robin",
            RouteStrategy::CloseByVisit => "close-by-visit",
        };

        sqlx::query(
            r#"
            INSERT INTO service_route_rule (route_rule_id, route_id, service_id, name, description, status, strategy)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(rule.route_rule_id.unwrap_or(0))
        .bind(&rule.route_id)
        .bind(&rule.service_id)
        .bind(&rule.name)
        .bind(&rule.description)
        .bind(status_str)
        .bind(strategy_str)
        .execute(&self.pool)
        .await?;

        // 插入规则分组关联
        for group in &rule.groups {
            self.insert_rule_group(&rule.route_id, group).await?;
        }

        Ok(())
    }

    /// 更新路由规则
    pub async fn update_rule(&self, rule: &RouteRule) -> anyhow::Result<()> {
        let status_str = match rule.status {
            RouteRuleStatus::Active => "active",
            RouteRuleStatus::Inactive => "inactive",
        };

        let strategy_str = match rule.strategy {
            RouteStrategy::WeightedRoundRobin => "weighted-round-robin",
            RouteStrategy::CloseByVisit => "close-by-visit",
        };

        sqlx::query(
            r#"
            UPDATE service_route_rule
            SET name = ?, description = ?, status = ?, strategy = ?, updated_at = CURRENT_TIMESTAMP
            WHERE route_id = ?
            "#
        )
        .bind(&rule.name)
        .bind(&rule.description)
        .bind(status_str)
        .bind(strategy_str)
        .bind(&rule.route_id)
        .execute(&self.pool)
        .await?;

        // 删除旧的规则分组关联并插入新的
        sqlx::query("DELETE FROM service_route_rule_group WHERE rule_id = ?")
            .bind(&rule.route_id)
            .execute(&self.pool)
            .await?;

        for group in &rule.groups {
            self.insert_rule_group(&rule.route_id, group).await?;
        }

        Ok(())
    }

    /// 删除路由规则
    pub async fn delete_rule(&self, rule_id: &str) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM service_route_rule WHERE route_id = ?")
            .bind(rule_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// 获取路由规则 (不加载groups,由Manager负责加载)
    pub async fn get_rule(&self, rule_id: &str) -> anyhow::Result<Option<RouteRule>> {
        let row = sqlx::query(
            r#"
            SELECT route_rule_id, route_id, service_id, name, description, status, strategy
            FROM service_route_rule
            WHERE route_id = ?
            "#
        )
        .bind(rule_id)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => {
                let status_str: String = row.get("status");
                let status = match status_str.as_str() {
                    "active" => RouteRuleStatus::Active,
                    _ => RouteRuleStatus::Inactive,
                };

                let strategy_str: String = row.get("strategy");
                let strategy = match strategy_str.as_str() {
                    "close-by-visit" => RouteStrategy::CloseByVisit,
                    _ => RouteStrategy::WeightedRoundRobin,
                };

                Ok(Some(RouteRule {
                    route_rule_id: row.get("route_rule_id"),
                    route_id: row.get("route_id"),
                    service_id: row.get("service_id"),
                    name: row.get("name"),
                    description: row.get("description"),
                    status,
                    strategy,
                    groups: vec![], // Manager通过get_rule_group_ids加载
                }))
            }
            None => Ok(None),
        }
    }

    /// 列出所有路由规则 (不加载groups)
    pub async fn list_rules(&self) -> anyhow::Result<Vec<RouteRule>> {
        let rows = sqlx::query(
            r#"
            SELECT route_rule_id, route_id, service_id, name, description, status, strategy
            FROM service_route_rule
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        let mut rules = Vec::new();
        for row in rows {
            let status_str: String = row.get("status");
            let status = match status_str.as_str() {
                "active" => RouteRuleStatus::Active,
                _ => RouteRuleStatus::Inactive,
            };

            let strategy_str: String = row.get("strategy");
            let strategy = match strategy_str.as_str() {
                "close-by-visit" => RouteStrategy::CloseByVisit,
                _ => RouteStrategy::WeightedRoundRobin,
            };

            rules.push(RouteRule {
                route_rule_id: row.get("route_rule_id"),
                route_id: row.get("route_id"),
                service_id: row.get("service_id"),
                name: row.get("name"),
                description: row.get("description"),
                status,
                strategy,
                groups: vec![], // Manager负责加载
            });
        }

        Ok(rules)
    }

    /// 插入规则分组关联 (存储group_key和weight)
    async fn insert_rule_group(&self, rule_id: &str, group: &ServiceGroup) -> anyhow::Result<()> {
        let weight = group.weight.unwrap_or(100);

        sqlx::query(
            r#"
            INSERT INTO service_route_rule_group (rule_id, group_id, weight, region_id)
            VALUES (?, ?, ?, ?)
            ON CONFLICT(rule_id, group_id) DO UPDATE SET weight = excluded.weight
            "#
        )
        .bind(rule_id)
        .bind(&group.group_key)
        .bind(weight as i32)
        .bind("")  // region_id 暂时为空
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// 获取规则关联的所有group_id列表
    pub async fn get_rule_group_ids(&self, rule_id: &str) -> anyhow::Result<Vec<(String, u32)>> {
        let rows = sqlx::query(
            r#"
            SELECT group_id, weight
            FROM service_route_rule_group
            WHERE rule_id = ?
            "#
        )
        .bind(rule_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| {
                let weight: i32 = row.get("weight");
                (row.get::<String, _>("group_id"), weight as u32)
            })
            .collect())
    }
}
