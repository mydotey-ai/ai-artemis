//! Enterprise client example demonstrating all Artemis client features.
//!
//! This example shows how to:
//! - Configure multi-server high availability
//! - Use address management with failover
//! - Apply instance filters
//! - Use HTTP retry mechanisms
//! - Monitor client operations (with metrics feature)
//!
//! Run with: cargo run --example enterprise_client
//! Run with metrics: cargo run --example enterprise_client --features metrics

use artemis_client::{AddressManager, ClientConfig, FilterChain, RegistryClient, StatusFilter};
use artemis_core::model::{Instance, InstanceStatus};
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).init();

    println!("=== Artemis Enterprise Client Example ===\n");

    // 1. Multi-server configuration
    let config = ClientConfig {
        server_urls: vec![
            "http://localhost:8080".into(),
            "http://localhost:8081".into(),
            "http://localhost:8082".into(),
        ],
        heartbeat_interval_secs: 10,
        heartbeat_ttl_secs: 30,
        http_retry_times: 3,
        http_retry_interval_ms: 100,
        websocket_ping_interval_secs: 15,
        cache_ttl_secs: 300,
        address_refresh_interval_secs: 60,
        enable_metrics: true,
    };

    println!("1. Configuration:");
    println!("   Server URLs: {:?}", config.server_urls);
    println!("   Heartbeat interval: {}s", config.heartbeat_interval_secs);
    println!("   Heartbeat TTL: {}s", config.heartbeat_ttl_secs);
    println!("   HTTP retry: {} times", config.http_retry_times);
    println!("   Cache TTL: {}s", config.cache_ttl_secs);

    // Validate configuration
    config.validate()?;
    println!("   Config validation: PASSED\n");

    // 2. Address management with load balancing
    let addr_manager = AddressManager::new_static(config.server_urls.clone());
    println!("2. Address Management:");
    println!("   Total addresses: {}", addr_manager.address_count());

    let random_addr = addr_manager.get_random_address().await;
    println!("   Random address: {:?}", random_addr);

    // Simulate failover
    addr_manager.mark_unavailable("http://localhost:8080").await;
    let failover_addr = addr_manager.get_random_address().await;
    println!("   After marking 8080 down: {:?}", failover_addr);

    addr_manager.mark_available("http://localhost:8080").await;
    println!("   Restored 8080\n");

    // 3. Instance filtering
    let instances = vec![
        make_instance("inst-1", InstanceStatus::Up),
        make_instance("inst-2", InstanceStatus::Down),
        make_instance("inst-3", InstanceStatus::Up),
        make_instance("inst-4", InstanceStatus::Unhealthy),
    ];

    let filter = FilterChain::new().add(Box::new(StatusFilter::new(vec![InstanceStatus::Up])));

    let filtered = filter.apply(instances.clone());
    println!("3. Instance Filtering:");
    println!("   Total instances: {}", instances.len());
    println!("   After StatusFilter(Up): {}", filtered.len());
    for inst in &filtered {
        println!("   - {} ({:?})", inst.instance_id, inst.status);
    }
    println!();

    // 4. Registry client with retry
    let _registry = Arc::new(RegistryClient::new(config.clone()));
    println!("4. Registry Client (with retry):");
    println!("   Created with {} retry attempts", config.http_retry_times);

    // NOTE: The following would require a running server
    // let register_req = RegisterRequest {
    //     instances: vec![make_instance("my-inst", InstanceStatus::Up)],
    // };
    // let response = registry.register(register_req).await?;
    // println!("   Registered: {:?}", response);

    println!("   (Server not running - skipping actual registration)\n");

    // 5. Metrics (when feature enabled)
    #[cfg(feature = "metrics")]
    {
        use artemis_client::CLIENT_METRICS;

        println!("5. Prometheus Metrics:");
        CLIENT_METRICS.heartbeat_total.inc();
        CLIENT_METRICS.discovery_total.inc();
        CLIENT_METRICS.record_http_status(200);

        println!("   Heartbeat total: {}", CLIENT_METRICS.heartbeat_total.get());
        println!("   Discovery total: {}", CLIENT_METRICS.discovery_total.get());
        println!();
    }

    #[cfg(not(feature = "metrics"))]
    {
        println!("5. Prometheus Metrics:");
        println!("   (Disabled - enable with --features metrics)\n");
    }

    println!("=== Example Complete ===");

    Ok(())
}

fn make_instance(id: &str, status: InstanceStatus) -> Instance {
    Instance {
        region_id: "us-east".into(),
        zone_id: "zone-1".into(),
        service_id: "my-service".into(),
        instance_id: id.into(),
        ip: "192.168.1.100".into(),
        port: 8080,
        status,
        group_id: None,
        machine_name: None,
        protocol: Some("http".into()),
        url: "http://192.168.1.100:8080".into(),
        health_check_url: None,
        metadata: None,
    }
}
