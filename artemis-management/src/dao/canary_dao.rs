use artemis_core::model::CanaryConfig;
use sea_orm::sea_query::Value;
use sea_orm::{ConnectionTrait, DatabaseConnection, Statement};

pub struct CanaryConfigDao {
    conn: DatabaseConnection,
}

impl CanaryConfigDao {
    pub fn new(conn: DatabaseConnection) -> Self {
        Self { conn }
    }

    /// 插入或更新金丝雀配置
    pub async fn upsert_config(&self, config: &CanaryConfig) -> anyhow::Result<()> {
        let ip_whitelist_json = serde_json::to_string(&config.ip_whitelist)?;

        let stmt = Statement::from_sql_and_values(
            self.conn.get_database_backend(),
            r#"
            INSERT INTO canary_config (service_id, ip_whitelist, enabled)
            VALUES (?, ?, ?)
            ON CONFLICT(service_id) DO UPDATE SET ip_whitelist = excluded.ip_whitelist, enabled = excluded.enabled, updated_at = CURRENT_TIMESTAMP
            "#,
            vec![
                Value::from(&config.service_id),
                Value::from(ip_whitelist_json),
                Value::from(config.enabled),
            ],
        );

        self.conn.execute(stmt).await?;
        Ok(())
    }

    /// 删除金丝雀配置
    pub async fn delete_config(&self, service_id: &str) -> anyhow::Result<()> {
        let stmt = Statement::from_sql_and_values(
            self.conn.get_database_backend(),
            "DELETE FROM canary_config WHERE service_id = ?",
            vec![Value::from(service_id)],
        );
        self.conn.execute(stmt).await?;
        Ok(())
    }

    /// 获取金丝雀配置
    pub async fn get_config(&self, service_id: &str) -> anyhow::Result<Option<CanaryConfig>> {
        let stmt = Statement::from_sql_and_values(
            self.conn.get_database_backend(),
            r#"
            SELECT service_id, ip_whitelist, enabled
            FROM canary_config
            WHERE service_id = ?
            "#,
            vec![Value::from(service_id)],
        );

        let result = self.conn.query_one(stmt).await?;

        match result {
            Some(row) => {
                let ip_whitelist_json: String = row.try_get("", "ip_whitelist")?;
                let ip_whitelist: Vec<String> = serde_json::from_str(&ip_whitelist_json)?;

                Ok(Some(CanaryConfig {
                    service_id: row.try_get("", "service_id")?,
                    ip_whitelist,
                    enabled: row.try_get("", "enabled")?,
                }))
            }
            None => Ok(None),
        }
    }

    /// 列出所有金丝雀配置
    pub async fn list_configs(&self) -> anyhow::Result<Vec<CanaryConfig>> {
        let stmt = Statement::from_sql_and_values(
            self.conn.get_database_backend(),
            r#"
            SELECT service_id, ip_whitelist, enabled
            FROM canary_config
            "#,
            vec![],
        );

        let rows = self.conn.query_all(stmt).await?;

        let mut configs = Vec::new();
        for row in rows {
            let ip_whitelist_json: String = row.try_get("", "ip_whitelist")?;
            let ip_whitelist: Vec<String> = serde_json::from_str(&ip_whitelist_json)?;

            configs.push(CanaryConfig {
                service_id: row.try_get("", "service_id")?,
                ip_whitelist,
                enabled: row.try_get("", "enabled")?,
            });
        }

        Ok(configs)
    }

    /// 设置启用状态
    pub async fn set_enabled(&self, service_id: &str, enabled: bool) -> anyhow::Result<()> {
        let stmt = Statement::from_sql_and_values(
            self.conn.get_database_backend(),
            r#"
            UPDATE canary_config
            SET enabled = ?, updated_at = CURRENT_TIMESTAMP
            WHERE service_id = ?
            "#,
            vec![Value::from(enabled), Value::from(service_id)],
        );

        self.conn.execute(stmt).await?;
        Ok(())
    }
}
