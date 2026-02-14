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
