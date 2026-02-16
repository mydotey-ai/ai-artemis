//! Artemis 压力测试工具
//!
//! 功能:
//! - 高并发客户端模拟 (100+ 并发)
//! - 10,000+ QPS 压力测试
//! - 延迟分布统计 (P50/P90/P99/P99.9)
//! - 吞吐量监控
//! - 错误率统计
//! - 实时进度显示

use artemis_core::model::{HeartbeatRequest, Instance, InstanceKey, InstanceStatus, RegisterRequest};
use clap::Parser;
use hdrhistogram::Histogram;
use indicatif::{ProgressBar, ProgressStyle};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

#[derive(Parser, Debug)]
#[command(name = "artemis-stress-test")]
#[command(about = "Artemis 压力测试工具", long_about = None)]
struct Args {
    /// Artemis 服务器地址
    #[arg(short, long, default_value = "http://localhost:8080")]
    url: String,

    /// 并发客户端数量
    #[arg(short, long, default_value = "100")]
    concurrency: usize,

    /// 目标 QPS (每秒请求数)
    #[arg(short, long, default_value = "10000")]
    qps: u64,

    /// 测试持续时间 (秒)
    #[arg(short, long, default_value = "60")]
    duration: u64,

    /// 每个客户端的实例数
    #[arg(short, long, default_value = "10")]
    instances_per_client: usize,

    /// 测试模式: register, heartbeat, discovery, mixed
    #[arg(short, long, default_value = "mixed")]
    mode: String,
}

struct StressTestStats {
    total_requests: u64,
    successful_requests: u64,
    failed_requests: u64,
    latency_histogram: Histogram<u64>,
}

impl StressTestStats {
    fn new() -> Self {
        Self {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            latency_histogram: Histogram::<u64>::new(3).unwrap(),
        }
    }

    fn record_success(&mut self, latency_us: u64) {
        self.total_requests += 1;
        self.successful_requests += 1;
        let _ = self.latency_histogram.record(latency_us);
    }

    fn record_failure(&mut self) {
        self.total_requests += 1;
        self.failed_requests += 1;
    }

    fn print_summary(&self, duration_secs: u64) {
        println!("\n=== 压力测试结果 ===");
        println!("总请求数: {}", self.total_requests);
        println!("成功请求: {} ({:.2}%)",
            self.successful_requests,
            self.successful_requests as f64 / self.total_requests as f64 * 100.0
        );
        println!("失败请求: {} ({:.2}%)",
            self.failed_requests,
            self.failed_requests as f64 / self.total_requests as f64 * 100.0
        );
        println!("实际 QPS: {:.2}", self.total_requests as f64 / duration_secs as f64);

        if self.successful_requests > 0 {
            println!("\n=== 延迟分布 (微秒) ===");
            println!("P50:   {} µs", self.latency_histogram.value_at_quantile(0.50));
            println!("P90:   {} µs", self.latency_histogram.value_at_quantile(0.90));
            println!("P95:   {} µs", self.latency_histogram.value_at_quantile(0.95));
            println!("P99:   {} µs", self.latency_histogram.value_at_quantile(0.99));
            println!("P99.9: {} µs", self.latency_histogram.value_at_quantile(0.999));
            println!("Min:   {} µs", self.latency_histogram.min());
            println!("Max:   {} µs", self.latency_histogram.max());
            println!("Mean:  {:.2} µs", self.latency_histogram.mean());
        }
    }
}

async fn register_instances(
    client: &reqwest::Client,
    base_url: &str,
    instances: Vec<Instance>,
) -> Result<Duration, reqwest::Error> {
    let request = RegisterRequest { instances };
    let start = Instant::now();

    client
        .post(format!("{}/api/registry/register.json", base_url))
        .json(&request)
        .send()
        .await?
        .error_for_status()?;

    Ok(start.elapsed())
}

async fn heartbeat_instances(
    client: &reqwest::Client,
    base_url: &str,
    instance_keys: Vec<InstanceKey>,
) -> Result<Duration, reqwest::Error> {
    let request = HeartbeatRequest { instance_keys };
    let start = Instant::now();

    client
        .post(format!("{}/api/registry/heartbeat.json", base_url))
        .json(&request)
        .send()
        .await?
        .error_for_status()?;

    Ok(start.elapsed())
}

fn create_test_instances(client_id: usize, count: usize) -> Vec<Instance> {
    (0..count)
        .map(|i| Instance {
            region_id: "stress-test-region".to_string(),
            zone_id: "stress-test-zone".to_string(),
            service_id: "stress-test-service".to_string(),
            group_id: None,
            instance_id: format!("client-{}-inst-{}", client_id, i),
            machine_name: None,
            ip: format!("192.168.{}.{}", (client_id % 255), (i % 255)),
            port: 8080,
            protocol: None,
            url: format!("http://192.168.{}.{}:8080", (client_id % 255), (i % 255)),
            health_check_url: None,
            status: InstanceStatus::Up,
            metadata: None,
        })
        .collect()
}

async fn run_stress_test(args: Args) {
    println!("=== Artemis 压力测试启动 ===");
    println!("服务器: {}", args.url);
    println!("并发数: {}", args.concurrency);
    println!("目标 QPS: {}", args.qps);
    println!("持续时间: {} 秒", args.duration);
    println!("每客户端实例数: {}", args.instances_per_client);
    println!("测试模式: {}", args.mode);
    println!();

    let stats = Arc::new(Mutex::new(StressTestStats::new()));
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .unwrap();

    // 进度条
    let pb = ProgressBar::new(args.duration);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len}s ({msg})")
            .unwrap()
            .progress_chars("#>-"),
    );

    // 计算每个客户端的请求间隔
    let requests_per_second_per_client = args.qps / args.concurrency as u64;
    let interval_nanos = if requests_per_second_per_client > 0 {
        1_000_000_000 / requests_per_second_per_client
    } else {
        1_000_000 // 1ms
    };

    let start_time = Instant::now();
    let test_duration = Duration::from_secs(args.duration);

    // 启动并发客户端
    let mut handles = vec![];
    for client_id in 0..args.concurrency {
        let client = client.clone();
        let base_url = args.url.clone();
        let mode = args.mode.clone();
        let instances_count = args.instances_per_client;
        let stats = stats.clone();
        let interval = Duration::from_nanos(interval_nanos);

        let handle = tokio::spawn(async move {
            // 创建测试实例
            let instances = create_test_instances(client_id, instances_count);
            let instance_keys: Vec<InstanceKey> = instances.iter().map(|i| i.key()).collect();

            // 先注册所有实例
            let _ = register_instances(&client, &base_url, instances.clone()).await;

            let mut request_count = 0u64;
            loop {
                if start_time.elapsed() >= test_duration {
                    break;
                }

                let iteration_start = Instant::now();

                // 根据模式执行不同的操作
                let result = match mode.as_str() {
                    "register" => register_instances(&client, &base_url, instances.clone()).await,
                    "heartbeat" => heartbeat_instances(&client, &base_url, instance_keys.clone()).await,
                    "mixed" => {
                        if request_count % 10 == 0 {
                            register_instances(&client, &base_url, instances.clone()).await
                        } else {
                            heartbeat_instances(&client, &base_url, instance_keys.clone()).await
                        }
                    }
                    _ => heartbeat_instances(&client, &base_url, instance_keys.clone()).await,
                };

                // 记录结果
                let mut stats = stats.lock().await;
                match result {
                    Ok(latency) => {
                        stats.record_success(latency.as_micros() as u64);
                    }
                    Err(_) => {
                        stats.record_failure();
                    }
                }
                drop(stats);

                request_count += 1;

                // 速率限制
                let elapsed = iteration_start.elapsed();
                if elapsed < interval {
                    tokio::time::sleep(interval - elapsed).await;
                }
            }
        });

        handles.push(handle);
    }

    // 进度更新
    let pb_handle = tokio::spawn({
        let pb = pb.clone();
        async move {
            while start_time.elapsed() < test_duration {
                let elapsed_secs = start_time.elapsed().as_secs();
                pb.set_position(elapsed_secs);
                pb.set_message(format!("进行中..."));
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            pb.finish_with_message("完成!");
        }
    });

    // 等待所有客户端完成
    for handle in handles {
        let _ = handle.await;
    }
    let _ = pb_handle.await;

    // 打印结果
    let stats = stats.lock().await;
    let actual_duration = start_time.elapsed().as_secs();
    stats.print_summary(actual_duration);
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    run_stress_test(args).await;
}
