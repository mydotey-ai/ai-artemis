//! Management Web API module
//!
//! This module provides HTTP API endpoints for management functionality:
//! - Authentication and user management
//! - Instance operations
//! - Group and routing management
//! - Audit logs
//! - Zone management
//! - Canary release

pub mod api;
pub mod middleware;
pub mod routes;
pub mod state;

pub use routes::management_routes;
pub use state::ManagementState;
