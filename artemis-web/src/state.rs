use crate::websocket::SessionManager;
use artemis_management::InstanceManager;
use artemis_server::{
    cache::VersionedCacheManager, cluster::ClusterManager, discovery::DiscoveryServiceImpl,
    replication::ReplicationManager, RegistryServiceImpl,
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
}
