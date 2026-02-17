use super::repository::RegistryRepository;
use crate::cache::VersionedCacheManager;
use crate::change::InstanceChangeManager;
use crate::lease::LeaseManager;
use crate::replication::ReplicationManager;
use crate::traits::RegistryService;
use artemis_core::model::{
    BatchHeartbeatRequest,
    BatchHeartbeatResponse,
    // Phase 23: 批量复制 API
    BatchRegisterRequest,
    BatchRegisterResponse,
    BatchUnregisterRequest,
    BatchUnregisterResponse,
    ErrorCode,
    GetAllServicesResponse,
    HeartbeatRequest,
    HeartbeatResponse,
    RegisterRequest,
    RegisterResponse,
    ReplicateHeartbeatRequest,
    ReplicateHeartbeatResponse,
    ReplicateRegisterRequest,
    ReplicateRegisterResponse,
    ReplicateUnregisterRequest,
    ReplicateUnregisterResponse,
    ResponseStatus,
    ServicesDeltaRequest,
    ServicesDeltaResponse,
    SyncFullDataRequest,
    SyncFullDataResponse,
    UnregisterRequest,
    UnregisterResponse,
};
use async_trait::async_trait;
use std::sync::Arc;
use tracing::{info, warn};

#[derive(Clone)]
pub struct RegistryServiceImpl {
    repository: RegistryRepository,
    lease_manager: Arc<LeaseManager>,
    cache: Arc<VersionedCacheManager>,
    change_manager: Arc<InstanceChangeManager>,
    replication_manager: Option<Arc<ReplicationManager>>,
}

impl RegistryServiceImpl {
    pub fn new(
        repository: RegistryRepository,
        lease_manager: Arc<LeaseManager>,
        cache: Arc<VersionedCacheManager>,
        change_manager: Arc<InstanceChangeManager>,
        replication_manager: Option<Arc<ReplicationManager>>,
    ) -> Self {
        Self { repository, lease_manager, cache, change_manager, replication_manager }
    }

    // ===== Phase 23: 批量复制 API (独立方法,不属于 trait) =====

    /// 批量注册 - 用于节点间批量数据同步
    pub async fn batch_register(&self, request: BatchRegisterRequest) -> BatchRegisterResponse {
        info!("Batch registering {} instances from replication", request.instances.len());

        let failed = Vec::new();
        let mut affected_services = std::collections::HashSet::new();

        for instance in request.instances {
            let service_id = instance.service_id.clone();
            let key = instance.key();

            self.repository.register(instance.clone());
            self.lease_manager.create_lease(key.clone());
            affected_services.insert(service_id);
        }

        // 批量更新缓存
        for service_id in affected_services {
            self.rebuild_and_cache_service(&service_id);
        }

        BatchRegisterResponse {
            response_status: if failed.is_empty() {
                ResponseStatus::success()
            } else {
                ResponseStatus::error(ErrorCode::BadRequest, "Some registrations failed")
            },
            failed_instances: if failed.is_empty() { None } else { Some(failed) },
        }
    }

    /// 批量心跳 - 优化网络请求
    pub async fn batch_heartbeat(&self, request: BatchHeartbeatRequest) -> BatchHeartbeatResponse {
        info!("Batch heartbeat for {} instances from replication", request.instance_keys.len());

        let mut failed = Vec::new();

        for key in request.instance_keys {
            if !self.lease_manager.renew(&key) {
                warn!("Batch heartbeat failed for non-existent instance: {:?}", key);
                failed.push(key);
            }
        }

        BatchHeartbeatResponse {
            response_status: if failed.is_empty() {
                ResponseStatus::success()
            } else {
                ResponseStatus::error(ErrorCode::BadRequest, "Some heartbeats failed")
            },
            failed_instance_keys: if failed.is_empty() { None } else { Some(failed) },
        }
    }

    /// 批量注销
    pub async fn batch_unregister(
        &self,
        request: BatchUnregisterRequest,
    ) -> BatchUnregisterResponse {
        info!("Batch unregistering {} instances from replication", request.instance_keys.len());

        let mut affected_services = std::collections::HashSet::new();
        let mut failed = Vec::new();

        for key in request.instance_keys {
            let service_id = key.service_id.clone();

            if let Some(instance) = self.repository.remove(&key) {
                self.lease_manager.remove_lease(&key);
                affected_services.insert(service_id);
                self.change_manager.publish_unregister(&key, &instance);
            } else {
                warn!("Batch unregister failed for non-existent instance: {:?}", key);
                failed.push(key);
            }
        }

        // 批量更新缓存
        for service_id in affected_services {
            self.rebuild_and_cache_service(&service_id);
        }

        BatchUnregisterResponse {
            response_status: if failed.is_empty() {
                ResponseStatus::success()
            } else {
                ResponseStatus::error(ErrorCode::BadRequest, "Some unregistrations failed")
            },
            failed_instance_keys: if failed.is_empty() { None } else { Some(failed) },
        }
    }

    /// 增量同步 - 获取指定时间戳之后的变更
    pub async fn get_services_delta(
        &self,
        _request: ServicesDeltaRequest,
    ) -> ServicesDeltaResponse {
        // 注意:当前实现返回所有服务 (与 Java 版本保持一致)
        // 未来可扩展为基于 version_id 的真正增量同步
        let services = self.repository.get_all_services();
        let current_timestamp =
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()
                as i64;

        ServicesDeltaResponse {
            response_status: ResponseStatus::success(),
            services,
            current_timestamp,
        }
    }

    /// 全量同步 - 新节点加入时的完整数据同步
    pub async fn sync_full_data(&self, request: SyncFullDataRequest) -> SyncFullDataResponse {
        info!(
            "Full data sync requested for region_id={}, zone_id={:?}",
            request.region_id, request.zone_id
        );

        let services = self.repository.get_all_services();
        let sync_timestamp =
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()
                as i64;

        SyncFullDataResponse {
            response_status: ResponseStatus::success(),
            services,
            sync_timestamp,
        }
    }

    /// 重建服务并更新缓存
    fn rebuild_and_cache_service(&self, service_id: &str) {
        let instances = self.repository.get_instances_by_service(service_id);

        if instances.is_empty() {
            // 没有实例,删除缓存
            self.cache.remove_service(service_id);
        } else {
            // 有实例,更新缓存
            let service = artemis_core::model::Service {
                service_id: service_id.to_string(),
                metadata: None,
                instances,
                logic_instances: None,
            };
            self.cache.update_service(service);
        }
    }

    /// 获取分组的实例
    pub fn get_instances_by_group(
        &self,
        service_id: &str,
        group_id: &str,
        region_id: Option<&str>,
    ) -> Vec<artemis_core::model::Instance> {
        self.repository.get_instances_by_group(service_id, group_id, region_id)
    }
}

#[async_trait]
impl RegistryService for RegistryServiceImpl {
    async fn register(&self, request: RegisterRequest) -> RegisterResponse {
        let failed = Vec::new();

        for instance in request.instances {
            let key = instance.key();
            info!("Registering instance: {:?}", key);

            let service_id = instance.service_id.clone();

            // 注册实例
            self.repository.register(instance.clone());

            // 创建租约
            self.lease_manager.create_lease(key);

            // 更新缓存
            self.rebuild_and_cache_service(&service_id);

            // 发布变更事件
            self.change_manager.publish_register(&instance);

            // 触发复制
            if let Some(ref repl_mgr) = self.replication_manager {
                repl_mgr.publish_register(instance.clone());
            }
        }

        RegisterResponse {
            response_status: if failed.is_empty() {
                ResponseStatus::success()
            } else {
                ResponseStatus::error(ErrorCode::BadRequest, "Some instances failed")
            },
            failed_instances: if failed.is_empty() { None } else { Some(failed) },
        }
    }

    async fn heartbeat(&self, request: HeartbeatRequest) -> HeartbeatResponse {
        // 预分配容量以减少内存重新分配
        let mut failed = Vec::with_capacity(request.instance_keys.len());

        for key in &request.instance_keys {
            if !self.lease_manager.renew(key) {
                warn!("Heartbeat failed for non-existent instance: {:?}", key);
                failed.push(key.clone());
            } else {
                // 触发复制(只复制成功的心跳)
                if let Some(ref repl_mgr) = self.replication_manager {
                    repl_mgr.publish_heartbeat(key.clone());
                }
            }
        }

        HeartbeatResponse {
            response_status: if failed.is_empty() {
                ResponseStatus::success()
            } else {
                ResponseStatus::error(ErrorCode::BadRequest, "Some heartbeats failed")
            },
            failed_instance_keys: if failed.is_empty() { None } else { Some(failed) },
        }
    }

    async fn unregister(&self, request: UnregisterRequest) -> UnregisterResponse {
        for key in request.instance_keys {
            info!("Unregistering instance: {:?}", key);

            let service_id = key.service_id.clone();

            // 获取实例信息用于发布事件
            if let Some(instance) = self.repository.remove(&key) {
                self.lease_manager.remove_lease(&key);

                // 更新缓存
                self.rebuild_and_cache_service(&service_id);

                // 发布变更事件
                self.change_manager.publish_unregister(&key, &instance);

                // 触发复制
                if let Some(ref repl_mgr) = self.replication_manager {
                    repl_mgr.publish_unregister(key.clone());
                }
            }
        }

        UnregisterResponse { response_status: ResponseStatus::success() }
    }

    // ===== 复制方法实现(不触发二次复制) =====

    async fn register_from_replication(
        &self,
        request: ReplicateRegisterRequest,
    ) -> ReplicateRegisterResponse {
        let failed = Vec::new();

        for instance in request.instances {
            let key = instance.key();
            info!("Registering instance from replication: {:?}", key);

            let service_id = instance.service_id.clone();

            // 只做本地处理,不触发复制
            self.repository.register(instance.clone());
            self.lease_manager.create_lease(key);

            // 更新缓存
            self.rebuild_and_cache_service(&service_id);

            // 发布变更事件(用于 WebSocket 推送等)
            self.change_manager.publish_register(&instance);
        }

        ReplicateRegisterResponse {
            response_status: if failed.is_empty() {
                ResponseStatus::success()
            } else {
                ResponseStatus::error(ErrorCode::BadRequest, "Some instances failed")
            },
            failed_instances: if failed.is_empty() { None } else { Some(failed) },
        }
    }

    async fn heartbeat_from_replication(
        &self,
        request: ReplicateHeartbeatRequest,
    ) -> ReplicateHeartbeatResponse {
        let mut failed = Vec::with_capacity(request.instance_keys.len());

        for key in request.instance_keys {
            if !self.lease_manager.renew(&key) {
                warn!("Heartbeat from replication failed for non-existent instance: {:?}", key);
                failed.push(key);
            }
        }

        ReplicateHeartbeatResponse {
            response_status: if failed.is_empty() {
                ResponseStatus::success()
            } else {
                ResponseStatus::error(ErrorCode::BadRequest, "Some heartbeats failed")
            },
            failed_instance_keys: if failed.is_empty() { None } else { Some(failed) },
        }
    }

    async fn unregister_from_replication(
        &self,
        request: ReplicateUnregisterRequest,
    ) -> ReplicateUnregisterResponse {
        for key in request.instance_keys {
            info!("Unregistering instance from replication: {:?}", key);

            let service_id = key.service_id.clone();

            if let Some(instance) = self.repository.remove(&key) {
                self.lease_manager.remove_lease(&key);

                // 更新缓存
                self.rebuild_and_cache_service(&service_id);

                self.change_manager.publish_unregister(&key, &instance);
            }
        }

        ReplicateUnregisterResponse { response_status: ResponseStatus::success() }
    }

    async fn get_all_services(&self) -> GetAllServicesResponse {
        let services = self.repository.get_all_services();

        GetAllServicesResponse { response_status: ResponseStatus::success(), services }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::change::InstanceChangeManager;
    use artemis_core::model::{Instance, InstanceStatus};
    use std::time::Duration;

    fn create_test_instance() -> Instance {
        Instance {
            region_id: "test".to_string(),
            zone_id: "zone".to_string(),
            group_id: None,
            service_id: "my-service".to_string(),
            instance_id: "inst-1".to_string(),
            machine_name: None,
            ip: "127.0.0.1".to_string(),
            port: 8080,
            protocol: None,
            url: "http://127.0.0.1:8080".to_string(),
            health_check_url: None,
            status: InstanceStatus::Up,
            metadata: None,
        }
    }

    #[tokio::test]
    async fn test_register() {
        let repo = RegistryRepository::new();
        let lease_mgr = Arc::new(LeaseManager::new(Duration::from_secs(30)));
        let cache = Arc::new(crate::cache::VersionedCacheManager::new());
        let change_mgr = Arc::new(InstanceChangeManager::new());
        let service = RegistryServiceImpl::new(repo.clone(), lease_mgr, cache, change_mgr, None);

        let request = RegisterRequest { instances: vec![create_test_instance()] };

        let response = service.register(request).await;
        assert_eq!(response.response_status.error_code, ErrorCode::Success);
        assert_eq!(repo.count(), 1);
    }

    #[tokio::test]
    async fn test_heartbeat() {
        let repo = RegistryRepository::new();
        let lease_mgr = Arc::new(LeaseManager::new(Duration::from_secs(30)));
        let cache = Arc::new(crate::cache::VersionedCacheManager::new());
        let change_mgr = Arc::new(InstanceChangeManager::new());
        let service = RegistryServiceImpl::new(repo, lease_mgr, cache, change_mgr, None);

        let instance = create_test_instance();
        let key = instance.key();

        // 先注册
        let reg_req = RegisterRequest { instances: vec![instance] };
        service.register(reg_req).await;

        // 心跳
        let hb_req = HeartbeatRequest { instance_keys: vec![key] };
        let response = service.heartbeat(hb_req).await;
        assert_eq!(response.response_status.error_code, ErrorCode::Success);
    }

    #[tokio::test]
    async fn test_unregister() {
        let repo = RegistryRepository::new();
        let lease_mgr = Arc::new(LeaseManager::new(Duration::from_secs(30)));
        let cache = Arc::new(crate::cache::VersionedCacheManager::new());
        let change_mgr = Arc::new(InstanceChangeManager::new());
        let service = RegistryServiceImpl::new(repo.clone(), lease_mgr, cache, change_mgr, None);

        let instance = create_test_instance();
        let key = instance.key();

        // 先注册
        service.register(RegisterRequest { instances: vec![instance] }).await;

        // 注销
        let unreg_req = UnregisterRequest { instance_keys: vec![key] };
        let response = service.unregister(unreg_req).await;
        assert_eq!(response.response_status.error_code, ErrorCode::Success);
        assert_eq!(repo.count(), 0);
    }
}
