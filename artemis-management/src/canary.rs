//! Canary release configuration management

use crate::dao::CanaryConfigDao;
use crate::db::Database;
use crate::model::CanaryConfig;
use dashmap::DashMap;
use std::sync::Arc;
use tracing::info;

/// 金丝雀配置管理器
#[derive(Clone)]
pub struct CanaryManager {
    /// 金丝雀配置存储: service_id -> CanaryConfig
    configs: Arc<DashMap<String, CanaryConfig>>,

    /// 可选数据库支持 - 用于持久化
    database: Option<Arc<Database>>,
}

impl Default for CanaryManager {
    fn default() -> Self {
        Self::new()
    }
}

impl CanaryManager {
    pub fn new() -> Self {
        Self::with_database(None)
    }

    pub fn with_database(database: Option<Arc<Database>>) -> Self {
        Self { configs: Arc::new(DashMap::new()), database }
    }

    /// 设置金丝雀配置
    pub fn set_config(&self, config: CanaryConfig) -> anyhow::Result<()> {
        info!(
            "Set canary config for service: {}, whitelist: {:?}",
            config.service_id, config.ip_whitelist
        );

        self.configs.insert(config.service_id.clone(), config.clone());

        // 持久化到数据库
        if let Some(db) = &self.database {
            let dao = CanaryConfigDao::new(db.conn().clone());
            let config_clone = config.clone();
            tokio::spawn(async move {
                if let Err(e) = dao.upsert_config(&config_clone).await {
                    tracing::error!("Failed to persist canary config to database: {}", e);
                }
            });
        }

        Ok(())
    }

    /// 获取金丝雀配置
    pub fn get_config(&self, service_id: &str) -> Option<CanaryConfig> {
        self.configs.get(service_id).map(|c| c.clone())
    }

    /// 启用/禁用金丝雀配置
    pub fn set_enabled(&self, service_id: &str, enabled: bool) -> anyhow::Result<()> {
        if let Some(mut config) = self.configs.get_mut(service_id) {
            config.enabled = enabled;
            info!("Set canary enabled={} for service: {}", enabled, service_id);

            // 持久化到数据库
            if let Some(db) = &self.database {
                let dao = CanaryConfigDao::new(db.conn().clone());
                let service_id_owned = service_id.to_string();
                tokio::spawn(async move {
                    if let Err(e) = dao.set_enabled(&service_id_owned, enabled).await {
                        tracing::error!(
                            "Failed to update canary enabled status in database: {}",
                            e
                        );
                    }
                });
            }

            Ok(())
        } else {
            anyhow::bail!("Canary config not found for service: {}", service_id)
        }
    }

    /// 检查 IP 是否在白名单中
    pub fn is_ip_whitelisted(&self, service_id: &str, client_ip: &str) -> bool {
        if let Some(config) = self.configs.get(service_id) {
            if !config.enabled {
                return false;
            }
            config.ip_whitelist.contains(&client_ip.to_string())
        } else {
            false
        }
    }

    /// 删除金丝雀配置
    pub fn remove_config(&self, service_id: &str) -> anyhow::Result<()> {
        self.configs.remove(service_id);
        info!("Removed canary config for service: {}", service_id);

        // 从数据库删除
        if let Some(db) = &self.database {
            let dao = CanaryConfigDao::new(db.conn().clone());
            let service_id_owned = service_id.to_string();
            tokio::spawn(async move {
                if let Err(e) = dao.delete_config(&service_id_owned).await {
                    tracing::error!("Failed to delete canary config from database: {}", e);
                }
            });
        }

        Ok(())
    }

    /// 列出所有金丝雀配置
    pub fn list_configs(&self) -> Vec<CanaryConfig> {
        self.configs.iter().map(|entry| entry.value().clone()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_get_config() {
        let manager = CanaryManager::new();

        let config = CanaryConfig {
            service_id: "my-service".to_string(),
            ip_whitelist: vec!["192.168.1.100".to_string(), "10.0.0.1".to_string()],
            enabled: true,
        };

        manager.set_config(config.clone()).unwrap();

        let retrieved = manager.get_config("my-service").unwrap();
        assert_eq!(retrieved.service_id, "my-service");
        assert_eq!(retrieved.ip_whitelist.len(), 2);
    }

    #[test]
    fn test_ip_whitelist() {
        let manager = CanaryManager::new();

        let config = CanaryConfig {
            service_id: "my-service".to_string(),
            ip_whitelist: vec!["192.168.1.100".to_string()],
            enabled: true,
        };

        manager.set_config(config).unwrap();

        assert!(manager.is_ip_whitelisted("my-service", "192.168.1.100"));
        assert!(!manager.is_ip_whitelisted("my-service", "192.168.1.101"));
    }

    #[test]
    fn test_enable_disable() {
        let manager = CanaryManager::new();

        let config = CanaryConfig {
            service_id: "my-service".to_string(),
            ip_whitelist: vec!["192.168.1.100".to_string()],
            enabled: true,
        };

        manager.set_config(config).unwrap();

        manager.set_enabled("my-service", false).unwrap();
        assert!(!manager.is_ip_whitelisted("my-service", "192.168.1.100"));

        manager.set_enabled("my-service", true).unwrap();
        assert!(manager.is_ip_whitelisted("my-service", "192.168.1.100"));
    }

    // ========== 新增补充测试 ==========

    #[test]
    fn test_canary_manager_default() {
        let manager = CanaryManager::default();
        assert_eq!(manager.list_configs().len(), 0);
    }

    #[test]
    fn test_canary_manager_clone() {
        let manager = CanaryManager::new();

        let config = CanaryConfig {
            service_id: "test".to_string(),
            ip_whitelist: vec!["192.168.1.1".to_string()],
            enabled: true,
        };

        manager.set_config(config).unwrap();

        let cloned = manager.clone();
        assert_eq!(cloned.list_configs().len(), 1);
    }

    #[test]
    fn test_get_config_not_found() {
        let manager = CanaryManager::new();
        assert!(manager.get_config("nonexistent").is_none());
    }

    #[test]
    fn test_set_enabled_not_found() {
        let manager = CanaryManager::new();
        assert!(manager.set_enabled("nonexistent", true).is_err());
    }

    #[test]
    fn test_remove_config() {
        let manager = CanaryManager::new();

        let config = CanaryConfig {
            service_id: "my-service".to_string(),
            ip_whitelist: vec!["192.168.1.100".to_string()],
            enabled: true,
        };

        manager.set_config(config).unwrap();
        assert_eq!(manager.list_configs().len(), 1);

        manager.remove_config("my-service").unwrap();
        assert_eq!(manager.list_configs().len(), 0);
    }

    #[test]
    fn test_remove_config_not_found() {
        let manager = CanaryManager::new();
        // remove_config 是幂等的,即使配置不存在也会返回 Ok
        assert!(manager.remove_config("nonexistent").is_ok());
    }

    #[test]
    fn test_list_configs_empty() {
        let manager = CanaryManager::new();
        assert_eq!(manager.list_configs().len(), 0);
    }

    #[test]
    fn test_list_configs_multiple() {
        let manager = CanaryManager::new();

        for i in 1..=3 {
            let config = CanaryConfig {
                service_id: format!("service-{}", i),
                ip_whitelist: vec![format!("192.168.1.{}", i)],
                enabled: true,
            };
            manager.set_config(config).unwrap();
        }

        let configs = manager.list_configs();
        assert_eq!(configs.len(), 3);
    }

    #[test]
    fn test_update_config() {
        let manager = CanaryManager::new();

        let config = CanaryConfig {
            service_id: "my-service".to_string(),
            ip_whitelist: vec!["192.168.1.100".to_string()],
            enabled: true,
        };

        manager.set_config(config).unwrap();

        // 更新配置 (添加更多 IP)
        let updated_config = CanaryConfig {
            service_id: "my-service".to_string(),
            ip_whitelist: vec!["192.168.1.100".to_string(), "192.168.1.101".to_string()],
            enabled: true,
        };

        manager.set_config(updated_config).unwrap();

        let retrieved = manager.get_config("my-service").unwrap();
        assert_eq!(retrieved.ip_whitelist.len(), 2);
    }

    #[test]
    fn test_is_ip_whitelisted_service_not_found() {
        let manager = CanaryManager::new();
        assert!(!manager.is_ip_whitelisted("nonexistent", "192.168.1.1"));
    }

    #[test]
    fn test_is_ip_whitelisted_disabled() {
        let manager = CanaryManager::new();

        let config = CanaryConfig {
            service_id: "my-service".to_string(),
            ip_whitelist: vec!["192.168.1.100".to_string()],
            enabled: false,
        };

        manager.set_config(config).unwrap();

        // 禁用时不应该匹配任何 IP
        assert!(!manager.is_ip_whitelisted("my-service", "192.168.1.100"));
    }

    #[test]
    fn test_empty_ip_whitelist() {
        let manager = CanaryManager::new();

        let config = CanaryConfig {
            service_id: "my-service".to_string(),
            ip_whitelist: vec![],
            enabled: true,
        };

        manager.set_config(config).unwrap();

        assert!(!manager.is_ip_whitelisted("my-service", "192.168.1.100"));
    }

    #[test]
    fn test_multiple_ips_in_whitelist() {
        let manager = CanaryManager::new();

        let config = CanaryConfig {
            service_id: "my-service".to_string(),
            ip_whitelist: vec![
                "192.168.1.100".to_string(),
                "192.168.1.101".to_string(),
                "10.0.0.1".to_string(),
            ],
            enabled: true,
        };

        manager.set_config(config).unwrap();

        assert!(manager.is_ip_whitelisted("my-service", "192.168.1.100"));
        assert!(manager.is_ip_whitelisted("my-service", "192.168.1.101"));
        assert!(manager.is_ip_whitelisted("my-service", "10.0.0.1"));
        assert!(!manager.is_ip_whitelisted("my-service", "192.168.1.102"));
    }

    #[test]
    fn test_enable_toggle() {
        let manager = CanaryManager::new();

        let config = CanaryConfig {
            service_id: "my-service".to_string(),
            ip_whitelist: vec!["192.168.1.100".to_string()],
            enabled: false,
        };

        manager.set_config(config).unwrap();
        assert!(!manager.is_ip_whitelisted("my-service", "192.168.1.100"));

        // 启用
        manager.set_enabled("my-service", true).unwrap();
        assert!(manager.is_ip_whitelisted("my-service", "192.168.1.100"));

        // 再次禁用
        manager.set_enabled("my-service", false).unwrap();
        assert!(!manager.is_ip_whitelisted("my-service", "192.168.1.100"));
    }

    #[test]
    fn test_config_persistence_after_enable_toggle() {
        let manager = CanaryManager::new();

        let config = CanaryConfig {
            service_id: "my-service".to_string(),
            ip_whitelist: vec!["192.168.1.100".to_string(), "192.168.1.101".to_string()],
            enabled: true,
        };

        manager.set_config(config).unwrap();

        // 切换 enabled 状态不应该改变 IP 白名单
        manager.set_enabled("my-service", false).unwrap();
        let retrieved = manager.get_config("my-service").unwrap();
        assert_eq!(retrieved.ip_whitelist.len(), 2);
        assert!(!retrieved.enabled);

        manager.set_enabled("my-service", true).unwrap();
        let retrieved = manager.get_config("my-service").unwrap();
        assert_eq!(retrieved.ip_whitelist.len(), 2);
        assert!(retrieved.enabled);
    }
}
