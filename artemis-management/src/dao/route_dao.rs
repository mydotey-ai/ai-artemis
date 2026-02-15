use artemis_core::model::{RouteRule, RouteRuleStatus, RouteStrategy};
use artemis_core::model::service::ServiceGroup;
use sea_orm::{DatabaseConnection, Statement, ConnectionTrait};
use sea_orm::sea_query::Value;

pub struct RouteRuleDao {
    conn: DatabaseConnection,
}

impl RouteRuleDao {
    pub fn new(conn: DatabaseConnection) -> Self {
        Self { conn }
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

        let stmt = Statement::from_sql_and_values(
            self.conn.get_database_backend(),
            r#"
            INSERT INTO service_route_rule (route_rule_id, route_id, service_id, name, description, status, strategy)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
            vec![
                Value::from(rule.route_rule_id.unwrap_or(0)),
                Value::from(&rule.route_id),
                Value::from(&rule.service_id),
                Value::from(&rule.name),
                Value::from(rule.description.as_deref().unwrap_or("")),
                Value::from(status_str),
                Value::from(strategy_str),
            ],
        );

        self.conn.execute(stmt).await?;

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

        let stmt = Statement::from_sql_and_values(
            self.conn.get_database_backend(),
            r#"
            UPDATE service_route_rule
            SET name = ?, description = ?, status = ?, strategy = ?, updated_at = CURRENT_TIMESTAMP
            WHERE route_id = ?
            "#,
            vec![
                Value::from(&rule.name),
                Value::from(rule.description.as_deref().unwrap_or("")),
                Value::from(status_str),
                Value::from(strategy_str),
                Value::from(&rule.route_id),
            ],
        );

        self.conn.execute(stmt).await?;

        // 删除旧的规则分组关联并插入新的
        let delete_stmt = Statement::from_sql_and_values(
            self.conn.get_database_backend(),
            "DELETE FROM service_route_rule_group WHERE rule_id = ?",
            vec![Value::from(&rule.route_id)],
        );
        self.conn.execute(delete_stmt).await?;

        for group in &rule.groups {
            self.insert_rule_group(&rule.route_id, group).await?;
        }

        Ok(())
    }

    /// 删除路由规则
    pub async fn delete_rule(&self, rule_id: &str) -> anyhow::Result<()> {
        let stmt = Statement::from_sql_and_values(
            self.conn.get_database_backend(),
            "DELETE FROM service_route_rule WHERE route_id = ?",
            vec![Value::from(rule_id)],
        );
        self.conn.execute(stmt).await?;
        Ok(())
    }

    /// 获取路由规则 (不加载groups,由Manager负责加载)
    pub async fn get_rule(&self, rule_id: &str) -> anyhow::Result<Option<RouteRule>> {
        let stmt = Statement::from_sql_and_values(
            self.conn.get_database_backend(),
            r#"
            SELECT route_rule_id, route_id, service_id, name, description, status, strategy
            FROM service_route_rule
            WHERE route_id = ?
            "#,
            vec![Value::from(rule_id)],
        );

        let result = self.conn.query_one(stmt).await?;

        match result {
            Some(row) => {
                let status_str: String = row.try_get("", "status")?;
                let status = match status_str.as_str() {
                    "active" => RouteRuleStatus::Active,
                    _ => RouteRuleStatus::Inactive,
                };

                let strategy_str: String = row.try_get("", "strategy")?;
                let strategy = match strategy_str.as_str() {
                    "close-by-visit" => RouteStrategy::CloseByVisit,
                    _ => RouteStrategy::WeightedRoundRobin,
                };

                Ok(Some(RouteRule {
                    route_rule_id: row.try_get("", "route_rule_id").ok(),
                    route_id: row.try_get("", "route_id")?,
                    service_id: row.try_get("", "service_id")?,
                    name: row.try_get("", "name")?,
                    description: row.try_get("", "description")?,
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
        let stmt = Statement::from_sql_and_values(
            self.conn.get_database_backend(),
            r#"
            SELECT route_rule_id, route_id, service_id, name, description, status, strategy
            FROM service_route_rule
            "#,
            vec![],
        );

        let rows = self.conn.query_all(stmt).await?;

        let mut rules = Vec::new();
        for row in rows {
            let status_str: String = row.try_get("", "status")?;
            let status = match status_str.as_str() {
                "active" => RouteRuleStatus::Active,
                _ => RouteRuleStatus::Inactive,
            };

            let strategy_str: String = row.try_get("", "strategy")?;
            let strategy = match strategy_str.as_str() {
                "close-by-visit" => RouteStrategy::CloseByVisit,
                _ => RouteStrategy::WeightedRoundRobin,
            };

            rules.push(RouteRule {
                route_rule_id: row.try_get("", "route_rule_id").ok(),
                route_id: row.try_get("", "route_id")?,
                service_id: row.try_get("", "service_id")?,
                name: row.try_get("", "name")?,
                description: row.try_get("", "description")?,
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

        let stmt = Statement::from_sql_and_values(
            self.conn.get_database_backend(),
            r#"
            INSERT INTO service_route_rule_group (rule_id, group_id, weight, region_id)
            VALUES (?, ?, ?, ?)
            ON CONFLICT(rule_id, group_id) DO UPDATE SET weight = excluded.weight
            "#,
            vec![
                Value::from(rule_id),
                Value::from(&group.group_key),
                Value::from(weight as i32),
                Value::from(""),  // region_id 暂时为空
            ],
        );

        self.conn.execute(stmt).await?;
        Ok(())
    }

    /// 获取规则关联的所有group_id列表
    pub async fn get_rule_group_ids(&self, rule_id: &str) -> anyhow::Result<Vec<(String, u32)>> {
        let stmt = Statement::from_sql_and_values(
            self.conn.get_database_backend(),
            r#"
            SELECT group_id, weight
            FROM service_route_rule_group
            WHERE rule_id = ?
            "#,
            vec![Value::from(rule_id)],
        );

        let rows = self.conn.query_all(stmt).await?;

        Ok(rows
            .into_iter()
            .map(|row| {
                let weight: i32 = row.try_get("", "weight").unwrap_or(100);
                let group_id: String = row.try_get("", "group_id").unwrap_or_default();
                (group_id, weight as u32)
            })
            .collect())
    }
}
