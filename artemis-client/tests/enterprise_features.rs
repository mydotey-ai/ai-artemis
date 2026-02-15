use artemis_client::{AddressManager, ClientConfig, FilterChain, StatusFilter};
use artemis_core::model::{
    DiscoveryConfig, Instance, InstanceStatus, LookupServicesRequest,
};
use std::time::Duration;

#[tokio::test]
async fn test_multi_address_failover() {
    let config = ClientConfig {
        server_urls: vec!["http://node1:8080".into(), "http://node2:8080".into()],
        ..Default::default()
    };

    assert!(config.validate().is_ok());

    let manager = AddressManager::new_static(config.server_urls.clone());
    let addr = manager.get_random_address().await;
    assert!(addr.is_some());

    let all = manager.get_all_addresses().await;
    assert_eq!(all.len(), 2);

    // Test marking addresses
    manager.mark_unavailable("http://node1:8080").await;
    let addr = manager.get_random_address().await.unwrap();
    assert_eq!(addr, "http://node2:8080");

    manager.mark_available("http://node1:8080").await;
}

#[tokio::test]
async fn test_config_validation() {
    // TTL too short
    let bad_config = ClientConfig {
        heartbeat_interval_secs: 30,
        heartbeat_ttl_secs: 60, // should be at least 90
        ..Default::default()
    };
    assert!(bad_config.validate().is_err());

    // Valid config
    let good_config = ClientConfig {
        heartbeat_interval_secs: 30,
        heartbeat_ttl_secs: 90,
        ..Default::default()
    };
    assert!(good_config.validate().is_ok());

    // Empty server URLs
    let empty_config = ClientConfig {
        server_urls: vec![],
        ..Default::default()
    };
    assert!(empty_config.validate().is_err());

    // Retry times out of range
    let retry_config = ClientConfig {
        http_retry_times: 15,
        ..Default::default()
    };
    assert!(retry_config.validate().is_err());
}

#[tokio::test]
async fn test_filter_chain_integration() {
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

    let instances = vec![
        make_instance("inst-1", InstanceStatus::Up),
        make_instance("inst-2", InstanceStatus::Down),
        make_instance("inst-3", InstanceStatus::Up),
        make_instance("inst-4", InstanceStatus::Unhealthy),
    ];

    // Filter only Up instances
    let chain = FilterChain::new().add(Box::new(StatusFilter::new(vec![InstanceStatus::Up])));
    let filtered = chain.apply(instances.clone());
    assert_eq!(filtered.len(), 2);
    assert!(filtered.iter().all(|i| i.status == InstanceStatus::Up));

    // Filter Up and Down instances
    let chain = FilterChain::new().add(Box::new(StatusFilter::new(vec![
        InstanceStatus::Up,
        InstanceStatus::Down,
    ])));
    let filtered = chain.apply(instances);
    assert_eq!(filtered.len(), 3);
}

#[tokio::test]
async fn test_batch_request_types() {
    let configs = vec![
        DiscoveryConfig {
            service_id: "service-a".into(),
            region_id: "us-east".into(),
            zone_id: "zone-1".into(),
            discovery_data: None,
        },
        DiscoveryConfig {
            service_id: "service-b".into(),
            region_id: "us-east".into(),
            zone_id: "zone-1".into(),
            discovery_data: None,
        },
        DiscoveryConfig {
            service_id: "service-c".into(),
            region_id: "eu-west".into(),
            zone_id: "zone-2".into(),
            discovery_data: None,
        },
    ];

    let request = LookupServicesRequest {
        discovery_configs: configs,
    };

    assert_eq!(request.discovery_configs.len(), 3);

    // Verify serialization roundtrip
    let json = serde_json::to_string(&request).unwrap();
    let deserialized: LookupServicesRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.discovery_configs.len(), 3);
    assert_eq!(deserialized.discovery_configs[0].service_id, "service-a");
    assert_eq!(deserialized.discovery_configs[2].region_id, "eu-west");
}

#[tokio::test]
async fn test_address_manager_dynamic() {
    let initial = vec!["http://node1:8080".into()];
    let manager = AddressManager::new_dynamic(initial, Duration::from_secs(3600));

    assert_eq!(manager.address_count(), 1);

    // Update with new addresses
    let new_urls = vec![
        "http://node1:8080".into(),
        "http://node2:8080".into(),
        "http://node3:8080".into(),
    ];
    manager.update_addresses(new_urls).await;
    assert_eq!(manager.address_count(), 3);

    let all = manager.get_all_addresses().await;
    assert_eq!(all.len(), 3);
}

#[tokio::test]
async fn test_retry_queue() {
    use artemis_client::retry::RetryQueue;

    let queue = RetryQueue::<String>::new(Duration::from_millis(50));

    // Add items
    queue.add("config-1".to_string()).await;
    queue.add("config-2".to_string()).await;
    assert_eq!(queue.len().await, 2);

    // Immediately - no items ready to retry
    let items = queue.get_items_to_retry().await;
    assert_eq!(items.len(), 0);

    // Wait for retry interval
    tokio::time::sleep(Duration::from_millis(60)).await;

    // Now items should be ready
    let items = queue.get_items_to_retry().await;
    assert_eq!(items.len(), 2);

    // Remove one
    queue.remove(&"config-1".to_string()).await;
    assert_eq!(queue.len().await, 1);
}

#[tokio::test]
async fn test_http_retry() {
    use artemis_client::error::ClientError;
    use artemis_client::http::retry_with_backoff;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    let counter = Arc::new(AtomicUsize::new(0));

    // Test: succeeds on third attempt
    let c = counter.clone();
    let result = retry_with_backoff(5, Duration::from_millis(1), || {
        let c = c.clone();
        async move {
            let count = c.fetch_add(1, Ordering::SeqCst);
            if count < 2 {
                Err(ClientError::Internal("fail".into()))
            } else {
                Ok("success")
            }
        }
    })
    .await;

    assert_eq!(result.unwrap(), "success");
    assert_eq!(counter.load(Ordering::SeqCst), 3);
}
