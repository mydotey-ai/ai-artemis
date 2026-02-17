use crate::model::{
    GetAllServicesResponse, ReplicateHeartbeatRequest, ReplicateHeartbeatResponse,
    ReplicateRegisterRequest, ReplicateRegisterResponse, ReplicateUnregisterRequest,
    ReplicateUnregisterResponse,
};
use crate::model::{HeartbeatRequest, HeartbeatResponse};
use crate::model::{RegisterRequest, RegisterResponse};
use crate::model::{UnregisterRequest, UnregisterResponse};
use async_trait::async_trait;

#[async_trait]
pub trait RegistryService: Send + Sync {
    /// 注册服务实例
    async fn register(&self, request: RegisterRequest) -> RegisterResponse;

    /// 心跳续约
    async fn heartbeat(&self, request: HeartbeatRequest) -> HeartbeatResponse;

    /// 注销服务实例
    async fn unregister(&self, request: UnregisterRequest) -> UnregisterResponse;

    // ===== 复制方法(不触发二次复制) =====

    /// 从复制请求注册(不触发二次复制)
    async fn register_from_replication(
        &self,
        request: ReplicateRegisterRequest,
    ) -> ReplicateRegisterResponse;

    /// 从复制请求心跳(不触发二次复制)
    async fn heartbeat_from_replication(
        &self,
        request: ReplicateHeartbeatRequest,
    ) -> ReplicateHeartbeatResponse;

    /// 从复制请求注销(不触发二次复制)
    async fn unregister_from_replication(
        &self,
        request: ReplicateUnregisterRequest,
    ) -> ReplicateUnregisterResponse;

    /// 获取所有服务(用于启动同步)
    async fn get_all_services(&self) -> GetAllServicesResponse;
}
