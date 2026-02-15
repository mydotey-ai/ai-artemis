use anyhow::Result;
use artemis_core::model::group::{BindingType, GroupInstance};
use sea_orm::{ConnectionTrait, DatabaseBackend, DbConn, Statement};

/// GroupInstance DAO - 分组实例绑定关系持久化
pub struct GroupInstanceDao {
    db: DbConn,
}

impl GroupInstanceDao {
    pub fn new(db: DbConn) -> Self {
        Self { db }
    }

    /// 插入分组实例绑定
    pub async fn insert(&self, binding: &GroupInstance) -> Result<i64> {
        let stmt = Statement::from_sql_and_values(
            DatabaseBackend::Sqlite,
            r#"
                INSERT INTO service_group_instance
                (group_id, region_id, zone_id, service_id, instance_id, ip, port, binding_type, operator_id, created_at)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            vec![
                binding.group_id.to_string().into(),
                binding.region_id.clone().into(),
                binding.zone_id.clone().into(),
                binding.service_id.clone().into(),
                binding.instance_id.clone().into(),
                binding.binding_type.as_ref().map(|_| "").into(), // ip optional
                binding.binding_type.as_ref().map(|_| 0).into(), // port optional
                binding
                    .binding_type
                    .map(|t| match t {
                        BindingType::Manual => "manual",
                        BindingType::Auto => "auto",
                    })
                    .unwrap_or("auto")
                    .into(),
                binding.operator_id.clone().into(),
                binding.created_at.unwrap_or_else(|| chrono::Utc::now().timestamp()).into(),
            ],
        );

        self.db.execute(stmt).await?;
        Ok(1) // SQLite 不直接返回 last_insert_id,这里简化处理
    }

    /// 删除分组实例绑定
    pub async fn delete(
        &self,
        group_id: i64,
        instance_id: &str,
        region_id: &str,
        zone_id: &str,
    ) -> Result<bool> {
        let stmt = Statement::from_sql_and_values(
            DatabaseBackend::Sqlite,
            r#"
                DELETE FROM service_group_instance
                WHERE group_id = ? AND instance_id = ? AND region_id = ? AND zone_id = ?
            "#,
            vec![
                group_id.to_string().into(),
                instance_id.into(),
                region_id.into(),
                zone_id.into(),
            ],
        );

        let result = self.db.execute(stmt).await?;
        Ok(result.rows_affected() > 0)
    }

    /// 获取分组的所有绑定实例
    pub async fn get_by_group(&self, group_id: i64) -> Result<Vec<GroupInstance>> {
        let stmt = Statement::from_sql_and_values(
            DatabaseBackend::Sqlite,
            r#"
                SELECT id, group_id, region_id, zone_id, service_id, instance_id, binding_type, operator_id, created_at
                FROM service_group_instance
                WHERE group_id = ?
            "#,
            vec![group_id.to_string().into()],
        );

        let result = self.db.query_all(stmt).await?;

        let bindings = result
            .iter()
            .map(|row| {
                let binding_type_str: String = row.try_get("", "binding_type").unwrap_or_else(|_| "auto".to_string());
                let binding_type = match binding_type_str.as_str() {
                    "manual" => Some(BindingType::Manual),
                    "auto" => Some(BindingType::Auto),
                    _ => None,
                };

                GroupInstance {
                    id: row.try_get("", "id").ok(),
                    group_id: row.try_get::<String>("", "group_id").ok().and_then(|s| s.parse().ok()).unwrap_or(0),
                    region_id: row.try_get("", "region_id").unwrap_or_default(),
                    zone_id: row.try_get("", "zone_id").unwrap_or_default(),
                    service_id: row.try_get("", "service_id").unwrap_or_default(),
                    instance_id: row.try_get("", "instance_id").unwrap_or_default(),
                    binding_type,
                    operator_id: row.try_get("", "operator_id").ok(),
                    created_at: row.try_get("", "created_at").ok(),
                }
            })
            .collect();

        Ok(bindings)
    }

    /// 获取实例的所有分组绑定
    pub async fn get_by_instance(
        &self,
        instance_id: &str,
        region_id: &str,
        zone_id: &str,
    ) -> Result<Vec<GroupInstance>> {
        let stmt = Statement::from_sql_and_values(
            DatabaseBackend::Sqlite,
            r#"
                SELECT id, group_id, region_id, zone_id, service_id, instance_id, binding_type, operator_id, created_at
                FROM service_group_instance
                WHERE instance_id = ? AND region_id = ? AND zone_id = ?
            "#,
            vec![instance_id.into(), region_id.into(), zone_id.into()],
        );

        let result = self.db.query_all(stmt).await?;

        let bindings = result
            .iter()
            .map(|row| {
                let binding_type_str: String = row.try_get("", "binding_type").unwrap_or_else(|_| "auto".to_string());
                let binding_type = match binding_type_str.as_str() {
                    "manual" => Some(BindingType::Manual),
                    "auto" => Some(BindingType::Auto),
                    _ => None,
                };

                GroupInstance {
                    id: row.try_get("", "id").ok(),
                    group_id: row.try_get::<String>("", "group_id").ok().and_then(|s| s.parse().ok()).unwrap_or(0),
                    region_id: row.try_get("", "region_id").unwrap_or_default(),
                    zone_id: row.try_get("", "zone_id").unwrap_or_default(),
                    service_id: row.try_get("", "service_id").unwrap_or_default(),
                    instance_id: row.try_get("", "instance_id").unwrap_or_default(),
                    binding_type,
                    operator_id: row.try_get("", "operator_id").ok(),
                    created_at: row.try_get("", "created_at").ok(),
                }
            })
            .collect();

        Ok(bindings)
    }

    /// 批量插入绑定 (用于服务实例批量添加)
    pub async fn batch_insert(&self, bindings: &[GroupInstance]) -> Result<usize> {
        let mut count = 0;
        for binding in bindings {
            if self.insert(binding).await.is_ok() {
                count += 1;
            }
        }
        Ok(count)
    }

    /// 删除分组的所有绑定
    pub async fn delete_all_by_group(&self, group_id: i64) -> Result<u64> {
        let stmt = Statement::from_sql_and_values(
            DatabaseBackend::Sqlite,
            r#"DELETE FROM service_group_instance WHERE group_id = ?"#,
            vec![group_id.to_string().into()],
        );

        let result = self.db.execute(stmt).await?;
        Ok(result.rows_affected())
    }
}

#[cfg(test)]
mod tests {
    

    // 测试需要实际数据库连接,这里仅作示例
    #[tokio::test]
    #[ignore] // 需要数据库环境
    async fn test_insert_and_get() {
        // 实际测试代码需要数据库连接
    }
}
