use axum::{http::StatusCode, response::IntoResponse};
use lazy_static::lazy_static;
use prometheus::{
    Encoder, IntCounter, IntGauge, TextEncoder, register_int_counter, register_int_gauge,
};

lazy_static! {
    pub static ref REGISTER_REQUESTS: IntCounter =
        register_int_counter!("artemis_register_requests_total", "Total register requests")
            .unwrap();
    pub static ref HEARTBEAT_REQUESTS: IntCounter =
        register_int_counter!("artemis_heartbeat_requests_total", "Total heartbeat requests")
            .unwrap();
    pub static ref DISCOVERY_REQUESTS: IntCounter =
        register_int_counter!("artemis_discovery_requests_total", "Total discovery requests")
            .unwrap();
    pub static ref ACTIVE_INSTANCES: IntGauge =
        register_int_gauge!("artemis_active_instances", "Number of active instances").unwrap();
}

pub async fn metrics() -> impl IntoResponse {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = Vec::new();

    if let Err(e) = encoder.encode(&metric_families, &mut buffer) {
        return (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to encode metrics: {}", e))
            .into_response();
    }

    (StatusCode::OK, buffer).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_metrics_endpoint_returns_ok() {
        let response = metrics().await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn test_register_requests_counter_exists() {
        // 验证计数器已注册并可访问
        let _ = REGISTER_REQUESTS.get();
    }

    #[test]
    fn test_heartbeat_requests_counter_exists() {
        // 验证计数器已注册并可访问
        let _ = HEARTBEAT_REQUESTS.get();
    }

    #[test]
    fn test_discovery_requests_counter_exists() {
        // 验证计数器已注册并可访问
        let _ = DISCOVERY_REQUESTS.get();
    }

    #[test]
    fn test_active_instances_gauge_exists() {
        // 验证 Gauge 已注册并可访问
        let _ = ACTIVE_INSTANCES.get();
    }

    #[test]
    fn test_metrics_can_be_gathered() {
        // 确保至少一个指标被初始化
        let _ = REGISTER_REQUESTS.get();
        let _ = HEARTBEAT_REQUESTS.get();
        let _ = DISCOVERY_REQUESTS.get();
        let _ = ACTIVE_INSTANCES.get();

        // 验证可以收集指标
        let metric_families = prometheus::gather();
        // 指标已经在 lazy_static 中注册,所以应该可以收集到
        assert!(!metric_families.is_empty(), "Should be able to gather metrics");
    }
}
