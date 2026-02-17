use crate::websocket::SessionManager;
use artemis_management::{
    AuditManager, AuthManager, CanaryManager, GroupManager, InstanceManager, RouteManager,
    ZoneManager,
};
use artemis_server::{
    RegistryServiceImpl, StatusService,
    cache::VersionedCacheManager,
    cluster::ClusterManager,
    discovery::{DiscoveryServiceImpl, LoadBalancer},
    replication::ReplicationManager,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub registry_service: Arc<RegistryServiceImpl>,
    pub discovery_service: Arc<DiscoveryServiceImpl>,
    pub cache: Arc<VersionedCacheManager>,
    pub session_manager: Arc<SessionManager>,
    pub cluster_manager: Option<Arc<ClusterManager>>,
    pub replication_manager: Option<Arc<ReplicationManager>>,
    pub instance_manager: Arc<InstanceManager>,
    pub group_manager: Arc<GroupManager>,
    pub route_manager: Arc<RouteManager>,
    pub zone_manager: Arc<ZoneManager>,
    pub canary_manager: Arc<CanaryManager>,
    pub audit_manager: Arc<AuditManager>,
    pub auth_manager: Arc<AuthManager>,
    pub load_balancer: Arc<LoadBalancer>,
    pub status_service: Arc<StatusService>,
}
