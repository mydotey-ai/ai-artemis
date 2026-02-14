//! Instance management operations - Pull-in/Pull-out functionality
//!
//! This module provides instance-level and server-level pull-in/pull-out operations.
//! These operations allow operators to manually control instance availability without
//! affecting the registration state.

use artemis_core::model::{
    InstanceKey, InstanceOperation, InstanceOperationRecord, ServerOperation,
};
use dashmap::DashMap;
use std::sync::Arc;
use tracing::info;

/// 实例管理器 - 提供实例拉入/拉出功能
#[derive(Clone)]
pub struct InstanceManager {
    /// 实例操作存储: instance_key_string -> InstanceOperation
    instance_operations: Arc<DashMap<String, InstanceOperationRecord>>,
    /// 服务器操作存储: server_key (server_id:region_id) -> ServerOperation
    server_operations: Arc<DashMap<String, ServerOperation>>,
}

impl Default for InstanceManager {
    fn default() -> Self {
        Self::new()
    }
}

impl InstanceManager {
    pub fn new() -> Self {
        Self {
            instance_operations: Arc::new(DashMap::new()),
            server_operations: Arc::new(DashMap::new()),
        }
    }

    // ========== 实例操作 ==========

    /// 拉出实例 (下线)
    pub fn pull_out_instance(
        &self,
        key: &InstanceKey,
        operator_id: String,
        operation_complete: bool,
    ) -> anyhow::Result<()> {
        let key_str = Self::instance_key_string(key);

        info!(
            "Pull-out instance: {} by operator: {}, complete: {}",
            key_str, operator_id, operation_complete
        );

        let record = InstanceOperationRecord {
            instance_key: key.clone(),
            operation: InstanceOperation::PullOut,
            operation_complete,
            operator_id,
            token: None,
        };

        self.instance_operations.insert(key_str, record);
        Ok(())
    }

    /// 拉入实例 (恢复)
    pub fn pull_in_instance(
        &self,
        key: &InstanceKey,
        operator_id: String,
        operation_complete: bool,
    ) -> anyhow::Result<()> {
        let key_str = Self::instance_key_string(key);

        info!(
            "Pull-in instance: {} by operator: {}, complete: {}",
            key_str, operator_id, operation_complete
        );

        if operation_complete {
            // 完成拉入操作 = 移除拉出记录
            self.instance_operations.remove(&key_str);
            info!("Instance operation removed: {}", key_str);
        } else {
            // 开始拉入操作 (标记为拉入中)
            let record = InstanceOperationRecord {
                instance_key: key.clone(),
                operation: InstanceOperation::PullIn,
                operation_complete: false,
                operator_id,
                token: None,
            };
            self.instance_operations.insert(key_str, record);
        }

        Ok(())
    }

    /// 查询实例是否被拉出 (用于发现服务过滤)
    pub fn is_instance_down(&self, key: &InstanceKey) -> bool {
        let key_str = Self::instance_key_string(key);

        if let Some(record) = self.instance_operations.get(&key_str) {
            // 只有 operation_complete=true 的 PullOut 操作才算真正下线
            return record.operation == InstanceOperation::PullOut && record.operation_complete;
        }

        false
    }

    /// 获取实例的操作列表
    pub fn get_instance_operations(&self, key: &InstanceKey) -> Vec<InstanceOperation> {
        let key_str = Self::instance_key_string(key);

        if let Some(record) = self.instance_operations.get(&key_str) {
            vec![record.operation.clone()]
        } else {
            vec![]
        }
    }

    // ========== 服务器操作 ==========

    /// 拉出整台服务器 (批量下线)
    pub fn pull_out_server(
        &self,
        server_id: &str,
        region_id: &str,
        operator_id: String,
        operation_complete: bool,
    ) -> anyhow::Result<()> {
        let server_key = Self::server_key(server_id, region_id);

        info!(
            "Pull-out server: {} by operator: {}, complete: {}",
            server_key, operator_id, operation_complete
        );

        if operation_complete {
            self.server_operations.insert(server_key, ServerOperation::PullOut);
        }

        Ok(())
    }

    /// 拉入整台服务器 (批量恢复)
    pub fn pull_in_server(
        &self,
        server_id: &str,
        region_id: &str,
        operator_id: String,
        operation_complete: bool,
    ) -> anyhow::Result<()> {
        let server_key = Self::server_key(server_id, region_id);

        info!(
            "Pull-in server: {} by operator: {}, complete: {}",
            server_key, operator_id, operation_complete
        );

        if operation_complete {
            // 完成拉入操作 = 移除拉出记录
            self.server_operations.remove(&server_key);
            info!("Server operation removed: {}", server_key);
        }

        Ok(())
    }

    /// 查询服务器是否被拉出
    pub fn is_server_down(&self, server_id: &str, region_id: &str) -> bool {
        let server_key = Self::server_key(server_id, region_id);

        if let Some(op) = self.server_operations.get(&server_key) {
            return *op == ServerOperation::PullOut;
        }

        false
    }

    // ========== 辅助方法 ==========

    /// 生成实例键字符串: service_id:instance_id:region_id
    fn instance_key_string(key: &InstanceKey) -> String {
        format!("{}:{}:{}", key.service_id, key.instance_id, key.region_id)
    }

    /// 生成服务器键字符串: server_id:region_id
    fn server_key(server_id: &str, region_id: &str) -> String {
        format!("{}:{}", server_id, region_id)
    }

    /// 获取当前所有被拉出的实例数量
    pub fn down_instance_count(&self) -> usize {
        self.instance_operations
            .iter()
            .filter(|entry| {
                entry.value().operation == InstanceOperation::PullOut
                    && entry.value().operation_complete
            })
            .count()
    }

    /// 获取当前所有被拉出的服务器数量
    pub fn down_server_count(&self) -> usize {
        self.server_operations
            .iter()
            .filter(|entry| *entry.value() == ServerOperation::PullOut)
            .count()
    }

    /// 清理所有操作记录 (用于测试)
    #[cfg(test)]
    pub fn clear_all(&self) {
        self.instance_operations.clear();
        self.server_operations.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use artemis_core::model::InstanceKey;

    fn create_test_instance_key() -> InstanceKey {
        InstanceKey {
            service_id: "test-service".to_string(),
            instance_id: "inst-1".to_string(),
            region_id: "us-east".to_string(),
            zone_id: "zone-1".to_string(),
            group_id: "default".to_string(),
        }
    }

    #[test]
    fn test_pull_out_and_pull_in_instance() {
        let manager = InstanceManager::new();
        let key = create_test_instance_key();

        // 初始状态: 未拉出
        assert!(!manager.is_instance_down(&key));

        // 拉出实例
        manager
            .pull_out_instance(&key, "admin".to_string(), true)
            .unwrap();
        assert!(manager.is_instance_down(&key));

        // 拉入实例
        manager
            .pull_in_instance(&key, "admin".to_string(), true)
            .unwrap();
        assert!(!manager.is_instance_down(&key));
    }

    #[test]
    fn test_pull_out_incomplete() {
        let manager = InstanceManager::new();
        let key = create_test_instance_key();

        // 拉出实例 (incomplete)
        manager
            .pull_out_instance(&key, "admin".to_string(), false)
            .unwrap();

        // incomplete 的拉出操作不算真正下线
        assert!(!manager.is_instance_down(&key));

        // 完成拉出操作
        manager
            .pull_out_instance(&key, "admin".to_string(), true)
            .unwrap();
        assert!(manager.is_instance_down(&key));
    }

    #[test]
    fn test_get_instance_operations() {
        let manager = InstanceManager::new();
        let key = create_test_instance_key();

        // 初始状态: 无操作
        assert_eq!(manager.get_instance_operations(&key).len(), 0);

        // 拉出后有操作记录
        manager
            .pull_out_instance(&key, "admin".to_string(), true)
            .unwrap();
        let ops = manager.get_instance_operations(&key);
        assert_eq!(ops.len(), 1);
        assert_eq!(ops[0], InstanceOperation::PullOut);
    }

    #[test]
    fn test_server_pull_out_and_pull_in() {
        let manager = InstanceManager::new();
        let server_id = "192.168.1.100";
        let region_id = "us-east";

        // 初始状态: 未拉出
        assert!(!manager.is_server_down(server_id, region_id));

        // 拉出服务器
        manager
            .pull_out_server(server_id, region_id, "admin".to_string(), true)
            .unwrap();
        assert!(manager.is_server_down(server_id, region_id));

        // 拉入服务器
        manager
            .pull_in_server(server_id, region_id, "admin".to_string(), true)
            .unwrap();
        assert!(!manager.is_server_down(server_id, region_id));
    }

    #[test]
    fn test_down_counts() {
        let manager = InstanceManager::new();

        let key1 = InstanceKey {
            service_id: "service-1".to_string(),
            instance_id: "inst-1".to_string(),
            region_id: "us-east".to_string(),
            zone_id: "zone-1".to_string(),
            group_id: "default".to_string(),
        };
        let key2 = InstanceKey {
            service_id: "service-2".to_string(),
            instance_id: "inst-2".to_string(),
            region_id: "us-west".to_string(),
            zone_id: "zone-1".to_string(),
            group_id: "default".to_string(),
        };

        // 拉出两个实例
        manager
            .pull_out_instance(&key1, "admin".to_string(), true)
            .unwrap();
        manager
            .pull_out_instance(&key2, "admin".to_string(), true)
            .unwrap();

        assert_eq!(manager.down_instance_count(), 2);

        // 拉出一台服务器
        manager
            .pull_out_server("192.168.1.100", "us-east", "admin".to_string(), true)
            .unwrap();
        assert_eq!(manager.down_server_count(), 1);
    }
}
