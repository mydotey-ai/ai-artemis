//! Cluster management framework (Phase 10)
//!
//! This module provides the foundation for cluster functionality including:
//! - Node discovery and registration
//! - Health checking
//! - Cluster state management
//!
//! Status: Framework only, full implementation pending

pub mod manager;
pub mod node;

pub use manager::ClusterManager;
pub use node::{ClusterNode, NodeStatus};
