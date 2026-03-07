use artemis_client::{ClientConfig, DiscoveryClient};
use artemis_common::model::{DiscoveryConfig, GetServiceRequest};
use reqwest;
use std::env;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[tokio::main]
async fn main() {
    // 获取配置
    let consumer_id = env::var("CONSUMER_ID").unwrap_or_else(|_| "rust-consumer-1".to_string());
    let artemis_servers = env::var("ARTEMIS_SERVERS")
        .unwrap_or_else(|_| "http://localhost:8081".to_string());
    let service_name = env::var("SERVICE_NAME")
        .unwrap_or_else(|_| "hybrid-test-hello-service".to_string());

    println!("[{}] Starting...", consumer_id);
    println!("[{}] Artemis servers: {}", consumer_id, artemis_servers);
    println!("[{}] Service name: {}", consumer_id, service_name);

    // 配置 DiscoveryClient
    let server_urls: Vec<String> = artemis_servers
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();

    let config = ClientConfig {
        server_urls,
        ..Default::default()
    };

    let discovery = Arc::new(DiscoveryClient::new(config));

    let round_robin = Arc::new(AtomicUsize::new(0));

    // 每 200ms 调用一次服务
    let mut interval = tokio::time::interval(Duration::from_millis(200));

    println!("[{}] Starting job loop (every 200ms)...", consumer_id);

    loop {
        interval.tick().await;

        let discovery = discovery.clone();
        let round_robin = round_robin.clone();
        let consumer_id = consumer_id.clone();
        let service_name = service_name.clone();

        tokio::spawn(async move {
            match run_job(discovery, round_robin, &consumer_id, &service_name).await {
                Ok(_) => {}
                Err(e) => eprintln!("[{}] Error: {}", consumer_id, e),
            }
        });
    }
}

async fn run_job(
    discovery: Arc<DiscoveryClient>,
    round_robin: Arc<AtomicUsize>,
    consumer_id: &str,
    service_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // 发现服务实例
    let request = GetServiceRequest {
        discovery_config: DiscoveryConfig {
            service_id: service_name.to_string(),
            region_id: "default".to_string(),
            zone_id: "default".to_string(),
            discovery_data: None,
        },
    };
    let service = discovery.get_service(request).await?;

    let instances = match service {
        Some(s) => s.instances,
        None => {
            return Err("No instances available".into());
        }
    };

    if instances.is_empty() {
        return Err("No instances available".into());
    }

    // 轮询选择
    let index = round_robin.fetch_add(1, Ordering::Relaxed) % instances.len();
    let target = &instances[index];

    // 构建 URL
    let url = format!("http://{}:{}/sayHello", target.ip, target.port);

    // 发起调用
    let start = Instant::now();
    let response = reqwest::get(&url).await?;
    let latency = start.elapsed();

    if response.status().is_success() {
        let body = response.text().await?;
        println!(
            "[{}] Target={}:{} Response={} Latency={:?}",
            consumer_id, target.ip, target.port, body, latency
        );
    } else {
        eprintln!(
            "[{}] HTTP {} from {}:{}",
            consumer_id,
            response.status(),
            target.ip,
            target.port
        );
    }

    Ok(())
}
