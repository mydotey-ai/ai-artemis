use std::fmt;

/// 复制错误类型
#[derive(Debug, Clone)]
pub struct ReplicationError {
    pub kind: ReplicationErrorKind,
    pub message: String,
}

/// 复制错误分类
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplicationErrorKind {
    /// 429 Too Many Requests - 可重试
    RateLimited,
    /// 网络超时 - 可重试
    NetworkTimeout,
    /// 503 Service Unavailable - 可重试
    ServiceUnavailable,
    /// 400 Bad Request - 不可重试
    BadRequest,
    /// 其他永久失败 - 不可重试
    PermanentFailure,
}

impl ReplicationError {
    pub fn new(kind: ReplicationErrorKind, message: impl Into<String>) -> Self {
        Self { kind, message: message.into() }
    }

    /// 判断错误是否可重试
    pub fn is_retryable(&self) -> bool {
        matches!(
            self.kind,
            ReplicationErrorKind::RateLimited
                | ReplicationErrorKind::NetworkTimeout
                | ReplicationErrorKind::ServiceUnavailable
        )
    }

    /// 从 HTTP 状态码创建错误
    pub fn from_status(status: reqwest::StatusCode) -> Self {
        let kind = match status.as_u16() {
            429 => ReplicationErrorKind::RateLimited,
            503 => ReplicationErrorKind::ServiceUnavailable,
            400 => ReplicationErrorKind::BadRequest,
            _ => ReplicationErrorKind::PermanentFailure,
        };

        Self::new(kind, format!("HTTP {}", status))
    }

    /// 从 reqwest 错误创建
    pub fn from_reqwest(error: reqwest::Error) -> Self {
        if error.is_timeout() {
            Self::new(ReplicationErrorKind::NetworkTimeout, format!("Request timeout: {}", error))
        } else if error.is_connect() {
            Self::new(
                ReplicationErrorKind::ServiceUnavailable,
                format!("Connection failed: {}", error),
            )
        } else {
            Self::new(ReplicationErrorKind::PermanentFailure, format!("Request failed: {}", error))
        }
    }
}

impl fmt::Display for ReplicationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: {}", self.kind, self.message)
    }
}

impl std::error::Error for ReplicationError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retryable_errors() {
        let err = ReplicationError::new(ReplicationErrorKind::RateLimited, "test");
        assert!(err.is_retryable());

        let err = ReplicationError::new(ReplicationErrorKind::NetworkTimeout, "test");
        assert!(err.is_retryable());

        let err = ReplicationError::new(ReplicationErrorKind::ServiceUnavailable, "test");
        assert!(err.is_retryable());
    }

    #[test]
    fn test_non_retryable_errors() {
        let err = ReplicationError::new(ReplicationErrorKind::BadRequest, "test");
        assert!(!err.is_retryable());

        let err = ReplicationError::new(ReplicationErrorKind::PermanentFailure, "test");
        assert!(!err.is_retryable());
    }

    #[test]
    fn test_from_status() {
        let err = ReplicationError::from_status(reqwest::StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(err.kind, ReplicationErrorKind::RateLimited);
        assert!(err.is_retryable());

        let err = ReplicationError::from_status(reqwest::StatusCode::BAD_REQUEST);
        assert_eq!(err.kind, ReplicationErrorKind::BadRequest);
        assert!(!err.is_retryable());
    }
}
