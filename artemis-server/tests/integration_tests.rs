// End-to-end integration tests for Artemis service registry
use artemis_client::{ClientConfig, DiscoveryClient, RegistryClient};
use artemis_core::model::{
    DiscoveryConfig, GetServiceRequest, HeartbeatRequest, Instance, InstanceStatus,
    RegisterRequest, UnregisterRequest,
};
use artemis_server::{
    cache::VersionedCacheManager, change::InstanceChangeManager, discovery::DiscoveryServiceImpl,
    lease::LeaseManager, registry::RegistryRepository, RegistryServiceImpl,
};
use artemis_web::state::AppState;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::time;

async fn start_test_server(port: u16) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let repository = RegistryRepository::new();
        let lease_manager = Arc::new(LeaseManager::new(Duration::from_secs(30)));
        let cache = Arc::new(VersionedCacheManager::new());
        let change_manager = Arc::new(InstanceChangeManager::new());

        let registry_service = Arc::new(RegistryServiceImpl::new(
            repository.clone(),
            lease_manager.clone(),
            change_manager.clone(),
        ));
        let discovery_service = Arc::new(DiscoveryServiceImpl::new(repository, cache.clone()));

        let session_manager = Arc::new(artemis_web::websocket::SessionManager::new());

        let app_state = AppState {
            registry_service,
            discovery_service,
            cache,
            session_manager,
        };

        let addr: SocketAddr = format!("127.0.0.1:{}", port).parse().unwrap();
        let _ = artemis_web::server::run_server(app_state, addr).await;
    })
}

#[tokio::test]
async fn test_full_lifecycle() {
    // 启动测试服务器
    let _server = start_test_server(18080).await;
    time::sleep(Duration::from_millis(500)).await; // 等待服务器启动

    // 创建客户端配置
    let config = ClientConfig {
        server_url: "http://127.0.0.1:18080".to_string(),
        heartbeat_interval_secs: 30,
    };

    // 1. 注册实例
    let registry_client = Arc::new(RegistryClient::new(config.clone()));
    let instance = Instance {
        region_id: "test".to_string(),
        zone_id: "zone".to_string(),
        group_id: None,
        service_id: "e2e-service".to_string(),
        instance_id: "e2e-inst-1".to_string(),
        machine_name: None,
        ip: "127.0.0.1".to_string(),
        port: 9090,
        protocol: None,
        url: "http://127.0.0.1:9090".to_string(),
        health_check_url: None,
        status: InstanceStatus::Up,
        metadata: None,
    };

    let reg_req = RegisterRequest {
        instances: vec![instance.clone()],
    };
    let reg_resp = registry_client.register(reg_req).await.unwrap();
    assert_eq!(
        reg_resp.response_status.error_code,
        artemis_core::model::ErrorCode::Success
    );

    // 2. 服务发现
    let discovery_client = Arc::new(DiscoveryClient::new(config));
    time::sleep(Duration::from_millis(100)).await; // 等待注册完成

    let get_service_req = GetServiceRequest {
        discovery_config: DiscoveryConfig {
            service_id: "e2e-service".to_string(),
            region_id: "test".to_string(),
            zone_id: "zone".to_string(),
            discovery_data: None,
        },
    };

    let service = discovery_client.get_service(get_service_req).await.unwrap();
    assert!(service.is_some());
    let service = service.unwrap();
    assert_eq!(service.instances.len(), 1);
    assert_eq!(service.instances[0].instance_id, "e2e-inst-1");

    // 3. 心跳
    let hb_req = HeartbeatRequest {
        instance_keys: vec![instance.key()],
    };
    let hb_resp = registry_client.heartbeat(hb_req).await.unwrap();
    assert_eq!(
        hb_resp.response_status.error_code,
        artemis_core::model::ErrorCode::Success
    );

    // 4. 注销
    let unreg_req = UnregisterRequest {
        instance_keys: vec![instance.key()],
    };
    let unreg_resp = registry_client.unregister(unreg_req).await.unwrap();
    assert_eq!(
        unreg_resp.response_status.error_code,
        artemis_core::model::ErrorCode::Success
    );
}

#[tokio::test]
async fn test_multiple_instances() {
    let _server = start_test_server(18081).await;
    time::sleep(Duration::from_millis(500)).await;

    let config = ClientConfig {
        server_url: "http://127.0.0.1:18081".to_string(),
        heartbeat_interval_secs: 30,
    };

    let registry_client = Arc::new(RegistryClient::new(config.clone()));

    // 注册多个实例
    let instances: Vec<Instance> = (0..5)
        .map(|i| Instance {
            region_id: "test".to_string(),
            zone_id: "zone".to_string(),
            group_id: None,
            service_id: "multi-service".to_string(),
            instance_id: format!("inst-{}", i),
            machine_name: None,
            ip: "127.0.0.1".to_string(),
            port: 9000 + i as u16,
            protocol: None,
            url: format!("http://127.0.0.1:{}", 9000 + i),
            health_check_url: None,
            status: InstanceStatus::Up,
            metadata: None,
        })
        .collect();

    let reg_req = RegisterRequest {
        instances: instances.clone(),
    };
    registry_client.register(reg_req).await.unwrap();

    // 验证服务发现
    let discovery_client = Arc::new(DiscoveryClient::new(config));
    time::sleep(Duration::from_millis(100)).await;

    let get_service_req = GetServiceRequest {
        discovery_config: DiscoveryConfig {
            service_id: "multi-service".to_string(),
            region_id: "test".to_string(),
            zone_id: "zone".to_string(),
            discovery_data: None,
        },
    };

    let service = discovery_client.get_service(get_service_req).await.unwrap();
    assert!(service.is_some());
    assert_eq!(service.unwrap().instances.len(), 5);
}

#[tokio::test]
async fn test_heartbeat_keeps_instance_alive() {
    let _server = start_test_server(18082).await;
    time::sleep(Duration::from_millis(500)).await;

    let config = ClientConfig {
        server_url: "http://127.0.0.1:18082".to_string(),
        heartbeat_interval_secs: 30,
    };

    let registry_client = Arc::new(RegistryClient::new(config.clone()));

    let instance = Instance {
        region_id: "test".to_string(),
        zone_id: "zone".to_string(),
        group_id: None,
        service_id: "heartbeat-service".to_string(),
        instance_id: "hb-inst-1".to_string(),
        machine_name: None,
        ip: "127.0.0.1".to_string(),
        port: 9090,
        protocol: None,
        url: "http://127.0.0.1:9090".to_string(),
        health_check_url: None,
        status: InstanceStatus::Up,
        metadata: None,
    };

    // 注册实例
    let reg_req = RegisterRequest {
        instances: vec![instance.clone()],
    };
    registry_client.register(reg_req).await.unwrap();

    // 发送心跳
    let hb_req = HeartbeatRequest {
        instance_keys: vec![instance.key()],
    };
    let hb_resp = registry_client.heartbeat(hb_req).await.unwrap();
    assert_eq!(
        hb_resp.response_status.error_code,
        artemis_core::model::ErrorCode::Success
    );

    // 验证实例仍然活跃
    let discovery_client = Arc::new(DiscoveryClient::new(config));
    let get_service_req = GetServiceRequest {
        discovery_config: DiscoveryConfig {
            service_id: "heartbeat-service".to_string(),
            region_id: "test".to_string(),
            zone_id: "zone".to_string(),
            discovery_data: None,
        },
    };

    let service = discovery_client.get_service(get_service_req).await.unwrap();
    assert!(service.is_some());
    assert_eq!(service.unwrap().instances.len(), 1);
}
