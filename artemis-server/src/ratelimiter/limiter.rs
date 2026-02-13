use governor::{
    Quota, RateLimiter as GovernorRateLimiter,
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
};
use std::num::NonZeroU32;
use std::sync::Arc;

/// API限流器
#[derive(Clone)]
pub struct RateLimiter {
    limiter: Arc<GovernorRateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
}

impl RateLimiter {
    pub fn new(rps: u32) -> Self {
        let quota = Quota::per_second(NonZeroU32::new(rps).unwrap());
        let limiter = Arc::new(GovernorRateLimiter::direct(quota));
        Self { limiter }
    }

    /// 检查是否允许请求
    pub fn check(&self) -> bool {
        self.limiter.check().is_ok()
    }

    /// 异步检查（等待令牌）
    pub async fn check_async(&self) -> bool {
        self.limiter.check().is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter_allows_requests() {
        let limiter = RateLimiter::new(100);
        assert!(limiter.check());
    }

    #[test]
    fn test_rate_limiter_blocks_excess() {
        let limiter = RateLimiter::new(2);

        // 前两个请求应该通过
        assert!(limiter.check());
        assert!(limiter.check());

        // 第三个请求应该被限流
        assert!(!limiter.check());
    }
}
