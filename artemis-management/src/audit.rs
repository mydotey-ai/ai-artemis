//! Operation audit log management

use artemis_core::model::{InstanceOperationRecord, ServerOperationRecord};
use chrono::Utc;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;

/// 审计日志记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    pub log_id: i64,
    pub operation_type: String,  // "instance" | "server" | "zone" | "group" | "route"
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
        Self {
            logs: Arc::new(DashMap::new()),
            next_log_id: Arc::new(AtomicI64::new(1)),
        }
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
                    && log.operation_type != op_type {
                        return false;
                    }

                if let Some(op_id) = operator_id
                    && log.operator_id != op_id {
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
                    && !log.target_id.starts_with(&format!("{}:", sid)) {
                        return false;
                    }

                if let Some(op_id) = operator_id
                    && log.operator_id != op_id {
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
                    && !log.target_id.starts_with(&format!("{}:", sid)) {
                        return false;
                    }

                if let Some(op_id) = operator_id
                    && log.operator_id != op_id {
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
                    && !log.target_id.contains(gid) {
                        return false;
                    }

                if let Some(op_id) = operator_id
                    && log.operator_id != op_id {
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
                    && !log.target_id.contains(rid) {
                        return false;
                    }

                if let Some(op_id) = operator_id
                    && log.operator_id != op_id {
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
                    && !log.target_id.contains(&format!("rule:{}", rid)) {
                        return false;
                    }

                if let Some(gid) = group_id
                    && !log.target_id.contains(&format!("group:{}", gid)) {
                        return false;
                    }

                if let Some(op_id) = operator_id
                    && log.operator_id != op_id {
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
                    && !log.target_id.contains(&format!("zone:{}", zid)) {
                        return false;
                    }

                if let Some(rid) = region_id
                    && !log.target_id.contains(&format!("region:{}", rid)) {
                        return false;
                    }

                if let Some(op_id) = operator_id
                    && log.operator_id != op_id {
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
                    && !log.target_id.contains(&format!("group:{}", gid)) {
                        return false;
                    }

                if let Some(iid) = instance_id
                    && !log.target_id.contains(&format!("instance:{}", iid)) {
                        return false;
                    }

                if let Some(op_id) = operator_id
                    && log.operator_id != op_id {
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
                    && !log.target_id.starts_with(&format!("{}:", sid)) {
                        return false;
                    }

                if let Some(rid) = region_id
                    && !log.target_id.contains(&format!("region:{}", rid)) {
                        return false;
                    }

                if let Some(op_id) = operator_id
                    && log.operator_id != op_id {
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
    use artemis_core::model::{InstanceKey, InstanceOperation};

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
    }

    #[test]
    fn test_query_logs_with_filter() {
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

        let operator1_logs = manager.query_logs(None, Some("operator-1"), None);
        assert_eq!(operator1_logs.len(), 1);
    }
}
