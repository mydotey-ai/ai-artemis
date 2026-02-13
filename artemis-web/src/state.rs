use artemis_server::{
    RegistryServiceImpl, cache::VersionedCacheManager, discovery::DiscoveryServiceImpl,
};
use crate::websocket::SessionManager;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub registry_service: Arc<RegistryServiceImpl>,
    pub discovery_service: Arc<DiscoveryServiceImpl>,
    pub cache: Arc<VersionedCacheManager>,
    pub session_manager: Arc<SessionManager>,
}
