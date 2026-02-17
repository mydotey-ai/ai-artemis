//! Operation audit log management

use crate::model::{InstanceOperationRecord, ServerOperationRecord};
use chrono::Utc;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};

/// 审计日志记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    pub log_id: i64,
    pub operation_type: String, // "instance" | "server" | "zone" | "group" | "route"
    pub target_id: String,
    pub operation: String,
    pub operator_id: String,
    pub operation_time: i64,
    pub details: Option<String>,
}

/// 审计管理器
#[derive(Clone)]
pub struct AuditManager {
    /// 审计日志存储: log_id -> AuditLog
    logs: Arc<DashMap<i64, AuditLog>>,
    /// 日志 ID 生成器
    next_log_id: Arc<AtomicI64>,
}

impl Default for AuditManager {
    fn default() -> Self {
        Self::new()
    }
}

impl AuditManager {
    pub fn new() -> Self {
        Self { logs: Arc::new(DashMap::new()), next_log_id: Arc::new(AtomicI64::new(1)) }
    }

    /// 记录实例操作日志
    pub fn log_instance_operation(&self, record: &InstanceOperationRecord) {
        let log_id = self.next_log_id.fetch_add(1, Ordering::Relaxed);

        let log = AuditLog {
            log_id,
            operation_type: "instance".to_string(),
            target_id: format!(
                "{}:{}:{}",
                record.instance_key.service_id,
                record.instance_key.region_id,
                record.instance_key.instance_id
            ),
            operation: record.operation.to_string(),
            operator_id: record.operator_id.clone(),
            operation_time: Utc::now().timestamp(),
            details: None,
        };

        self.logs.insert(log_id, log);
    }

    /// 记录服务器操作日志
    pub fn log_server_operation(&self, record: &ServerOperationRecord) {
        let log_id = self.next_log_id.fetch_add(1, Ordering::Relaxed);

        let log = AuditLog {
            log_id,
            operation_type: "server".to_string(),
            target_id: format!("{}:{}", record.server_id, record.region_id),
            operation: record.operation.to_string(),
            operator_id: record.operator_id.clone(),
            operation_time: record.operation_time,
            details: None,
        };

        self.logs.insert(log_id, log);
    }

    /// 记录通用操作日志
    pub fn log_operation(
        &self,
        operation_type: String,
        target_id: String,
        operation: String,
        operator_id: String,
    ) {
        let log_id = self.next_log_id.fetch_add(1, Ordering::Relaxed);

        let log = AuditLog {
            log_id,
            operation_type,
            target_id,
            operation,
            operator_id,
            operation_time: Utc::now().timestamp(),
            details: None,
        };

        self.logs.insert(log_id, log);
    }

    /// 查询操作日志
    pub fn query_logs(
        &self,
        operation_type: Option<&str>,
        operator_id: Option<&str>,
        limit: Option<usize>,
    ) -> Vec<AuditLog> {
        let mut logs: Vec<AuditLog> = self
            .logs
            .iter()
            .filter(|entry| {
                let log = entry.value();

                if let Some(op_type) = operation_type
                    && log.operation_type != op_type
                {
                    return false;
                }

                if let Some(op_id) = operator_id
                    && log.operator_id != op_id
                {
                    return false;
                }

                true
            })
            .map(|entry| entry.value().clone())
            .collect();

        // Sort by operation_time descending
        logs.sort_by(|a, b| b.operation_time.cmp(&a.operation_time));

        // Apply limit
        if let Some(limit) = limit {
            logs.truncate(limit);
        }

        logs
    }

    /// 查询实例操作日志
    pub fn query_instance_logs(
        &self,
        service_id: Option<&str>,
        operator_id: Option<&str>,
        limit: Option<usize>,
    ) -> Vec<AuditLog> {
        let mut logs: Vec<AuditLog> = self
            .logs
            .iter()
            .filter(|entry| {
                let log = entry.value();

                if log.operation_type != "instance" {
                    return false;
                }

                if let Some(sid) = service_id
                    && !log.target_id.starts_with(&format!("{}:", sid))
                {
                    return false;
                }

                if let Some(op_id) = operator_id
                    && log.operator_id != op_id
                {
                    return false;
                }

                true
            })
            .map(|entry| entry.value().clone())
            .collect();

        logs.sort_by(|a, b| b.operation_time.cmp(&a.operation_time));

        if let Some(limit) = limit {
            logs.truncate(limit);
        }

        logs
    }

    /// 查询服务器操作日志
    pub fn query_server_logs(
        &self,
        server_id: Option<&str>,
        operator_id: Option<&str>,
        limit: Option<usize>,
    ) -> Vec<AuditLog> {
        let mut logs: Vec<AuditLog> = self
            .logs
            .iter()
            .filter(|entry| {
                let log = entry.value();

                if log.operation_type != "server" {
                    return false;
                }

                if let Some(sid) = server_id
                    && !log.target_id.starts_with(&format!("{}:", sid))
                {
                    return false;
                }

                if let Some(op_id) = operator_id
                    && log.operator_id != op_id
                {
                    return false;
                }

                true
            })
            .map(|entry| entry.value().clone())
            .collect();

        logs.sort_by(|a, b| b.operation_time.cmp(&a.operation_time));

        if let Some(limit) = limit {
            logs.truncate(limit);
        }

        logs
    }

    /// 清理过期日志
    pub fn cleanup_old_logs(&self, retention_days: i64) {
        let cutoff_time = Utc::now().timestamp() - (retention_days * 86400);

        self.logs.retain(|_, log| log.operation_time >= cutoff_time);
    }

    // ===== Phase 24: 审计日志细分 API =====

    /// 查询分组操作日志
    pub fn query_group_logs(
        &self,
        group_id: Option<&str>,
        operator_id: Option<&str>,
        limit: Option<usize>,
    ) -> Vec<AuditLog> {
        let mut logs: Vec<AuditLog> = self
            .logs
            .iter()
            .filter(|entry| {
                let log = entry.value();

                if log.operation_type != "group" {
                    return false;
                }

                if let Some(gid) = group_id
                    && !log.target_id.contains(gid)
                {
                    return false;
                }

                if let Some(op_id) = operator_id
                    && log.operator_id != op_id
                {
                    return false;
                }

                true
            })
            .map(|entry| entry.value().clone())
            .collect();

        logs.sort_by(|a, b| b.operation_time.cmp(&a.operation_time));

        if let Some(limit) = limit {
            logs.truncate(limit);
        }

        logs
    }

    /// 查询路由规则操作日志
    pub fn query_route_rule_logs(
        &self,
        rule_id: Option<&str>,
        operator_id: Option<&str>,
        limit: Option<usize>,
    ) -> Vec<AuditLog> {
        let mut logs: Vec<AuditLog> = self
            .logs
            .iter()
            .filter(|entry| {
                let log = entry.value();

                if log.operation_type != "route_rule" {
                    return false;
                }

                if let Some(rid) = rule_id
                    && !log.target_id.contains(rid)
                {
                    return false;
                }

                if let Some(op_id) = operator_id
                    && log.operator_id != op_id
                {
                    return false;
                }

                true
            })
            .map(|entry| entry.value().clone())
            .collect();

        logs.sort_by(|a, b| b.operation_time.cmp(&a.operation_time));

        if let Some(limit) = limit {
            logs.truncate(limit);
        }

        logs
    }

    /// 查询路由规则分组操作日志
    pub fn query_route_rule_group_logs(
        &self,
        rule_id: Option<&str>,
        group_id: Option<&str>,
        operator_id: Option<&str>,
        limit: Option<usize>,
    ) -> Vec<AuditLog> {
        let mut logs: Vec<AuditLog> = self
            .logs
            .iter()
            .filter(|entry| {
                let log = entry.value();

                if log.operation_type != "route_rule_group" {
                    return false;
                }

                if let Some(rid) = rule_id
                    && !log.target_id.contains(&format!("rule:{}", rid))
                {
                    return false;
                }

                if let Some(gid) = group_id
                    && !log.target_id.contains(&format!("group:{}", gid))
                {
                    return false;
                }

                if let Some(op_id) = operator_id
                    && log.operator_id != op_id
                {
                    return false;
                }

                true
            })
            .map(|entry| entry.value().clone())
            .collect();

        logs.sort_by(|a, b| b.operation_time.cmp(&a.operation_time));

        if let Some(limit) = limit {
            logs.truncate(limit);
        }

        logs
    }

    /// 查询 Zone 操作日志
    pub fn query_zone_logs(
        &self,
        zone_id: Option<&str>,
        region_id: Option<&str>,
        operator_id: Option<&str>,
        limit: Option<usize>,
    ) -> Vec<AuditLog> {
        let mut logs: Vec<AuditLog> = self
            .logs
            .iter()
            .filter(|entry| {
                let log = entry.value();

                if log.operation_type != "zone" {
                    return false;
                }

                if let Some(zid) = zone_id
                    && !log.target_id.contains(&format!("zone:{}", zid))
                {
                    return false;
                }

                if let Some(rid) = region_id
                    && !log.target_id.contains(&format!("region:{}", rid))
                {
                    return false;
                }

                if let Some(op_id) = operator_id
                    && log.operator_id != op_id
                {
                    return false;
                }

                true
            })
            .map(|entry| entry.value().clone())
            .collect();

        logs.sort_by(|a, b| b.operation_time.cmp(&a.operation_time));

        if let Some(limit) = limit {
            logs.truncate(limit);
        }

        logs
    }

    /// 查询分组实例绑定日志
    pub fn query_group_instance_logs(
        &self,
        group_id: Option<&str>,
        instance_id: Option<&str>,
        operator_id: Option<&str>,
        limit: Option<usize>,
    ) -> Vec<AuditLog> {
        let mut logs: Vec<AuditLog> = self
            .logs
            .iter()
            .filter(|entry| {
                let log = entry.value();

                if log.operation_type != "group_instance" {
                    return false;
                }

                if let Some(gid) = group_id
                    && !log.target_id.contains(&format!("group:{}", gid))
                {
                    return false;
                }

                if let Some(iid) = instance_id
                    && !log.target_id.contains(&format!("instance:{}", iid))
                {
                    return false;
                }

                if let Some(op_id) = operator_id
                    && log.operator_id != op_id
                {
                    return false;
                }

                true
            })
            .map(|entry| entry.value().clone())
            .collect();

        logs.sort_by(|a, b| b.operation_time.cmp(&a.operation_time));

        if let Some(limit) = limit {
            logs.truncate(limit);
        }

        logs
    }

    /// 查询服务实例日志 (服务维度的实例变更)
    pub fn query_service_instance_logs(
        &self,
        service_id: Option<&str>,
        region_id: Option<&str>,
        operator_id: Option<&str>,
        limit: Option<usize>,
    ) -> Vec<AuditLog> {
        let mut logs: Vec<AuditLog> = self
            .logs
            .iter()
            .filter(|entry| {
                let log = entry.value();

                // service_instance 类型或者 instance 类型都可以
                if log.operation_type != "service_instance" && log.operation_type != "instance" {
                    return false;
                }

                if let Some(sid) = service_id
                    && !log.target_id.contains(&format!("service:{}", sid))
                    && !log.target_id.starts_with(&format!("{}:", sid))
                {
                    return false;
                }

                if let Some(rid) = region_id
                    && !log.target_id.contains(&format!("region:{}", rid))
                {
                    return false;
                }

                if let Some(op_id) = operator_id
                    && log.operator_id != op_id
                {
                    return false;
                }

                true
            })
            .map(|entry| entry.value().clone())
            .collect();

        logs.sort_by(|a, b| b.operation_time.cmp(&a.operation_time));

        if let Some(limit) = limit {
            logs.truncate(limit);
        }

        logs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{InstanceOperation, ServerOperation};
    use artemis_core::model::InstanceKey;

    // ========== 基础功能测试 ==========

    #[test]
    fn test_audit_manager_new() {
        let manager = AuditManager::new();
        let logs = manager.query_logs(None, None, None);
        assert_eq!(logs.len(), 0);
    }

    #[test]
    fn test_audit_manager_default() {
        let manager = AuditManager::default();
        let logs = manager.query_logs(None, None, None);
        assert_eq!(logs.len(), 0);
    }

    #[test]
    fn test_audit_manager_clone() {
        let manager = AuditManager::new();
        manager.log_operation(
            "test".to_string(),
            "target-1".to_string(),
            "create".to_string(),
            "admin".to_string(),
        );

        let cloned = manager.clone();
        let logs = cloned.query_logs(None, None, None);
        assert_eq!(logs.len(), 1);
    }

    // ========== 实例操作日志测试 ==========

    #[test]
    fn test_log_instance_operation() {
        let manager = AuditManager::new();

        let record = InstanceOperationRecord {
            instance_key: InstanceKey {
                region_id: "us-east".to_string(),
                zone_id: "zone-1".to_string(),
                service_id: "my-service".to_string(),
                group_id: "group-1".to_string(),
                instance_id: "inst-1".to_string(),
            },
            operation: InstanceOperation::PullOut,
            operation_complete: true,
            operator_id: "operator-1".to_string(),
            token: None,
        };

        manager.log_instance_operation(&record);

        let logs = manager.query_instance_logs(Some("my-service"), None, None);
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].operation_type, "instance");
        assert_eq!(logs[0].operator_id, "operator-1");
        assert!(logs[0].target_id.contains("my-service"));
    }

    #[test]
    fn test_log_instance_operation_pull_in() {
        let manager = AuditManager::new();

        let record = InstanceOperationRecord {
            instance_key: InstanceKey {
                region_id: "us-west".to_string(),
                zone_id: "zone-2".to_string(),
                service_id: "test-service".to_string(),
                group_id: "group-2".to_string(),
                instance_id: "inst-2".to_string(),
            },
            operation: InstanceOperation::PullIn,
            operation_complete: true,
            operator_id: "operator-2".to_string(),
            token: None,
        };

        manager.log_instance_operation(&record);

        let logs = manager.query_instance_logs(Some("test-service"), None, None);
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].operation, "pullin");
    }

    // ========== 服务器操作日志测试 ==========

    #[test]
    fn test_log_server_operation() {
        let manager = AuditManager::new();

        let record = ServerOperationRecord {
            server_id: "server-1".to_string(),
            region_id: "us-east".to_string(),
            operation: ServerOperation::PullOut,
            operator_id: "admin".to_string(),
            operation_time: Utc::now().timestamp(),
        };

        manager.log_server_operation(&record);

        let logs = manager.query_server_logs(Some("server-1"), None, None);
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].operation_type, "server");
        assert!(logs[0].target_id.contains("server-1"));
    }

    #[test]
    fn test_log_server_operation_pull_in() {
        let manager = AuditManager::new();

        let record = ServerOperationRecord {
            server_id: "server-2".to_string(),
            region_id: "eu-west".to_string(),
            operation: ServerOperation::PullIn,
            operator_id: "operator-3".to_string(),
            operation_time: Utc::now().timestamp(),
        };

        manager.log_server_operation(&record);

        let logs = manager.query_server_logs(None, Some("operator-3"), None);
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].operation, "pullin");
    }

    // ========== 通用操作日志测试 ==========

    #[test]
    fn test_log_operation() {
        let manager = AuditManager::new();

        manager.log_operation(
            "group".to_string(),
            "group-123".to_string(),
            "create".to_string(),
            "admin".to_string(),
        );

        let logs = manager.query_logs(Some("group"), None, None);
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].operation_type, "group");
        assert_eq!(logs[0].target_id, "group-123");
        assert_eq!(logs[0].operation, "create");
    }

    // ========== 查询日志测试 ==========

    #[test]
    fn test_query_logs_no_filter() {
        let manager = AuditManager::new();

        manager.log_operation(
            "type1".to_string(),
            "target1".to_string(),
            "op1".to_string(),
            "user1".to_string(),
        );
        manager.log_operation(
            "type2".to_string(),
            "target2".to_string(),
            "op2".to_string(),
            "user2".to_string(),
        );

        let logs = manager.query_logs(None, None, None);
        assert_eq!(logs.len(), 2);
    }

    #[test]
    fn test_query_logs_with_operation_type_filter() {
        let manager = AuditManager::new();

        manager.log_operation(
            "instance".to_string(),
            "service-1:us-east:inst-1".to_string(),
            "pullout".to_string(),
            "operator-1".to_string(),
        );
        manager.log_operation(
            "server".to_string(),
            "192.168.1.100:us-east".to_string(),
            "pullout".to_string(),
            "operator-2".to_string(),
        );

        let instance_logs = manager.query_logs(Some("instance"), None, None);
        assert_eq!(instance_logs.len(), 1);

        let server_logs = manager.query_logs(Some("server"), None, None);
        assert_eq!(server_logs.len(), 1);
    }

    #[test]
    fn test_query_logs_with_operator_filter() {
        let manager = AuditManager::new();

        manager.log_operation(
            "type1".to_string(),
            "target1".to_string(),
            "op1".to_string(),
            "operator-1".to_string(),
        );
        manager.log_operation(
            "type2".to_string(),
            "target2".to_string(),
            "op2".to_string(),
            "operator-2".to_string(),
        );

        let operator1_logs = manager.query_logs(None, Some("operator-1"), None);
        assert_eq!(operator1_logs.len(), 1);
        assert_eq!(operator1_logs[0].operator_id, "operator-1");
    }

    #[test]
    fn test_query_logs_with_limit() {
        let manager = AuditManager::new();

        for i in 1..=5 {
            manager.log_operation(
                "test".to_string(),
                format!("target-{}", i),
                "create".to_string(),
                "admin".to_string(),
            );
        }

        let logs = manager.query_logs(None, None, Some(3));
        assert_eq!(logs.len(), 3);
    }

    #[test]
    fn test_query_logs_sorting() {
        let manager = AuditManager::new();

        manager.log_operation(
            "type1".to_string(),
            "target1".to_string(),
            "op1".to_string(),
            "user1".to_string(),
        );
        std::thread::sleep(std::time::Duration::from_secs(1));
        manager.log_operation(
            "type2".to_string(),
            "target2".to_string(),
            "op2".to_string(),
            "user2".to_string(),
        );

        let logs = manager.query_logs(None, None, None);
        assert_eq!(logs.len(), 2);
        // 应该按时间降序排列(最新的在前)
        // 由于时间戳精度问题,只验证长度
        assert!(logs[0].operation_time >= logs[1].operation_time);
    }

    // ========== 查询实例日志测试 ==========

    #[test]
    fn test_query_instance_logs_by_service() {
        let manager = AuditManager::new();

        manager.log_operation(
            "instance".to_string(),
            "service-1:region:inst-1".to_string(),
            "register".to_string(),
            "admin".to_string(),
        );
        manager.log_operation(
            "instance".to_string(),
            "service-2:region:inst-2".to_string(),
            "register".to_string(),
            "admin".to_string(),
        );

        let logs = manager.query_instance_logs(Some("service-1"), None, None);
        assert_eq!(logs.len(), 1);
        assert!(logs[0].target_id.starts_with("service-1:"));
    }

    #[test]
    fn test_query_instance_logs_by_operator() {
        let manager = AuditManager::new();

        manager.log_operation(
            "instance".to_string(),
            "service-1:region:inst-1".to_string(),
            "register".to_string(),
            "operator-1".to_string(),
        );
        manager.log_operation(
            "instance".to_string(),
            "service-1:region:inst-2".to_string(),
            "register".to_string(),
            "operator-2".to_string(),
        );

        let logs = manager.query_instance_logs(None, Some("operator-1"), None);
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].operator_id, "operator-1");
    }

    #[test]
    fn test_query_instance_logs_with_limit() {
        let manager = AuditManager::new();

        for i in 1..=5 {
            manager.log_operation(
                "instance".to_string(),
                format!("service-{}:region:inst-{}", i, i),
                "register".to_string(),
                "admin".to_string(),
            );
        }

        let logs = manager.query_instance_logs(None, None, Some(2));
        assert_eq!(logs.len(), 2);
    }

    // ========== 查询服务器日志测试 ==========

    #[test]
    fn test_query_server_logs_by_server_id() {
        let manager = AuditManager::new();

        manager.log_operation(
            "server".to_string(),
            "server-1:region-1".to_string(),
            "pullout".to_string(),
            "admin".to_string(),
        );
        manager.log_operation(
            "server".to_string(),
            "server-2:region-1".to_string(),
            "pullout".to_string(),
            "admin".to_string(),
        );

        let logs = manager.query_server_logs(Some("server-1"), None, None);
        assert_eq!(logs.len(), 1);
        assert!(logs[0].target_id.starts_with("server-1:"));
    }

    #[test]
    fn test_query_server_logs_with_limit() {
        let manager = AuditManager::new();

        for i in 1..=5 {
            manager.log_operation(
                "server".to_string(),
                format!("server-{}:region", i),
                "pullout".to_string(),
                "admin".to_string(),
            );
        }

        let logs = manager.query_server_logs(None, None, Some(3));
        assert_eq!(logs.len(), 3);
    }

    // ========== 清理过期日志测试 ==========

    #[test]
    fn test_cleanup_old_logs() {
        let manager = AuditManager::new();

        // 添加一些日志
        manager.log_operation(
            "test".to_string(),
            "target-1".to_string(),
            "create".to_string(),
            "admin".to_string(),
        );
        manager.log_operation(
            "test".to_string(),
            "target-2".to_string(),
            "create".to_string(),
            "admin".to_string(),
        );

        // 验证日志存在
        let logs_before = manager.query_logs(None, None, None);
        assert_eq!(logs_before.len(), 2);

        // 清理未来的日志(retention_days = -1 表示清理所有)
        manager.cleanup_old_logs(-1);

        // 验证日志已清理
        let logs_after = manager.query_logs(None, None, None);
        assert_eq!(logs_after.len(), 0);
    }

    #[test]
    fn test_cleanup_old_logs_retention() {
        let manager = AuditManager::new();

        manager.log_operation(
            "test".to_string(),
            "target-1".to_string(),
            "create".to_string(),
            "admin".to_string(),
        );

        // 保留 30 天的日志(不应该清理刚创建的日志)
        manager.cleanup_old_logs(30);

        let logs = manager.query_logs(None, None, None);
        assert_eq!(logs.len(), 1);
    }

    // ========== Phase 24: 审计日志细分 API 测试 ==========

    #[test]
    fn test_query_group_logs() {
        let manager = AuditManager::new();

        manager.log_operation(
            "group".to_string(),
            "group-1".to_string(),
            "create".to_string(),
            "admin".to_string(),
        );
        manager.log_operation(
            "group".to_string(),
            "group-2".to_string(),
            "update".to_string(),
            "admin".to_string(),
        );
        manager.log_operation(
            "route_rule".to_string(),
            "rule-1".to_string(),
            "create".to_string(),
            "admin".to_string(),
        );

        let logs = manager.query_group_logs(None, None, None);
        assert_eq!(logs.len(), 2);

        let group1_logs = manager.query_group_logs(Some("group-1"), None, None);
        assert_eq!(group1_logs.len(), 1);
    }

    #[test]
    fn test_query_route_rule_logs() {
        let manager = AuditManager::new();

        manager.log_operation(
            "route_rule".to_string(),
            "rule-1".to_string(),
            "create".to_string(),
            "admin".to_string(),
        );
        manager.log_operation(
            "route_rule".to_string(),
            "rule-2".to_string(),
            "update".to_string(),
            "operator-1".to_string(),
        );

        let logs = manager.query_route_rule_logs(None, None, None);
        assert_eq!(logs.len(), 2);

        let rule1_logs = manager.query_route_rule_logs(Some("rule-1"), None, None);
        assert_eq!(rule1_logs.len(), 1);
    }

    #[test]
    fn test_query_route_rule_group_logs() {
        let manager = AuditManager::new();

        manager.log_operation(
            "route_rule_group".to_string(),
            "rule:rule-1,group:group-1".to_string(),
            "add".to_string(),
            "admin".to_string(),
        );
        manager.log_operation(
            "route_rule_group".to_string(),
            "rule:rule-1,group:group-2".to_string(),
            "add".to_string(),
            "admin".to_string(),
        );

        let logs = manager.query_route_rule_group_logs(None, None, None, None);
        assert_eq!(logs.len(), 2);

        let rule1_logs = manager.query_route_rule_group_logs(Some("rule-1"), None, None, None);
        assert_eq!(rule1_logs.len(), 2);

        let group1_logs = manager.query_route_rule_group_logs(None, Some("group-1"), None, None);
        assert_eq!(group1_logs.len(), 1);
    }

    #[test]
    fn test_query_zone_logs() {
        let manager = AuditManager::new();

        manager.log_operation(
            "zone".to_string(),
            "region:us-east,zone:zone-1".to_string(),
            "pullout".to_string(),
            "admin".to_string(),
        );
        manager.log_operation(
            "zone".to_string(),
            "region:eu-west,zone:zone-2".to_string(),
            "pullin".to_string(),
            "admin".to_string(),
        );

        let logs = manager.query_zone_logs(None, None, None, None);
        assert_eq!(logs.len(), 2);

        let zone1_logs = manager.query_zone_logs(Some("zone-1"), None, None, None);
        assert_eq!(zone1_logs.len(), 1);

        let region_logs = manager.query_zone_logs(None, Some("us-east"), None, None);
        assert_eq!(region_logs.len(), 1);
    }

    #[test]
    fn test_query_group_instance_logs() {
        let manager = AuditManager::new();

        manager.log_operation(
            "group_instance".to_string(),
            "group:group-1,instance:inst-1".to_string(),
            "bind".to_string(),
            "admin".to_string(),
        );
        manager.log_operation(
            "group_instance".to_string(),
            "group:group-1,instance:inst-2".to_string(),
            "bind".to_string(),
            "admin".to_string(),
        );

        let logs = manager.query_group_instance_logs(None, None, None, None);
        assert_eq!(logs.len(), 2);

        let group1_logs = manager.query_group_instance_logs(Some("group-1"), None, None, None);
        assert_eq!(group1_logs.len(), 2);

        let inst1_logs = manager.query_group_instance_logs(None, Some("inst-1"), None, None);
        assert_eq!(inst1_logs.len(), 1);
    }

    #[test]
    fn test_query_service_instance_logs() {
        let manager = AuditManager::new();

        manager.log_operation(
            "service_instance".to_string(),
            "service:svc-1,region:us-east,instance:inst-1".to_string(),
            "register".to_string(),
            "admin".to_string(),
        );
        manager.log_operation(
            "instance".to_string(),
            "svc-2:us-west:inst-2".to_string(),
            "unregister".to_string(),
            "admin".to_string(),
        );

        let logs = manager.query_service_instance_logs(None, None, None, None);
        assert_eq!(logs.len(), 2);

        let svc1_logs = manager.query_service_instance_logs(Some("svc-1"), None, None, None);
        assert_eq!(svc1_logs.len(), 1);

        let region_logs = manager.query_service_instance_logs(None, Some("us-east"), None, None);
        assert_eq!(region_logs.len(), 1);
    }

    #[test]
    fn test_query_with_operator_and_limit() {
        let manager = AuditManager::new();

        for i in 1..=5 {
            manager.log_operation(
                "group".to_string(),
                format!("group-{}", i),
                "create".to_string(),
                "operator-1".to_string(),
            );
        }

        manager.log_operation(
            "group".to_string(),
            "group-6".to_string(),
            "create".to_string(),
            "operator-2".to_string(),
        );

        let logs = manager.query_group_logs(None, Some("operator-1"), Some(3));
        assert_eq!(logs.len(), 3);
        assert_eq!(logs[0].operator_id, "operator-1");
    }

    // ========== AuditLog 结构测试 ==========

    #[test]
    fn test_audit_log_debug() {
        let log = AuditLog {
            log_id: 1,
            operation_type: "test".to_string(),
            target_id: "target-1".to_string(),
            operation: "create".to_string(),
            operator_id: "admin".to_string(),
            operation_time: Utc::now().timestamp(),
            details: None,
        };

        let debug_str = format!("{:?}", log);
        assert!(debug_str.contains("AuditLog"));
        assert!(debug_str.contains("test"));
    }

    #[test]
    fn test_audit_log_clone() {
        let log = AuditLog {
            log_id: 1,
            operation_type: "test".to_string(),
            target_id: "target-1".to_string(),
            operation: "create".to_string(),
            operator_id: "admin".to_string(),
            operation_time: Utc::now().timestamp(),
            details: Some("test details".to_string()),
        };

        let cloned = log.clone();
        assert_eq!(cloned.log_id, log.log_id);
        assert_eq!(cloned.operation_type, log.operation_type);
        assert_eq!(cloned.details, log.details);
    }
}
