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

    // ===== Builder 模式测试 =====

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

    #[test]
    fn test_route_context_default() {
        let ctx = RouteContext::default();

        assert!(ctx.client_ip.is_none(), "默认 IP 应为 None");
        assert!(ctx.client_region.is_none(), "默认 Region 应为 None");
        assert!(ctx.client_zone.is_none(), "默认 Zone 应为 None");
    }

    #[test]
    fn test_route_context_new() {
        let ctx = RouteContext::new();

        assert!(ctx.client_ip.is_none());
        assert!(ctx.client_region.is_none());
        assert!(ctx.client_zone.is_none());
    }

    #[test]
    fn test_route_context_partial_info() {
        let ctx = RouteContext::new().with_ip("192.168.1.100".to_string());

        assert_eq!(ctx.client_ip, Some("192.168.1.100".to_string()));
        assert!(ctx.client_region.is_none(), "未设置 Region 应为 None");
        assert!(ctx.client_zone.is_none(), "未设置 Zone 应为 None");
    }

    #[test]
    fn test_route_context_with_region_only() {
        let ctx = RouteContext::new().with_region("us-west".to_string());

        assert!(ctx.client_ip.is_none());
        assert_eq!(ctx.client_region, Some("us-west".to_string()));
        assert!(ctx.client_zone.is_none());
    }

    #[test]
    fn test_route_context_with_zone_only() {
        let ctx = RouteContext::new().with_zone("zone-2".to_string());

        assert!(ctx.client_ip.is_none());
        assert!(ctx.client_region.is_none());
        assert_eq!(ctx.client_zone, Some("zone-2".to_string()));
    }

    #[test]
    fn test_route_context_clone() {
        let ctx1 = RouteContext::new()
            .with_ip("192.168.1.100".to_string())
            .with_region("us-east".to_string());

        let ctx2 = ctx1.clone();

        assert_eq!(ctx2.client_ip, ctx1.client_ip);
        assert_eq!(ctx2.client_region, ctx1.client_region);
    }

    #[test]
    fn test_route_context_debug_format() {
        let ctx = RouteContext::new().with_ip("127.0.0.1".to_string());

        let debug_str = format!("{:?}", ctx);
        assert!(debug_str.contains("RouteContext"));
    }
}
