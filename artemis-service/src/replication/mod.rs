//! Data replication implementation
//!
//! This module provides data replication across cluster nodes:
//! - Replication event handling
//! - Batch replication optimization
//! - HTTP-based replication client
//! - Error handling and retry logic

pub mod client;
pub mod error;
pub mod manager;
pub mod worker;

pub use client::ReplicationClient;
pub use error::{ReplicationError, ReplicationErrorKind};
pub use manager::{ReplicationEvent, ReplicationManager};
pub use worker::ReplicationWorker;
