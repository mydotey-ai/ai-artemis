//! Prometheus metrics for Artemis client.
//!
//! This module is only available when the `metrics` feature is enabled.
//! It provides counters and histograms for tracking client operations.

#[cfg(feature = "metrics")]
use lazy_static::lazy_static;
#[cfg(feature = "metrics")]
use prometheus::{Histogram, HistogramOpts, IntCounter, IntCounterVec, Opts, Registry};

#[cfg(feature = "metrics")]
lazy_static! {
    /// Global Prometheus registry for client metrics
    pub static ref REGISTRY: Registry = Registry::new();

    /// Client metrics singleton
    pub static ref CLIENT_METRICS: ClientMetrics = ClientMetrics::new();
}

/// Client metrics with Prometheus counters and histograms.
///
/// Tracks heartbeat, discovery, HTTP, and WebSocket operations.
#[cfg(feature = "metrics")]
pub struct ClientMetrics {
    /// Total number of heartbeat requests
    pub heartbeat_total: IntCounter,
    /// Total number of heartbeat errors
    pub heartbeat_errors: IntCounter,
    /// Heartbeat latency histogram (seconds)
    pub heartbeat_latency: Histogram,

    /// Total number of discovery requests
    pub discovery_total: IntCounter,
    /// Discovery latency histogram (seconds)
    pub discovery_latency: Histogram,

    /// HTTP status code distribution
    pub http_status_codes: IntCounterVec,

    /// Total number of WebSocket messages received
    pub websocket_messages: IntCounter,
    /// Total number of WebSocket connections made
    pub websocket_connections: IntCounter,
}

#[cfg(feature = "metrics")]
impl Default for ClientMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "metrics")]
impl ClientMetrics {
    /// Create a new ClientMetrics instance and register all metrics
    pub fn new() -> Self {
        let heartbeat_total = IntCounter::with_opts(Opts::new(
            "artemis_client_heartbeat_total",
            "Total number of heartbeats",
        ))
        .unwrap();

        let heartbeat_errors = IntCounter::with_opts(Opts::new(
            "artemis_client_heartbeat_errors",
            "Total heartbeat errors",
        ))
        .unwrap();

        let heartbeat_latency = Histogram::with_opts(
            HistogramOpts::new(
                "artemis_client_heartbeat_latency_seconds",
                "Heartbeat latency",
            )
            .buckets(vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0]),
        )
        .unwrap();

        let discovery_total = IntCounter::with_opts(Opts::new(
            "artemis_client_discovery_total",
            "Total service discoveries",
        ))
        .unwrap();

        let discovery_latency = Histogram::with_opts(
            HistogramOpts::new(
                "artemis_client_discovery_latency_seconds",
                "Discovery latency",
            )
            .buckets(vec![0.01, 0.05, 0.1, 0.5, 1.0, 2.0, 5.0]),
        )
        .unwrap();

        let http_status_codes = IntCounterVec::new(
            Opts::new(
                "artemis_client_http_status_total",
                "HTTP status code distribution",
            ),
            &["status_code"],
        )
        .unwrap();

        let websocket_messages = IntCounter::with_opts(Opts::new(
            "artemis_client_websocket_messages_total",
            "Total WebSocket messages",
        ))
        .unwrap();

        let websocket_connections = IntCounter::with_opts(Opts::new(
            "artemis_client_websocket_connections_total",
            "Total WebSocket connections",
        ))
        .unwrap();

        // Register all metrics
        REGISTRY.register(Box::new(heartbeat_total.clone())).ok();
        REGISTRY.register(Box::new(heartbeat_errors.clone())).ok();
        REGISTRY.register(Box::new(heartbeat_latency.clone())).ok();
        REGISTRY.register(Box::new(discovery_total.clone())).ok();
        REGISTRY.register(Box::new(discovery_latency.clone())).ok();
        REGISTRY.register(Box::new(http_status_codes.clone())).ok();
        REGISTRY
            .register(Box::new(websocket_messages.clone()))
            .ok();
        REGISTRY
            .register(Box::new(websocket_connections.clone()))
            .ok();

        Self {
            heartbeat_total,
            heartbeat_errors,
            heartbeat_latency,
            discovery_total,
            discovery_latency,
            http_status_codes,
            websocket_messages,
            websocket_connections,
        }
    }

    /// Record an HTTP status code
    pub fn record_http_status(&self, status: u16) {
        self.http_status_codes
            .with_label_values(&[&status.to_string()])
            .inc();
    }
}

/// No-op metrics implementation when the `metrics` feature is disabled
#[cfg(not(feature = "metrics"))]
pub struct ClientMetrics;

#[cfg(not(feature = "metrics"))]
impl ClientMetrics {
    /// Create a no-op metrics instance
    pub fn new() -> Self {
        Self
    }

    /// No-op HTTP status recording
    pub fn record_http_status(&self, _status: u16) {}
}

#[cfg(not(feature = "metrics"))]
impl Default for ClientMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(all(test, feature = "metrics"))]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_initialization() {
        let metrics = ClientMetrics::new();

        // Record some metrics
        metrics.heartbeat_total.inc();
        metrics.discovery_total.inc();

        // Verify counters incremented
        assert!(metrics.heartbeat_total.get() > 0);
        assert!(metrics.discovery_total.get() > 0);
    }

    #[test]
    fn test_http_status_recording() {
        let metrics = ClientMetrics::new();

        metrics.record_http_status(200);
        metrics.record_http_status(404);
        metrics.record_http_status(500);

        // Verify does not panic (exact values hard to test due to global state)
    }
}
