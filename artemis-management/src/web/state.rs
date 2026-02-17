//! Management state for HTTP API

use crate::{
    AuditManager, AuthManager, CanaryManager, GroupManager, InstanceManager, RouteManager,
    ZoneManager,
};
use std::sync::Arc;

/// Management state containing all management-related services
#[derive(Clone)]
pub struct ManagementState {
    pub auth_manager: Arc<AuthManager>,
    pub instance_manager: Arc<InstanceManager>,
    pub group_manager: Arc<GroupManager>,
    pub route_manager: Arc<RouteManager>,
    pub zone_manager: Arc<ZoneManager>,
    pub canary_manager: Arc<CanaryManager>,
    pub audit_manager: Arc<AuditManager>,
}

impl ManagementState {
    /// Create a new ManagementState
    pub fn new(
        auth_manager: Arc<AuthManager>,
        instance_manager: Arc<InstanceManager>,
        group_manager: Arc<GroupManager>,
        route_manager: Arc<RouteManager>,
        zone_manager: Arc<ZoneManager>,
        canary_manager: Arc<CanaryManager>,
        audit_manager: Arc<AuditManager>,
    ) -> Self {
        Self {
            auth_manager,
            instance_manager,
            group_manager,
            route_manager,
            zone_manager,
            canary_manager,
            audit_manager,
        }
    }
}
