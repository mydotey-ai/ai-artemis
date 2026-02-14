use super::repository::RegistryRepository;
use crate::change::InstanceChangeManager;
use crate::lease::LeaseManager;
use crate::replication::ReplicationManager;
use artemis_core::model::{
    ErrorCode, HeartbeatRequest, HeartbeatResponse, RegisterRequest, RegisterResponse,
    ResponseStatus, UnregisterRequest, UnregisterResponse,
    ReplicateRegisterRequest, ReplicateRegisterResponse,
    ReplicateHeartbeatRequest, ReplicateHeartbeatResponse,
    ReplicateUnregisterRequest, ReplicateUnregisterResponse,
    GetAllServicesResponse,
};
use artemis_core::traits::RegistryService;
use async_trait::async_trait;
use std::sync::Arc;
use tracing::{info, warn};

#[derive(Clone)]
pub struct RegistryServiceImpl {
    repository: RegistryRepository,
    lease_manager: Arc<LeaseManager>,
    change_manager: Arc<InstanceChangeManager>,
    replication_manager: Option<Arc<ReplicationManager>>,
}

impl RegistryServiceImpl {
    pub fn new(
        repository: RegistryRepository,
        lease_manager: Arc<LeaseManager>,
        change_manager: Arc<InstanceChangeManager>,
        replication_manager: Option<Arc<ReplicationManager>>,
    ) -> Self {
        Self {
            repository,
            lease_manager,
            change_manager,
            replication_manager,
        }
    }
}

#[async_trait]
impl RegistryService for RegistryServiceImpl {
    async fn register(&self, request: RegisterRequest) -> RegisterResponse {
        let failed = Vec::new();

        for instance in request.instances {
            let key = instance.key();
            info!("Registering instance: {:?}", key);

            // 注册实例
            self.repository.register(instance.clone());

            // 创建租约
            self.lease_manager.create_lease(key);

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

            // 获取实例信息用于发布事件
            if let Some(instance) = self.repository.remove(&key) {
                self.lease_manager.remove_lease(&key);

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

    async fn register_from_replication(&self, request: ReplicateRegisterRequest) -> ReplicateRegisterResponse {
        let failed = Vec::new();

        for instance in request.instances {
            let key = instance.key();
            info!("Registering instance from replication: {:?}", key);

            // 只做本地处理,不触发复制
            self.repository.register(instance.clone());
            self.lease_manager.create_lease(key);

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

    async fn heartbeat_from_replication(&self, request: ReplicateHeartbeatRequest) -> ReplicateHeartbeatResponse {
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

    async fn unregister_from_replication(&self, request: ReplicateUnregisterRequest) -> ReplicateUnregisterResponse {
        for key in request.instance_keys {
            info!("Unregistering instance from replication: {:?}", key);

            if let Some(instance) = self.repository.remove(&key) {
                self.lease_manager.remove_lease(&key);
                self.change_manager.publish_unregister(&key, &instance);
            }
        }

        ReplicateUnregisterResponse { response_status: ResponseStatus::success() }
    }

    async fn get_all_services(&self) -> GetAllServicesResponse {
        let services = self.repository.get_all_services();

        GetAllServicesResponse {
            response_status: ResponseStatus::success(),
            services,
        }
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
        let change_mgr = Arc::new(InstanceChangeManager::new());
        let service = RegistryServiceImpl::new(repo.clone(), lease_mgr, change_mgr, None);

        let request = RegisterRequest { instances: vec![create_test_instance()] };

        let response = service.register(request).await;
        assert_eq!(response.response_status.error_code, ErrorCode::Success);
        assert_eq!(repo.count(), 1);
    }

    #[tokio::test]
    async fn test_heartbeat() {
        let repo = RegistryRepository::new();
        let lease_mgr = Arc::new(LeaseManager::new(Duration::from_secs(30)));
        let change_mgr = Arc::new(InstanceChangeManager::new());
        let service = RegistryServiceImpl::new(repo, lease_mgr, change_mgr, None);

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
        let change_mgr = Arc::new(InstanceChangeManager::new());
        let service = RegistryServiceImpl::new(repo.clone(), lease_mgr, change_mgr, None);

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
