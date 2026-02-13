use crate::model::{GetServiceRequest, GetServiceResponse};
use crate::model::{GetServicesDeltaRequest, GetServicesDeltaResponse};
use crate::model::{GetServicesRequest, GetServicesResponse};
use async_trait::async_trait;

#[async_trait]
pub trait DiscoveryService: Send + Sync {
    /// 查询单个服务
    async fn get_service(&self, request: GetServiceRequest) -> GetServiceResponse;

    /// 查询所有服务
    async fn get_services(&self, request: GetServicesRequest) -> GetServicesResponse;

    /// 查询服务变更（增量）
    async fn get_services_delta(
        &self,
        request: GetServicesDeltaRequest,
    ) -> GetServicesDeltaResponse;
}
