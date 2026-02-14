use artemis_core::model::{RouteRule, RouteRuleStatus, RouteStrategy};
use sqlx::{SqlitePool, Row};

pub struct RouteRuleDao {
    pool: SqlitePool,
}

impl RouteRuleDao {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// 插入路由规则
    /// TODO: 实现分组关联的持久化 (groups字段)
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

    /// 获取路由规则 - 简化版
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
                    groups: vec![], // TODO: 从数据库加载groups
                }))
            }
            None => Ok(None),
        }
    }

    /// 列出所有路由规则 - 简化版
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
                groups: vec![], // TODO: 从数据库加载groups
            });
        }

        Ok(rules)
    }
}
