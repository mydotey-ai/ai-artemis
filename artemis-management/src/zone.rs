//! Zone management operations

use crate::dao::ZoneOperationDao;
use crate::db::Database;
use artemis_core::model::{ZoneOperation, ZoneOperationRecord};
use chrono::Utc;
use dashmap::DashMap;
use std::sync::Arc;
use tracing::info;

/// Zone 管理器
#[derive(Clone)]
pub struct ZoneManager {
    /// Zone 操作存储: zone_key (zone_id:region_id) -> ZoneOperationRecord
    operations: Arc<DashMap<String, ZoneOperationRecord>>,

    /// 可选数据库支持 - 用于持久化
    database: Option<Arc<Database>>,
}

impl Default for ZoneManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ZoneManager {
    pub fn new() -> Self {
        Self::with_database(None)
    }

    pub fn with_database(database: Option<Arc<Database>>) -> Self {
        Self { operations: Arc::new(DashMap::new()), database }
    }

    /// 拉出整个 Zone (批量下线)
    pub fn pull_out_zone(
        &self,
        zone_id: &str,
        region_id: &str,
        operator_id: String,
    ) -> anyhow::Result<()> {
        let zone_key = Self::zone_key(zone_id, region_id);

        info!("Pull-out zone: {} by operator: {}", zone_key, operator_id);

        let record = ZoneOperationRecord {
            zone_id: zone_id.to_string(),
            region_id: region_id.to_string(),
            operation: ZoneOperation::PullOut,
            operator_id,
            operation_time: Utc::now().timestamp(),
        };

        self.operations.insert(zone_key, record.clone());

        // 持久化到数据库
        if let Some(db) = &self.database {
            let dao = ZoneOperationDao::new(db.conn().clone());
            let record_clone = record.clone();
            tokio::spawn(async move {
                if let Err(e) = dao.insert_operation(&record_clone).await {
                    tracing::error!("Failed to persist zone operation to database: {}", e);
                }
            });
        }

        Ok(())
    }

    /// 拉入整个 Zone (批量恢复)
    pub fn pull_in_zone(
        &self,
        zone_id: &str,
        region_id: &str,
        operator_id: String,
    ) -> anyhow::Result<()> {
        let zone_key = Self::zone_key(zone_id, region_id);

        info!("Pull-in zone: {} by operator: {}", zone_key, operator_id);

        // 移除拉出记录
        self.operations.remove(&zone_key);

        // 从数据库删除
        if let Some(db) = &self.database {
            let dao = ZoneOperationDao::new(db.conn().clone());
            let zone_id_owned = zone_id.to_string();
            let region_id_owned = region_id.to_string();
            tokio::spawn(async move {
                if let Err(e) = dao.delete_operation(&zone_id_owned, &region_id_owned).await {
                    tracing::error!("Failed to delete zone operation from database: {}", e);
                }
            });
        }

        Ok(())
    }

    /// 查询 Zone 是否被拉出
    pub fn is_zone_down(&self, zone_id: &str, region_id: &str) -> bool {
        let zone_key = Self::zone_key(zone_id, region_id);
        self.operations.contains_key(&zone_key)
    }

    /// 获取 Zone 状态
    pub fn get_zone_status(&self, zone_id: &str, region_id: &str) -> Option<ZoneOperationRecord> {
        let zone_key = Self::zone_key(zone_id, region_id);
        self.operations.get(&zone_key).map(|r| r.clone())
    }

    /// 列出所有 Zone 操作
    pub fn list_operations(&self, region_id: Option<&str>) -> Vec<ZoneOperationRecord> {
        self.operations
            .iter()
            .filter(
                |entry| {
                    if let Some(rid) = region_id { entry.value().region_id == rid } else { true }
                },
            )
            .map(|entry| entry.value().clone())
            .collect()
    }

    // Helper: Generate zone key
    fn zone_key(zone_id: &str, region_id: &str) -> String {
        format!("{}:{}", zone_id, region_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pull_out_pull_in() {
        let manager = ZoneManager::new();

        // Pull out
        manager.pull_out_zone("zone-1", "us-east", "operator-1".to_string()).unwrap();
        assert!(manager.is_zone_down("zone-1", "us-east"));

        // Pull in
        manager.pull_in_zone("zone-1", "us-east", "operator-1".to_string()).unwrap();
        assert!(!manager.is_zone_down("zone-1", "us-east"));
    }

    #[test]
    fn test_list_operations() {
        let manager = ZoneManager::new();

        manager.pull_out_zone("zone-1", "us-east", "operator-1".to_string()).unwrap();
        manager.pull_out_zone("zone-2", "us-west", "operator-2".to_string()).unwrap();

        let all_ops = manager.list_operations(None);
        assert_eq!(all_ops.len(), 2);

        let east_ops = manager.list_operations(Some("us-east"));
        assert_eq!(east_ops.len(), 1);
        assert_eq!(east_ops[0].zone_id, "zone-1");
    }
}
