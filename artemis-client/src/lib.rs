pub mod config;
pub mod discovery;
pub mod error;
pub mod registry;
pub mod websocket;

pub use config::ClientConfig;
pub use discovery::DiscoveryClient;
pub use error::{ClientError, Result};
pub use registry::RegistryClient;
