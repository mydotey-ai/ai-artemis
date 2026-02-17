use anyhow::Result;
use crate::model::{BindingType, GroupInstance};
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
                binding.binding_type.as_ref().map(|_| 0).into(),  // port optional
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
            vec![group_id.to_string().into(), instance_id.into(), region_id.into(), zone_id.into()],
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
                let binding_type_str: String =
                    row.try_get("", "binding_type").unwrap_or_else(|_| "auto".to_string());
                let binding_type = match binding_type_str.as_str() {
                    "manual" => Some(BindingType::Manual),
                    "auto" => Some(BindingType::Auto),
                    _ => None,
                };

                GroupInstance {
                    id: row.try_get("", "id").ok(),
                    group_id: row
                        .try_get::<String>("", "group_id")
                        .ok()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0),
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
                let binding_type_str: String =
                    row.try_get("", "binding_type").unwrap_or_else(|_| "auto".to_string());
                let binding_type = match binding_type_str.as_str() {
                    "manual" => Some(BindingType::Manual),
                    "auto" => Some(BindingType::Auto),
                    _ => None,
                };

                GroupInstance {
                    id: row.try_get("", "id").ok(),
                    group_id: row
                        .try_get::<String>("", "group_id")
                        .ok()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0),
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
    use super::*;
    use sea_orm::{Database, DatabaseConnection};

    /// 创建内存 SQLite 测试数据库
    async fn create_test_db() -> DatabaseConnection {
        let db = Database::connect("sqlite::memory:")
            .await
            .expect("Failed to create in-memory database");

        // 创建 service_group_instance 表
        let create_table_sql = r#"
            CREATE TABLE IF NOT EXISTS service_group_instance (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                group_id TEXT NOT NULL,
                region_id TEXT NOT NULL,
                zone_id TEXT NOT NULL,
                service_id TEXT NOT NULL,
                instance_id TEXT NOT NULL,
                ip TEXT,
                port INTEGER,
                binding_type TEXT NOT NULL DEFAULT 'auto' CHECK(binding_type IN ('manual', 'auto')),
                operator_id TEXT,
                created_at BIGINT NOT NULL,
                UNIQUE(group_id, instance_id, region_id, zone_id)
            )
        "#;

        let stmt = Statement::from_string(DatabaseBackend::Sqlite, create_table_sql.to_string());
        db.execute(stmt).await.expect("Failed to create table");

        db
    }

    /// 创建测试 GroupInstance
    fn create_test_binding(group_id: i64, instance_id: &str) -> GroupInstance {
        GroupInstance {
            id: None,
            group_id,
            region_id: "us-east".to_string(),
            zone_id: "zone-1".to_string(),
            service_id: "test-service".to_string(),
            instance_id: instance_id.to_string(),
            binding_type: Some(BindingType::Manual),
            operator_id: Some("admin".to_string()),
            created_at: Some(chrono::Utc::now().timestamp()),
        }
    }

    #[tokio::test]
    async fn test_insert_and_get() {
        let db = create_test_db().await;
        let dao = GroupInstanceDao::new(db);

        // 1. 插入绑定
        let binding = create_test_binding(1, "inst-1");
        let result = dao.insert(&binding).await;
        assert!(result.is_ok(), "插入绑定应该成功");

        // 2. 按分组查询
        let bindings = dao.get_by_group(1).await.unwrap();
        assert_eq!(bindings.len(), 1, "应该查询到 1 个绑定");
        assert_eq!(bindings[0].instance_id, "inst-1");
        assert_eq!(bindings[0].group_id, 1);
    }

    #[tokio::test]
    async fn test_get_by_instance() {
        let db = create_test_db().await;
        let dao = GroupInstanceDao::new(db);

        // 插入 2 个绑定到不同分组
        let binding1 = create_test_binding(1, "inst-100");
        let binding2 = create_test_binding(2, "inst-100");

        dao.insert(&binding1).await.unwrap();
        dao.insert(&binding2).await.unwrap();

        // 按实例查询
        let bindings = dao.get_by_instance("inst-100", "us-east", "zone-1").await.unwrap();
        assert_eq!(bindings.len(), 2, "实例应该属于 2 个分组");
    }

    #[tokio::test]
    async fn test_delete_binding() {
        let db = create_test_db().await;
        let dao = GroupInstanceDao::new(db);

        // 插入
        let binding = create_test_binding(1, "inst-200");
        dao.insert(&binding).await.unwrap();

        // 删除
        let deleted = dao.delete(1, "inst-200", "us-east", "zone-1").await.unwrap();
        assert!(deleted, "删除应该成功");

        // 验证已删除
        let bindings = dao.get_by_group(1).await.unwrap();
        assert_eq!(bindings.len(), 0, "分组应该没有绑定");
    }

    #[tokio::test]
    async fn test_batch_insert() {
        let db = create_test_db().await;
        let dao = GroupInstanceDao::new(db);

        // 批量插入 3 个绑定
        let bindings = vec![
            create_test_binding(1, "inst-1"),
            create_test_binding(1, "inst-2"),
            create_test_binding(1, "inst-3"),
        ];

        let count = dao.batch_insert(&bindings).await.unwrap();
        assert_eq!(count, 3, "应该插入 3 个绑定");

        // 验证
        let result = dao.get_by_group(1).await.unwrap();
        assert_eq!(result.len(), 3, "分组应该有 3 个绑定");
    }

    #[tokio::test]
    async fn test_delete_all_by_group() {
        let db = create_test_db().await;
        let dao = GroupInstanceDao::new(db);

        // 插入 3 个绑定
        let bindings = vec![
            create_test_binding(1, "inst-1"),
            create_test_binding(1, "inst-2"),
            create_test_binding(1, "inst-3"),
        ];
        dao.batch_insert(&bindings).await.unwrap();

        // 删除分组所有绑定
        let deleted = dao.delete_all_by_group(1).await.unwrap();
        assert_eq!(deleted, 3, "应该删除 3 个绑定");

        // 验证
        let result = dao.get_by_group(1).await.unwrap();
        assert_eq!(result.len(), 0, "分组应该没有绑定");
    }

    #[tokio::test]
    async fn test_binding_type_auto() {
        let db = create_test_db().await;
        let dao = GroupInstanceDao::new(db);

        // 插入 auto 类型绑定
        let mut binding = create_test_binding(1, "inst-auto");
        binding.binding_type = Some(BindingType::Auto);

        dao.insert(&binding).await.unwrap();

        // 验证
        let bindings = dao.get_by_group(1).await.unwrap();
        assert_eq!(bindings.len(), 1);
        assert_eq!(bindings[0].binding_type, Some(BindingType::Auto));
    }

    #[tokio::test]
    async fn test_multiple_groups() {
        let db = create_test_db().await;
        let dao = GroupInstanceDao::new(db);

        // 分组 1: 2 个实例
        dao.insert(&create_test_binding(1, "inst-1")).await.unwrap();
        dao.insert(&create_test_binding(1, "inst-2")).await.unwrap();

        // 分组 2: 3 个实例
        dao.insert(&create_test_binding(2, "inst-3")).await.unwrap();
        dao.insert(&create_test_binding(2, "inst-4")).await.unwrap();
        dao.insert(&create_test_binding(2, "inst-5")).await.unwrap();

        // 验证分组 1
        let group1 = dao.get_by_group(1).await.unwrap();
        assert_eq!(group1.len(), 2);

        // 验证分组 2
        let group2 = dao.get_by_group(2).await.unwrap();
        assert_eq!(group2.len(), 3);
    }
}
