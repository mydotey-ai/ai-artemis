use crate::db::Database;
use crate::dao::{GroupDao, RouteRuleDao, ZoneOperationDao, CanaryConfigDao};
use crate::{GroupManager, RouteManager, ZoneManager, CanaryManager};
use std::sync::Arc;

/// 配置加载器 - 从数据库加载所有持久化配置到内存
pub struct ConfigLoader {
    database: Arc<Database>,
    group_manager: Arc<GroupManager>,
    route_manager: Arc<RouteManager>,
    zone_manager: Arc<ZoneManager>,
    canary_manager: Arc<CanaryManager>,
}

impl ConfigLoader {
    pub fn new(
        database: Arc<Database>,
        group_manager: Arc<GroupManager>,
        route_manager: Arc<RouteManager>,
        zone_manager: Arc<ZoneManager>,
        canary_manager: Arc<CanaryManager>,
    ) -> Self {
        Self {
            database,
            group_manager,
            route_manager,
            zone_manager,
            canary_manager,
        }
    }

    /// 加载所有配置
    pub async fn load_all(&self) -> anyhow::Result<()> {
        tracing::info!("Loading all configurations from database");

        // 1. 加载服务分组
        self.load_service_groups().await?;

        // 2. 加载路由规则
        self.load_route_rules().await?;

        // 3. 加载Zone操作
        self.load_zone_operations().await?;

        // 4. 加载金丝雀配置
        self.load_canary_configs().await?;

        tracing::info!("All configurations loaded successfully");
        Ok(())
    }

    /// 加载服务分组
    async fn load_service_groups(&self) -> anyhow::Result<()> {
        let dao = GroupDao::new(self.database.pool().clone());
        let groups = dao.list_groups().await?;

        tracing::info!("Loading {} service groups", groups.len());

        for group in groups {
            // 恢复到内存 (GroupManager内部使用DashMap)
            if let Err(e) = self.group_manager.create_group(group.clone()) {
                tracing::warn!("Failed to load group {}: {}", group.name, e);
            }
        }

        Ok(())
    }

    /// 加载路由规则
    async fn load_route_rules(&self) -> anyhow::Result<()> {
        let dao = RouteRuleDao::new(self.database.pool().clone());
        let rules = dao.list_rules().await?;

        tracing::info!("Loading {} route rules", rules.len());

        for mut rule in rules {
            // 加载规则关联的分组
            let group_ids = dao.get_rule_group_ids(&rule.route_id).await?;

            // 通过GroupManager查询完整的ServiceGroup
            for (group_id, weight) in group_ids {
                if let Some(group) = self.group_manager.get_group(&group_id) {
                    // 创建service.rs的ServiceGroup (用于RouteRule.groups)
                    let service_group = artemis_core::model::service::ServiceGroup {
                        group_key: group.name.clone(),
                        weight: Some(weight),
                        instance_ids: None,
                        instances: None,
                        metadata: group.metadata.clone(),
                    };
                    rule.groups.push(service_group);
                }
            }

            // 恢复到内存
            if let Err(e) = self.route_manager.create_rule(rule.clone()) {
                tracing::warn!("Failed to load rule {}: {}", rule.route_id, e);
            }
        }

        Ok(())
    }

    /// 加载Zone操作
    async fn load_zone_operations(&self) -> anyhow::Result<()> {
        let dao = ZoneOperationDao::new(self.database.pool().clone());
        let operations = dao.list_operations().await?;

        tracing::info!("Loading {} zone operations", operations.len());

        for op in operations {
            // 恢复Zone操作到内存
            match op.operation {
                artemis_core::model::ZoneOperation::PullOut => {
                    if let Err(e) = self.zone_manager.pull_out_zone(
                        &op.zone_id,
                        &op.region_id,
                        op.operator_id.clone(),
                    ) {
                        tracing::warn!("Failed to load zone pullout {}/{}: {}", op.zone_id, op.region_id, e);
                    }
                }
                artemis_core::model::ZoneOperation::PullIn => {
                    // PullIn不需要恢复,因为默认状态就是PullIn
                }
            }
        }

        Ok(())
    }

    /// 加载金丝雀配置
    async fn load_canary_configs(&self) -> anyhow::Result<()> {
        let dao = CanaryConfigDao::new(self.database.pool().clone());
        let configs = dao.list_configs().await?;

        tracing::info!("Loading {} canary configs", configs.len());

        for config in configs {
            // 恢复到内存
            if let Err(e) = self.canary_manager.set_config(config.clone()) {
                tracing::warn!("Failed to load canary config {}: {}", config.service_id, e);
            }
        }

        Ok(())
    }
}
