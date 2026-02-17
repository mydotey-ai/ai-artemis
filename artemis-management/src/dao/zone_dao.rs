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

    /// 插入 Zone 操作
    pub async fn insert_operation(&self, record: &ZoneOperationRecord) -> anyhow::Result<()> {
        let operation_str = match record.operation {
            ZoneOperation::PullIn => "pullin",
            ZoneOperation::PullOut => "pullout",
        };

        let stmt = Statement::from_sql_and_values(
            self.conn.get_database_backend(),
            r#"
            INSERT INTO zone_operation (zone_id, region_id, operation, operator_id, operation_time)
            VALUES (?, ?, ?, ?, ?)
            ON CONFLICT(zone_id, region_id) DO UPDATE SET operation = excluded.operation, operator_id = excluded.operator_id, operation_time = excluded.operation_time
            "#,
            vec![
                Value::from(&record.zone_id),
                Value::from(&record.region_id),
                Value::from(operation_str),
                Value::from(&record.operator_id),
                Value::from(record.operation_time),
            ],
        );

        self.conn.execute(stmt).await?;
        Ok(())
    }

    /// 删除 Zone 操作
    pub async fn delete_operation(&self, zone_id: &str, region_id: &str) -> anyhow::Result<()> {
        let stmt = Statement::from_sql_and_values(
            self.conn.get_database_backend(),
            "DELETE FROM zone_operation WHERE zone_id = ? AND region_id = ?",
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
            SELECT zone_id, region_id, operation, operator_id, operation_time
            FROM zone_operation
            WHERE zone_id = ? AND region_id = ?
            "#,
            vec![Value::from(zone_id), Value::from(region_id)],
        );

        let result = self.conn.query_one(stmt).await?;

        match result {
            Some(row) => {
                let operation_str: String = row.try_get("", "operation")?;
                let operation = match operation_str.as_str() {
                    "pullin" => ZoneOperation::PullIn,
                    _ => ZoneOperation::PullOut,
                };

                Ok(Some(ZoneOperationRecord {
                    zone_id: row.try_get("", "zone_id")?,
                    region_id: row.try_get("", "region_id")?,
                    operation,
                    operator_id: row.try_get("", "operator_id")?,
                    operation_time: row.try_get("", "operation_time")?,
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
            SELECT zone_id, region_id, operation, operator_id, operation_time
            FROM zone_operation
            "#,
            vec![],
        );

        let rows = self.conn.query_all(stmt).await?;

        Ok(rows
            .into_iter()
            .map(|row| {
                let operation_str: String = row.try_get("", "operation").unwrap_or_default();
                let operation = match operation_str.as_str() {
                    "pullin" => ZoneOperation::PullIn,
                    _ => ZoneOperation::PullOut,
                };

                ZoneOperationRecord {
                    zone_id: row.try_get("", "zone_id").unwrap_or_default(),
                    region_id: row.try_get("", "region_id").unwrap_or_default(),
                    operation,
                    operator_id: row.try_get("", "operator_id").unwrap_or_default(),
                    operation_time: row.try_get("", "operation_time").unwrap_or_default(),
                }
            })
            .collect())
    }
}
