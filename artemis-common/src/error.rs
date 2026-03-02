use thiserror::Error;

#[derive(Error, Debug)]
pub enum ArtemisError {
    #[error("Invalid instance: {0}")]
    InvalidInstance(String),

    #[error("Service not found: {0}")]
    ServiceNotFound(String),

    #[error("Instance not found: {0}")]
    InstanceNotFound(String),

    #[error("Lease expired for instance: {0}")]
    LeaseExpired(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Internal error: {0}")]
    InternalError(String),
}

pub type Result<T> = std::result::Result<T, ArtemisError>;
