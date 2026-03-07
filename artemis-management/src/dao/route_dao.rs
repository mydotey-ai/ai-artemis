use crate::model::{RouteRule, RouteRuleStatus, RouteStrategy};
use artemis_common::model::service::ServiceGroup;
use sea_orm::sea_query::Value;
use sea_orm::{ConnectionTrait, DatabaseConnection, Statement};

pub struct RouteRuleDao {
    conn: DatabaseConnection,
}

impl RouteRuleDao {
    pub fn new(conn: DatabaseConnection) -> Self {
        Self { conn }
    }

    /// 插入路由规则 (兼容 Java 列名)
    pub async fn insert_rule(&self, rule: &RouteRule) -> anyhow::Result<()> {
        let status_str = match rule.status {
            RouteRuleStatus::Active => "active",
            RouteRuleStatus::Inactive => "inactive",
        };

        let strategy_str = match rule.strategy {
            RouteStrategy::WeightedRoundRobin => "weighted-round-robin",
            RouteStrategy::CloseByVisit => "close-by-visit",
        };

        // 使用 Java 列名: SERVICE_ID, NAME, DESCRIPTION, STATUS, strategy
        let stmt = Statement::from_sql_and_values(
            self.conn.get_database_backend(),
            r#"
            INSERT INTO service_route_rule (SERVICE_ID, NAME, DESCRIPTION, STATUS, strategy)
            VALUES (?, ?, ?, ?, ?)
            "#,
            vec![
                Value::from(&rule.service_id),
                Value::from(&rule.route_id), // NAME 使用 route_id 作为标识符
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

    /// 更新路由规则 (兼容 Java 列名)
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
            SET NAME = ?, DESCRIPTION = ?, STATUS = ?, strategy = ?, DataChange_LastTime = CURRENT_TIMESTAMP
            WHERE NAME = ?
            "#,
            vec![
                Value::from(&rule.route_id),
                Value::from(rule.description.as_deref().unwrap_or("")),
                Value::from(status_str),
                Value::from(strategy_str),
                Value::from(&rule.route_id),
            ],
        );

        self.conn.execute(stmt).await?;

        // 删除旧的规则分组关联并插入新的
        // 先获取 route_rule_id
        let route_rule_id = self.get_route_rule_id(&rule.route_id).await?;
        if let Some(id) = route_rule_id {
            let delete_stmt = Statement::from_sql_and_values(
                self.conn.get_database_backend(),
                "DELETE FROM service_route_rule_group WHERE ROUTE_RULE_ID = ?",
                vec![Value::from(id)],
            );
            self.conn.execute(delete_stmt).await?;
        }

        for group in &rule.groups {
            self.insert_rule_group(&rule.route_id, group).await?;
        }

        Ok(())
    }

    /// 获取路由规则 ID
    async fn get_route_rule_id(&self, rule_name: &str) -> anyhow::Result<Option<i64>> {
        let stmt = Statement::from_sql_and_values(
            self.conn.get_database_backend(),
            "SELECT id FROM service_route_rule WHERE NAME = ?",
            vec![Value::from(rule_name)],
        );

        let result = self.conn.query_one(stmt).await?;
        match result {
            Some(row) => Ok(Some(row.try_get("", "id")?)),
            None => Ok(None),
        }
    }

    /// 删除路由规则 (兼容 Java 列名)
    pub async fn delete_rule(&self, rule_id: &str) -> anyhow::Result<()> {
        // 先删除关联的分组
        let route_rule_id = self.get_route_rule_id(rule_id).await?;
        if let Some(id) = route_rule_id {
            let delete_assoc_stmt = Statement::from_sql_and_values(
                self.conn.get_database_backend(),
                "DELETE FROM service_route_rule_group WHERE ROUTE_RULE_ID = ?",
                vec![Value::from(id)],
            );
            self.conn.execute(delete_assoc_stmt).await?;
        }

        let stmt = Statement::from_sql_and_values(
            self.conn.get_database_backend(),
            "DELETE FROM service_route_rule WHERE NAME = ?",
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
            SELECT id, SERVICE_ID, NAME, DESCRIPTION, STATUS, strategy
            FROM service_route_rule
            WHERE NAME = ?
            "#,
            vec![Value::from(rule_id)],
        );

        let result = self.conn.query_one(stmt).await?;

        match result {
            Some(row) => {
                let status_str: String = row.try_get("", "STATUS")?;
                let status = match status_str.as_str() {
                    "active" => RouteRuleStatus::Active,
                    _ => RouteRuleStatus::Inactive,
                };

                let strategy_str: String = row.try_get("", "strategy")?;
                let strategy = match strategy_str.as_str() {
                    "close-by-visit" => RouteStrategy::CloseByVisit,
                    _ => RouteStrategy::WeightedRoundRobin,
                };

                let route_rule_id: i64 = row.try_get("", "id")?;

                Ok(Some(RouteRule {
                    route_rule_id: Some(route_rule_id),
                    route_id: row.try_get("", "NAME")?,
                    service_id: row.try_get("", "SERVICE_ID")?,
                    name: row.try_get("", "NAME")?,
                    description: row.try_get("", "DESCRIPTION")?,
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
            SELECT id, SERVICE_ID, NAME, DESCRIPTION, STATUS, strategy
            FROM service_route_rule
            "#,
            vec![],
        );

        let rows = self.conn.query_all(stmt).await?;

        let mut rules = Vec::new();
        for row in rows {
            let status_str: String = row.try_get("", "STATUS")?;
            let status = match status_str.as_str() {
                "active" => RouteRuleStatus::Active,
                _ => RouteRuleStatus::Inactive,
            };

            let strategy_str: String = row.try_get("", "strategy")?;
            let strategy = match strategy_str.as_str() {
                "close-by-visit" => RouteStrategy::CloseByVisit,
                _ => RouteStrategy::WeightedRoundRobin,
            };

            let route_rule_id: i64 = row.try_get("", "id")?;

            rules.push(RouteRule {
                route_rule_id: Some(route_rule_id),
                route_id: row.try_get("", "NAME")?,
                service_id: row.try_get("", "SERVICE_ID")?,
                name: row.try_get("", "NAME")?,
                description: row.try_get("", "DESCRIPTION")?,
                status,
                strategy,
                groups: vec![], // Manager负责加载
            });
        }

        Ok(rules)
    }

    /// 插入规则分组关联 (兼容 Java 列名)
    async fn insert_rule_group(&self, rule_id: &str, group: &ServiceGroup) -> anyhow::Result<()> {
        let weight = group.weight.unwrap_or(100);

        // 获取 route_rule_id
        let route_rule_id = self.get_route_rule_id(rule_id).await?.unwrap_or(0);

        // 获取 group_id (需要通过 group_name 查询)
        let group_id = self.get_group_id_by_name(&group.group_key).await?;

        let stmt = Statement::from_sql_and_values(
            self.conn.get_database_backend(),
            r#"
            INSERT INTO service_route_rule_group (ROUTE_RULE_ID, GROUP_ID, WEIGHT)
            VALUES (?, ?, ?)
            "#,
            vec![
                Value::from(route_rule_id),
                Value::from(group_id),
                Value::from(weight as i32),
            ],
        );

        self.conn.execute(stmt).await?;
        Ok(())
    }

    /// 根据 group_name 获取 group_id
    async fn get_group_id_by_name(&self, group_name: &str) -> anyhow::Result<i64> {
        let stmt = Statement::from_sql_and_values(
            self.conn.get_database_backend(),
            "SELECT id FROM service_group WHERE NAME = ?",
            vec![Value::from(group_name)],
        );

        let result = self.conn.query_one(stmt).await?;
        match result {
            Some(row) => Ok(row.try_get("", "id")?),
            None => Ok(0), // 如果找不到返回 0
        }
    }

    /// 获取规则关联的所有group_id列表
    pub async fn get_rule_group_ids(&self, rule_id: &str) -> anyhow::Result<Vec<(String, u32)>> {
        // 先获取 route_rule_id
        let route_rule_id = match self.get_route_rule_id(rule_id).await? {
            Some(id) => id,
            None => return Ok(vec![]),
        };

        let stmt = Statement::from_sql_and_values(
            self.conn.get_database_backend(),
            r#"
            SELECT rg.GROUP_ID, rg.WEIGHT, g.NAME
            FROM service_route_rule_group rg
            LEFT JOIN service_group g ON rg.GROUP_ID = g.id
            WHERE rg.ROUTE_RULE_ID = ?
            "#,
            vec![Value::from(route_rule_id)],
        );

        let rows = self.conn.query_all(stmt).await?;

        Ok(rows
            .into_iter()
            .map(|row| {
                let weight: i32 = row.try_get("", "WEIGHT").unwrap_or(100);
                let group_name: String = row.try_get("", "NAME").unwrap_or_default();
                (group_name, weight as u32)
            })
            .collect())
    }
}
