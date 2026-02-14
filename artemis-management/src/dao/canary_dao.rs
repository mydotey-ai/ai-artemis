use artemis_core::model::CanaryConfig;
use sqlx::{SqlitePool, Row};

pub struct CanaryConfigDao {
    pool: SqlitePool,
}

impl CanaryConfigDao {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// 插入或更新金丝雀配置
    pub async fn upsert_config(&self, config: &CanaryConfig) -> anyhow::Result<()> {
        let ip_whitelist_json = serde_json::to_string(&config.ip_whitelist)?;

        sqlx::query(
            r#"
            INSERT INTO canary_config (service_id, ip_whitelist, enabled)
            VALUES (?, ?, ?)
            ON CONFLICT(service_id) DO UPDATE SET ip_whitelist = excluded.ip_whitelist, enabled = excluded.enabled, updated_at = CURRENT_TIMESTAMP
            "#
        )
        .bind(&config.service_id)
        .bind(&ip_whitelist_json)
        .bind(config.enabled)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 删除金丝雀配置
    pub async fn delete_config(&self, service_id: &str) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM canary_config WHERE service_id = ?")
            .bind(service_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// 获取金丝雀配置
    pub async fn get_config(&self, service_id: &str) -> anyhow::Result<Option<CanaryConfig>> {
        let row = sqlx::query(
            r#"
            SELECT service_id, ip_whitelist, enabled
            FROM canary_config
            WHERE service_id = ?
            "#
        )
        .bind(service_id)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => {
                let ip_whitelist_json: String = row.get("ip_whitelist");
                let ip_whitelist: Vec<String> = serde_json::from_str(&ip_whitelist_json)?;

                Ok(Some(CanaryConfig {
                    service_id: row.get("service_id"),
                    ip_whitelist,
                    enabled: row.get("enabled"),
                }))
            }
            None => Ok(None),
        }
    }

    /// 列出所有金丝雀配置
    pub async fn list_configs(&self) -> anyhow::Result<Vec<CanaryConfig>> {
        let rows = sqlx::query(
            r#"
            SELECT service_id, ip_whitelist, enabled
            FROM canary_config
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        let mut configs = Vec::new();
        for row in rows {
            let ip_whitelist_json: String = row.get("ip_whitelist");
            let ip_whitelist: Vec<String> = serde_json::from_str(&ip_whitelist_json)?;

            configs.push(CanaryConfig {
                service_id: row.get("service_id"),
                ip_whitelist,
                enabled: row.get("enabled"),
            });
        }

        Ok(configs)
    }

    /// 设置启用状态
    pub async fn set_enabled(&self, service_id: &str, enabled: bool) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            UPDATE canary_config
            SET enabled = ?, updated_at = CURRENT_TIMESTAMP
            WHERE service_id = ?
            "#
        )
        .bind(enabled)
        .bind(service_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
