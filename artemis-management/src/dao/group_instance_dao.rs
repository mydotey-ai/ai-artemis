use crate::model::GroupInstance;
use anyhow::Result;
use sea_orm::{ConnectionTrait, DatabaseBackend, DbConn, Statement};

/// GroupInstance DAO - 分组实例绑定关系持久化 (兼容 Java 表结构)
pub struct GroupInstanceDao {
    db: DbConn,
}

impl GroupInstanceDao {
    pub fn new(db: DbConn) -> Self {
        Self { db }
    }

    /// 插入分组实例绑定 (兼容 Java 表结构)
    pub async fn insert(&self, binding: &GroupInstance) -> Result<i64> {
        let stmt = Statement::from_sql_and_values(
            DatabaseBackend::Sqlite,
            r#"
                INSERT INTO service_group_instance (GROUP_ID, INSTANCE_ID)
                VALUES (?, ?)
            "#,
            vec![
                binding.group_id.into(),
                binding.instance_id.clone().into(),
            ],
        );

        self.db.execute(stmt).await?;
        Ok(1)
    }

    /// 删除分组实例绑定
    pub async fn delete(
        &self,
        group_id: i64,
        instance_id: &str,
        _region_id: &str,
        _zone_id: &str,
    ) -> Result<bool> {
        let stmt = Statement::from_sql_and_values(
            DatabaseBackend::Sqlite,
            r#"
                DELETE FROM service_group_instance
                WHERE GROUP_ID = ? AND INSTANCE_ID = ?
            "#,
            vec![group_id.into(), instance_id.into()],
        );

        let result = self.db.execute(stmt).await?;
        Ok(result.rows_affected() > 0)
    }

    /// 获取分组的所有绑定实例
    pub async fn get_by_group(&self, group_id: i64) -> Result<Vec<GroupInstance>> {
        let stmt = Statement::from_sql_and_values(
            DatabaseBackend::Sqlite,
            r#"
                SELECT id, GROUP_ID, INSTANCE_ID
                FROM service_group_instance
                WHERE GROUP_ID = ?
            "#,
            vec![group_id.into()],
        );

        let result = self.db.query_all(stmt).await?;

        let bindings = result
            .iter()
            .map(|row| {
                GroupInstance {
                    id: row.try_get("", "id").ok(),
                    group_id: row.try_get::<i64>("", "GROUP_ID").unwrap_or(0),
                    region_id: String::new(),
                    zone_id: String::new(),
                    service_id: String::new(),
                    instance_id: row.try_get("", "INSTANCE_ID").unwrap_or_default(),
                    binding_type: None,
                    operator_id: None,
                    created_at: None,
                }
            })
            .collect();

        Ok(bindings)
    }

    /// 获取实例的所有分组绑定
    pub async fn get_by_instance(
        &self,
        instance_id: &str,
        _region_id: &str,
        _zone_id: &str,
    ) -> Result<Vec<GroupInstance>> {
        let stmt = Statement::from_sql_and_values(
            DatabaseBackend::Sqlite,
            r#"
                SELECT id, GROUP_ID, INSTANCE_ID
                FROM service_group_instance
                WHERE INSTANCE_ID = ?
            "#,
            vec![instance_id.into()],
        );

        let result = self.db.query_all(stmt).await?;

        let bindings = result
            .iter()
            .map(|row| {
                GroupInstance {
                    id: row.try_get("", "id").ok(),
                    group_id: row.try_get::<i64>("", "GROUP_ID").unwrap_or(0),
                    region_id: String::new(),
                    zone_id: String::new(),
                    service_id: String::new(),
                    instance_id: row.try_get("", "INSTANCE_ID").unwrap_or_default(),
                    binding_type: None,
                    operator_id: None,
                    created_at: None,
                }
            })
            .collect();

        Ok(bindings)
    }

    /// 批量插入绑定
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
            "DELETE FROM service_group_instance WHERE GROUP_ID = ?",
            vec![group_id.into()],
        );

        let result = self.db.execute(stmt).await?;
        Ok(result.rows_affected())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::BindingType;
    use sea_orm::{Database, DatabaseConnection};

    async fn create_test_db() -> DatabaseConnection {
        let db = Database::connect("sqlite::memory:")
            .await
            .expect("Failed to create in-memory database");

        let create_table_sql = r#"
            CREATE TABLE IF NOT EXISTS service_group_instance (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                GROUP_ID INTEGER NOT NULL DEFAULT 0,
                INSTANCE_ID TEXT NOT NULL DEFAULT '',
                CREATE_TIME DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                DataChange_LastTime TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                UNIQUE(GROUP_ID, INSTANCE_ID)
            )
        "#;

        let stmt = Statement::from_string(DatabaseBackend::Sqlite, create_table_sql.to_string());
        db.execute(stmt).await.expect("Failed to create table");

        db
    }

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

        let binding = create_test_binding(1, "inst-1");
        let result = dao.insert(&binding).await;
        assert!(result.is_ok(), "插入绑定应该成功");

        let bindings = dao.get_by_group(1).await.unwrap();
        assert_eq!(bindings.len(), 1, "应该查询到 1 个绑定");
        assert_eq!(bindings[0].instance_id, "inst-1");
        assert_eq!(bindings[0].group_id, 1);
    }

    #[tokio::test]
    async fn test_get_by_instance() {
        let db = create_test_db().await;
        let dao = GroupInstanceDao::new(db);

        let binding1 = create_test_binding(1, "inst-100");
        let binding2 = create_test_binding(2, "inst-100");

        dao.insert(&binding1).await.unwrap();
        dao.insert(&binding2).await.unwrap();

        let bindings = dao.get_by_instance("inst-100", "us-east", "zone-1").await.unwrap();
        assert_eq!(bindings.len(), 2, "实例应该属于 2 个分组");
    }

    #[tokio::test]
    async fn test_delete_binding() {
        let db = create_test_db().await;
        let dao = GroupInstanceDao::new(db);

        let binding = create_test_binding(1, "inst-200");
        dao.insert(&binding).await.unwrap();

        let deleted = dao.delete(1, "inst-200", "us-east", "zone-1").await.unwrap();
        assert!(deleted, "删除应该成功");

        let bindings = dao.get_by_group(1).await.unwrap();
        assert_eq!(bindings.len(), 0, "分组应该没有绑定");
    }

    #[tokio::test]
    async fn test_batch_insert() {
        let db = create_test_db().await;
        let dao = GroupInstanceDao::new(db);

        let bindings = vec![
            create_test_binding(1, "inst-1"),
            create_test_binding(1, "inst-2"),
            create_test_binding(1, "inst-3"),
        ];

        let count = dao.batch_insert(&bindings).await.unwrap();
        assert_eq!(count, 3, "应该插入 3 个绑定");

        let result = dao.get_by_group(1).await.unwrap();
        assert_eq!(result.len(), 3, "分组应该有 3 个绑定");
    }

    #[tokio::test]
    async fn test_delete_all_by_group() {
        let db = create_test_db().await;
        let dao = GroupInstanceDao::new(db);

        let bindings = vec![
            create_test_binding(1, "inst-1"),
            create_test_binding(1, "inst-2"),
            create_test_binding(1, "inst-3"),
        ];
        dao.batch_insert(&bindings).await.unwrap();

        let deleted = dao.delete_all_by_group(1).await.unwrap();
        assert_eq!(deleted, 3, "应该删除 3 个绑定");

        let result = dao.get_by_group(1).await.unwrap();
        assert_eq!(result.len(), 0, "分组应该没有绑定");
    }
}
