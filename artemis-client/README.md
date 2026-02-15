# Artemis Client SDK

Enterprise-grade Rust client SDK for the Artemis service registry.

## Features

### Core Features
- Service registration and deregistration
- Service discovery with caching
- Automatic heartbeat renewal
- WebSocket real-time push notifications

### Enterprise Features
- Multi-address support with automatic failover
- HTTP automatic retry mechanism with backoff
- Heartbeat TTL checking and recovery
- WebSocket Ping/Pong health checking
- Service cache TTL management (configurable expiry)
- Failed configuration retry queue
- Registry filter chain (composable instance filters)
- Prometheus metrics integration (optional)
- Batch service discovery
- WebSocket subscribe/unsubscribe support

## Quick Start

### Add Dependency

```toml
[dependencies]
artemis-client = "0.1"

# Enable Prometheus monitoring
artemis-client = { version = "0.1", features = ["metrics"] }
```

### Basic Usage

```rust
use artemis_client::{ClientConfig, RegistryClient};
use std::sync::Arc;

let config = ClientConfig::default();
let client = Arc::new(RegistryClient::new(config));

// Register instance
let response = client.register(request).await?;

// Start heartbeat
client.clone().start_heartbeat_task(keys);
```

### Multi-Address High Availability

```rust
let config = ClientConfig {
    server_urls: vec![
        "http://node1:8080".into(),
        "http://node2:8080".into(),
        "http://node3:8080".into(),
    ],
    http_retry_times: 5,
    ..Default::default()
};
```

### Using Filters

```rust
use artemis_client::{FilterChain, StatusFilter};
use artemis_core::model::InstanceStatus;

let filter = FilterChain::new()
    .add(Box::new(StatusFilter::new(vec![InstanceStatus::Up])));

let filtered = filter.apply(instances);
```

### Prometheus Monitoring

```rust
#[cfg(feature = "metrics")]
use artemis_client::{CLIENT_METRICS, REGISTRY};

// Metrics are collected automatically
// Export to Prometheus:
let metrics = prometheus::TextEncoder::new()
    .encode_to_string(&REGISTRY.gather())?;
```

### WebSocket Subscriptions

```rust
use artemis_client::websocket::WebSocketClient;

let (client, mut change_rx) = WebSocketClient::new(config);
let client = Arc::new(client);

// Subscribe to service changes
tokio::spawn(async move {
    client.connect_and_subscribe("my-service".to_string()).await.unwrap();
});

// Receive real-time changes
while let Some(changes) = change_rx.recv().await {
    for change in changes {
        println!("{:?}: {}", change.change_type, change.instance.instance_id);
    }
}
```

## Configuration Options

| Option | Default | Description |
|--------|---------|-------------|
| `server_urls` | `["http://localhost:8080"]` | Server address list |
| `heartbeat_interval_secs` | `30` | Heartbeat interval (seconds) |
| `heartbeat_ttl_secs` | `90` | Heartbeat TTL (seconds, must be >= 3x interval) |
| `http_retry_times` | `5` | HTTP retry attempts (1-10) |
| `http_retry_interval_ms` | `100` | Retry interval (milliseconds) |
| `websocket_ping_interval_secs` | `30` | WebSocket ping interval (5-300) |
| `cache_ttl_secs` | `900` | Cache TTL (15 minutes, minimum 60) |
| `address_refresh_interval_secs` | `60` | Address refresh interval |
| `enable_metrics` | `false` | Enable Prometheus monitoring |

## Module Overview

| Module | Description |
|--------|-------------|
| `address` | Multi-address management with load balancing |
| `config` | Client configuration with validation |
| `discovery` | Service discovery with TTL cache |
| `error` | Error types |
| `filter` | Instance filter chain |
| `http` | HTTP retry utilities |
| `metrics` | Prometheus metrics (optional) |
| `registry` | Service registration with retry |
| `retry` | Generic retry queue |
| `websocket` | WebSocket client with health check |

## Examples

See the `examples/` directory:
- `enterprise_client.rs` - Complete enterprise features demo
- `websocket_client.rs` - WebSocket subscription example

Run examples:
```bash
cargo run --example enterprise_client
cargo run --example enterprise_client --features metrics
cargo run --example websocket_client
```

## Testing

```bash
# Run all tests
cargo test --package artemis-client

# Run with metrics feature
cargo test --package artemis-client --features metrics

# Run all features
cargo test --package artemis-client --all-features
```

## License

MIT OR Apache-2.0
