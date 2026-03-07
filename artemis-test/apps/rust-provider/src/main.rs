use artemis_client::{ClientConfig, RegistryClient};
use artemis_common::model::{Instance, InstanceStatus, RegisterRequest};
use axum::{
    routing::get,
    Router,
};
use chrono::Local;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::Duration;

#[tokio::main]
async fn main() {
    // 获取配置
    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "8089".to_string())
        .parse()
        .expect("Invalid PORT");

    let artemis_servers = std::env::var("ARTEMIS_SERVERS")
        .unwrap_or_else(|_| "http://localhost:8081".to_string());

    let service_name = std::env::var("SERVICE_NAME")
        .unwrap_or_else(|_| "hybrid-test-hello-service".to_string());

    println!("[Rust Provider] Starting on port {}", port);
    println!("[Rust Provider] Artemis servers: {}", artemis_servers);
    println!("[Rust Provider] Service name: {}", service_name);

    // 注册到 Artemis（在后台任务中）
    let servers_clone = artemis_servers.clone();
    let service_clone = service_name.clone();
    let port_clone = port;
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(2)).await;
        if let Err(e) = register_to_artemis(&servers_clone, &service_clone, port_clone).await {
            eprintln!("[Rust Provider] Failed to register: {}", e);
        }
    });

    // 启动 HTTP 服务
    let app = Router::new()
        .route("/sayHello", get(say_hello))
        .route("/health", get(health));

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    println!("[Rust Provider] Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn say_hello() -> String {
    let port = std::env::var("PORT").unwrap_or_else(|_| "8089".to_string());
    let timestamp = Local::now().to_rfc3339();
    format!("Hello from Rust [{}] at {}", port, timestamp)
}

async fn health() -> String {
    "OK".to_string()
}

async fn register_to_artemis(servers: &str, service_name: &str, port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let server_urls: Vec<String> = servers.split(',').map(|s| s.trim().to_string()).collect();

    let config = ClientConfig {
        server_urls,
        heartbeat_interval_secs: 10,
        heartbeat_ttl_secs: 30,
        ..Default::default()
    };

    let registry = RegistryClient::new(config);

    let mut metadata = HashMap::new();
    metadata.insert("app".to_string(), "rust-provider".to_string());
    metadata.insert("language".to_string(), "rust".to_string());

    let instance = Instance {
        region_id: "default".to_string(),
        zone_id: "default".to_string(),
        group_id: None,
        service_id: service_name.to_string(),
        instance_id: format!("rust-web-{}", port),
        machine_name: None,
        ip: "127.0.0.1".to_string(),
        port,
        protocol: Some("http".to_string()),
        url: format!("http://127.0.0.1:{}", port),
        health_check_url: Some(format!("http://127.0.0.1:{}/health", port)),
        status: InstanceStatus::Up,
        metadata: Some(metadata),
    };

    let request = RegisterRequest {
        instances: vec![instance],
    };

    registry.register(request).await?;

    println!("[Rust Provider] Registered to Artemis: port={}, service={}, servers={}", port, service_name, servers);
    Ok(())
}
