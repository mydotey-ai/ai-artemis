// Core service APIs (kept in artemis-web)
pub mod discovery;
pub mod metrics;
pub mod registry;
pub mod replication;
pub mod routing; // Kept here due to dependency on registry_service
pub mod status;

// Management APIs have been moved to artemis-management crate:
// - auth (authentication and user management)
// - audit (audit logs)
// - canary (canary release)
// - management (instance operations)
// - zone (zone management)
