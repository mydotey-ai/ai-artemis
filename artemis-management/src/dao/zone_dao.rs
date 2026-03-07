use crate::model::{ZoneOperation, ZoneOperationRecord};
use sea_orm::sea_query::Value;
use sea_orm::{ConnectionTrait, DatabaseConnection, Statement};

pub struct ZoneOperationDao {
    conn: DatabaseConnection,
}

impl ZoneOperationDao {
    pub fn new(conn: DatabaseConnection) -> Self {
        Self { conn }
    }

    /// 插入 Zone 操作 (兼容 Java 表名 service_zone)
    pub async fn insert_operation(&self, record: &ZoneOperationRecord) -> anyhow::Result<()> {
        let operation_str = match record.operation {
            ZoneOperation::PullIn => "pullin",
            ZoneOperation::PullOut => "pullout",
        };

        let stmt = Statement::from_sql_and_values(
            self.conn.get_database_backend(),
            r#"
            INSERT INTO service_zone (SERVICE_ID, ZONE_ID, REGION_ID, OPERATION)
            VALUES (?, ?, ?, ?)
            ON CONFLICT(SERVICE_ID, REGION_ID, ZONE_ID, OPERATION) DO UPDATE SET OPERATION = excluded.OPERATION
            "#,
            vec![
                Value::from(""), // SERVICE_ID 暂时为空
                Value::from(&record.zone_id),
                Value::from(&record.region_id),
                Value::from(operation_str),
            ],
        );

        self.conn.execute(stmt).await?;
        Ok(())
    }

    /// 删除 Zone 操作
    pub async fn delete_operation(&self, zone_id: &str, region_id: &str) -> anyhow::Result<()> {
        let stmt = Statement::from_sql_and_values(
            self.conn.get_database_backend(),
            "DELETE FROM service_zone WHERE ZONE_ID = ? AND REGION_ID = ?",
            vec![Value::from(zone_id), Value::from(region_id)],
        );
        self.conn.execute(stmt).await?;
        Ok(())
    }

    /// 获取 Zone 操作
    pub async fn get_operation(
        &self,
        zone_id: &str,
        region_id: &str,
    ) -> anyhow::Result<Option<ZoneOperationRecord>> {
        let stmt = Statement::from_sql_and_values(
            self.conn.get_database_backend(),
            r#"
            SELECT ZONE_ID, REGION_ID, OPERATION
            FROM service_zone
            WHERE ZONE_ID = ? AND REGION_ID = ?
            "#,
            vec![Value::from(zone_id), Value::from(region_id)],
        );

        let result = self.conn.query_one(stmt).await?;

        match result {
            Some(row) => {
                let operation_str: String = row.try_get("", "OPERATION")?;
                let operation = match operation_str.as_str() {
                    "pullin" => ZoneOperation::PullIn,
                    _ => ZoneOperation::PullOut,
                };

                Ok(Some(ZoneOperationRecord {
                    zone_id: row.try_get("", "ZONE_ID")?,
                    region_id: row.try_get("", "REGION_ID")?,
                    operation,
                    operator_id: String::new(), // Java 表没有这个字段
                    operation_time: 0, // Java 表没有这个字段
                }))
            }
            None => Ok(None),
        }
    }

    /// 列出所有 Zone 操作
    pub async fn list_operations(&self) -> anyhow::Result<Vec<ZoneOperationRecord>> {
        let stmt = Statement::from_sql_and_values(
            self.conn.get_database_backend(),
            r#"
            SELECT ZONE_ID, REGION_ID, OPERATION
            FROM service_zone
            "#,
            vec![],
        );

        let rows = self.conn.query_all(stmt).await?;

        Ok(rows
            .into_iter()
            .map(|row| {
                let operation_str: String = row.try_get("", "OPERATION").unwrap_or_default();
                let operation = match operation_str.as_str() {
                    "pullin" => ZoneOperation::PullIn,
                    _ => ZoneOperation::PullOut,
                };

                ZoneOperationRecord {
                    zone_id: row.try_get("", "ZONE_ID").unwrap_or_default(),
                    region_id: row.try_get("", "REGION_ID").unwrap_or_default(),
                    operation,
                    operator_id: String::new(),
                    operation_time: 0,
                }
            })
            .collect())
    }
}
