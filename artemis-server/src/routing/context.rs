//! Routing context for strategy execution

use serde::{Deserialize, Serialize};

/// 路由上下文 - 包含客户端信息用于路由决策
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RouteContext {
    /// 客户端 IP 地址
    pub client_ip: Option<String>,
    /// 客户端所在 Region
    pub client_region: Option<String>,
    /// 客户端所在 Zone
    pub client_zone: Option<String>,
}

impl RouteContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_ip(mut self, ip: String) -> Self {
        self.client_ip = Some(ip);
        self
    }

    pub fn with_region(mut self, region: String) -> Self {
        self.client_region = Some(region);
        self
    }

    pub fn with_zone(mut self, zone: String) -> Self {
        self.client_zone = Some(zone);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_context_builder() {
        let ctx = RouteContext::new()
            .with_ip("192.168.1.100".to_string())
            .with_region("us-east".to_string())
            .with_zone("zone-1".to_string());

        assert_eq!(ctx.client_ip, Some("192.168.1.100".to_string()));
        assert_eq!(ctx.client_region, Some("us-east".to_string()));
        assert_eq!(ctx.client_zone, Some("zone-1".to_string()));
    }
}
