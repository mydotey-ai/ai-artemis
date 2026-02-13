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
}
