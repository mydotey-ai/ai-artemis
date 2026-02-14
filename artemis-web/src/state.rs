use crate::websocket::SessionManager;
use artemis_server::{
    RegistryServiceImpl, cache::VersionedCacheManager, discovery::DiscoveryServiceImpl,
    cluster::ClusterManager, replication::ReplicationManager,
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
}
