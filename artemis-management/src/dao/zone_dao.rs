use artemis_core::model::{ZoneOperation, ZoneOperationRecord};
use sqlx::{Pool, Any, Row};

pub struct ZoneOperationDao {
    pool: Pool<Any>,
}

impl ZoneOperationDao {
    pub fn new(pool: Pool<Any>) -> Self {
        Self { pool }
    }

    /// 插入 Zone 操作
    pub async fn insert_operation(&self, record: &ZoneOperationRecord) -> anyhow::Result<()> {
        let operation_str = match record.operation {
            ZoneOperation::PullIn => "pullin",
            ZoneOperation::PullOut => "pullout",
        };

        sqlx::query(
            r#"
            INSERT INTO zone_operation (zone_id, region_id, operation, operator_id, operation_time)
            VALUES (?, ?, ?, ?, ?)
            ON CONFLICT(zone_id, region_id) DO UPDATE SET operation = excluded.operation, operator_id = excluded.operator_id, operation_time = excluded.operation_time
            "#
        )
        .bind(&record.zone_id)
        .bind(&record.region_id)
        .bind(operation_str)
        .bind(&record.operator_id)
        .bind(record.operation_time)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 删除 Zone 操作
    pub async fn delete_operation(&self, zone_id: &str, region_id: &str) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM zone_operation WHERE zone_id = ? AND region_id = ?")
            .bind(zone_id)
            .bind(region_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// 获取 Zone 操作
    pub async fn get_operation(&self, zone_id: &str, region_id: &str) -> anyhow::Result<Option<ZoneOperationRecord>> {
        let row = sqlx::query(
            r#"
            SELECT zone_id, region_id, operation, operator_id, operation_time
            FROM zone_operation
            WHERE zone_id = ? AND region_id = ?
            "#
        )
        .bind(zone_id)
        .bind(region_id)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => {
                let operation_str: String = row.get("operation");
                let operation = match operation_str.as_str() {
                    "pullin" => ZoneOperation::PullIn,
                    _ => ZoneOperation::PullOut,
                };

                Ok(Some(ZoneOperationRecord {
                    zone_id: row.get("zone_id"),
                    region_id: row.get("region_id"),
                    operation,
                    operator_id: row.get("operator_id"),
                    operation_time: row.get("operation_time"),
                }))
            }
            None => Ok(None),
        }
    }

    /// 列出所有 Zone 操作
    pub async fn list_operations(&self) -> anyhow::Result<Vec<ZoneOperationRecord>> {
        let rows = sqlx::query(
            r#"
            SELECT zone_id, region_id, operation, operator_id, operation_time
            FROM zone_operation
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| {
                let operation_str: String = row.get("operation");
                let operation = match operation_str.as_str() {
                    "pullin" => ZoneOperation::PullIn,
                    _ => ZoneOperation::PullOut,
                };

                ZoneOperationRecord {
                    zone_id: row.get("zone_id"),
                    region_id: row.get("region_id"),
                    operation,
                    operator_id: row.get("operator_id"),
                    operation_time: row.get("operation_time"),
                }
            })
            .collect())
    }
}
