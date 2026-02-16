# 阶段8: 集成测试和生产就绪

> **For Claude:** 集成测试、性能测试、生产就绪特性

**优先级**: P0 (必须完成)
**状态**: ✅ **已完成** (2026-02-13)
**目标:** 确保系统可以投入生产使用
**任务数:** 5个Task

---

## Task 8.1: 端到端集成测试

**Files:**
- Create: `tests/e2e_test.rs`

**Step 1: 创建workspace级别的集成测试**

```rust
// tests/e2e_test.rs
use artemis_client::{ClientConfig, DiscoveryClient, RegistryClient};
use artemis_core::config::ArtemisConfig;
use artemis_core::model::{Instance, InstanceStatus};
use artemis_server::{
    cache::VersionedCacheManager, discovery::DiscoveryServiceImpl, lease::LeaseManager,
    ratelimiter::RateLimiter, registry::RegistryRepository, registry::RegistryServiceImpl,
};
use artemis_web::{AppState, WebServer};
use std::sync::Arc;
use std::time::Duration;
use tokio::time;

async fn start_test_server(port: u16) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let config = ArtemisConfig {
            server: artemis_core::config::ServerConfig {
                host: "127.0.0.1".to_string(),
                port,
                region_id: "test".to_string(),
                zone_id: "zone".to_string(),
            },
            registry: artemis_core::config::RegistryConfig {
                lease_ttl: Duration::from_secs(30),
                eviction_interval: Duration::from_secs(10),
                rate_limit_rps: 1000,
            },
            cluster: artemis_core::config::ClusterConfig {
                enabled: false,
                peer_nodes: None,
            },
            database: None,
        };

        let repository = RegistryRepository::new();
        let lease_manager = Arc::new(LeaseManager::new(config.registry.lease_ttl));
        let cache = Arc::new(VersionedCacheManager::new());
        let rate_limiter = RateLimiter::new(config.registry.rate_limit_rps);

        let registry_service = RegistryServiceImpl::new(repository.clone(), lease_manager.clone());
        let discovery_service = DiscoveryServiceImpl::new(repository, cache.clone());

        let app_state = AppState::new(registry_service, discovery_service, rate_limiter, cache);
        let server = WebServer::new(&config.server.host, config.server.port, app_state);

        let _ = server.run().await;
    })
}

#[tokio::test]
async fn test_full_lifecycle() {
    // 启动测试服务器
    let _server = start_test_server(18080).await;
    time::sleep(Duration::from_millis(500)).await; // 等待服务器启动

    // 创建客户端配置
    let config = ClientConfig::builder()
        .server_url("http://127.0.0.1:18080")
        .region_id("test")
        .zone_id("zone")
        .build();

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

    let reg_resp = registry_client.register(vec![instance.clone()]).await.unwrap();
    assert_eq!(
        reg_resp.response_status.error_code,
        artemis_core::model::ErrorCode::Success
    );

    // 2. 服务发现
    let discovery_client = Arc::new(DiscoveryClient::new(config));
    time::sleep(Duration::from_millis(100)).await; // 等待注册完成

    let service = discovery_client.get_service("e2e-service").await.unwrap();
    assert!(service.is_some());
    let service = service.unwrap();
    assert_eq!(service.instances.len(), 1);
    assert_eq!(service.instances[0].instance_id, "e2e-inst-1");

    // 3. 心跳
    let hb_resp = registry_client
        .heartbeat(vec![instance.key()])
        .await
        .unwrap();
    assert_eq!(
        hb_resp.response_status.error_code,
        artemis_core::model::ErrorCode::Success
    );

    // 4. 注销
    let unreg_resp = registry_client
        .unregister(vec![instance.key()])
        .await
        .unwrap();
    assert_eq!(
        unreg_resp.response_status.error_code,
        artemis_core::model::ErrorCode::Success
    );
}

#[tokio::test]
async fn test_multiple_instances() {
    let _server = start_test_server(18081).await;
    time::sleep(Duration::from_millis(500)).await;

    let config = ClientConfig::builder()
        .server_url("http://127.0.0.1:18081")
        .region_id("test")
        .zone_id("zone")
        .build();

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

    registry_client.register(instances).await.unwrap();

    // 验证服务发现
    let discovery_client = Arc::new(DiscoveryClient::new(config));
    time::sleep(Duration::from_millis(100)).await;

    let service = discovery_client.get_service("multi-service").await.unwrap();
    assert!(service.is_some());
    assert_eq!(service.unwrap().instances.len(), 5);
}
```

**Step 2: 运行集成测试**

```bash
cargo test --test e2e_test
```

Expected: 所有测试通过

**Step 3: 提交**

```bash
git add tests/
git commit -m "test: add end-to-end integration tests

- Test full lifecycle: register -> discover -> heartbeat -> unregister
- Test multiple instances registration
- Start test server in background
- Verify client-server integration

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 8.2: 性能基准测试

**Files:**
- Create: `benches/performance.rs`
- Update: `Cargo.toml`

**Step 1: 添加criterion依赖**

更新workspace `Cargo.toml`:

```toml
[workspace.dependencies]
# ... existing dependencies ...
criterion = { version = "0.5", features = ["async_tokio"] }
```

**Step 2: 创建性能测试**

```rust
// benches/performance.rs
use artemis_core::model::{Instance, InstanceStatus, RegisterRequest};
use artemis_core::traits::RegistryService;
use artemis_server::{
    lease::LeaseManager, registry::RegistryRepository, registry::RegistryServiceImpl,
};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::sync::Arc;
use std::time::Duration;

fn create_test_instance(id: u32) -> Instance {
    Instance {
        region_id: "bench".to_string(),
        zone_id: "zone".to_string(),
        group_id: None,
        service_id: format!("service-{}", id % 100),
        instance_id: format!("inst-{}", id),
        machine_name: None,
        ip: "127.0.0.1".to_string(),
        port: 8000 + (id % 1000) as u16,
        protocol: None,
        url: format!("http://127.0.0.1:{}", 8000 + id % 1000),
        health_check_url: None,
        status: InstanceStatus::Up,
        metadata: None,
    }
}

fn benchmark_register(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("register_single_instance", |b| {
        b.iter(|| {
            rt.block_on(async {
                let repo = RegistryRepository::new();
                let lease_mgr = Arc::new(LeaseManager::new(Duration::from_secs(30)));
                let service = RegistryServiceImpl::new(repo, lease_mgr);

                let request = RegisterRequest {
                    instances: vec![create_test_instance(1)],
                };
                black_box(service.register(request).await);
            })
        })
    });

    c.bench_function("register_100_instances", |b| {
        b.iter(|| {
            rt.block_on(async {
                let repo = RegistryRepository::new();
                let lease_mgr = Arc::new(LeaseManager::new(Duration::from_secs(30)));
                let service = RegistryServiceImpl::new(repo, lease_mgr);

                let instances: Vec<Instance> = (0..100).map(create_test_instance).collect();
                let request = RegisterRequest { instances };
                black_box(service.register(request).await);
            })
        })
    });
}

fn benchmark_repository(c: &mut Criterion) {
    c.bench_function("repository_concurrent_writes", |b| {
        b.iter(|| {
            let repo = RegistryRepository::new();
            let handles: Vec<_> = (0..1000)
                .map(|i| {
                    let repo = repo.clone();
                    std::thread::spawn(move || {
                        repo.register(create_test_instance(i));
                    })
                })
                .collect();

            for handle in handles {
                handle.join().unwrap();
            }
        })
    });
}

criterion_group!(benches, benchmark_register, benchmark_repository);
criterion_main!(benches);
```

**Step 3: 更新workspace Cargo.toml**

```toml
# Cargo.toml (在文件末尾添加)
[[bench]]
name = "performance"
harness = false
```

**Step 4: 添加artemis-server依赖到bench**

创建 `Cargo.toml` 在workspace根目录（如果还没有bench配置）:

```toml
[dev-dependencies]
criterion = { workspace = true }
artemis-core = { path = "artemis-core" }
artemis-server = { path = "artemis-server" }
tokio = { workspace = true }
```

**Step 5: 运行基准测试**

```bash
cargo bench
```

Expected: 生成性能报告

**Step 6: 提交**

```bash
git add benches/ Cargo.toml
git commit -m "perf: add performance benchmarks

- Benchmark single and batch registration
- Benchmark concurrent repository writes
- Use criterion for accurate measurements

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 8.3: 添加健康检查和指标

**Files:**
- Update: `artemis-web/Cargo.toml`
- Create: `artemis-web/src/api/metrics.rs`

**Step 1: 添加prometheus依赖**

```toml
# artemis-web/Cargo.toml
[dependencies]
# ... existing dependencies ...
prometheus = "0.13"
lazy_static = "1.4"
```

**Step 2: 实现指标收集**

```rust
// artemis-web/src/api/metrics.rs
use axum::{http::StatusCode, response::IntoResponse};
use lazy_static::lazy_static;
use prometheus::{register_int_counter, register_int_gauge, Encoder, IntCounter, IntGauge, TextEncoder};

lazy_static! {
    pub static ref REGISTER_REQUESTS: IntCounter =
        register_int_counter!("artemis_register_requests_total", "Total register requests").unwrap();
    pub static ref HEARTBEAT_REQUESTS: IntCounter =
        register_int_counter!("artemis_heartbeat_requests_total", "Total heartbeat requests").unwrap();
    pub static ref DISCOVERY_REQUESTS: IntCounter =
        register_int_counter!("artemis_discovery_requests_total", "Total discovery requests").unwrap();
    pub static ref ACTIVE_INSTANCES: IntGauge =
        register_int_gauge!("artemis_active_instances", "Number of active instances").unwrap();
}

pub async fn metrics() -> impl IntoResponse {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = Vec::new();

    if let Err(e) = encoder.encode(&metric_families, &mut buffer) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to encode metrics: {}", e),
        )
            .into_response();
    }

    (StatusCode::OK, buffer).into_response()
}
```

**Step 3: 更新路由添加metrics端点**

更新 `artemis-web/src/server.rs`:

```rust
// 在路由中添加
.route("/metrics", get(api::metrics::metrics))
```

**Step 4: 更新api/mod.rs**

```rust
// artemis-web/src/api/mod.rs
pub mod discovery;
pub mod health;
pub mod metrics;
pub mod registry;
```

**Step 5: 提交**

```bash
git add artemis-web/
git commit -m "feat(web): add Prometheus metrics

- Add register/heartbeat/discovery request counters
- Add active instances gauge
- Add /metrics endpoint for Prometheus scraping

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 8.4: 添加优雅关闭

**Files:**
- Update: `artemis/src/commands/server.rs`

**Step 1: 实现优雅关闭**

```rust
// artemis/src/commands/server.rs
use tokio::signal;

pub async fn run_server(config_path: &str) -> Result<()> {
    // ... 现有的初始化代码 ...

    // 创建关闭信号
    let shutdown_signal = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install CTRL+C signal handler");
        info!("Received shutdown signal");
    };

    // 启动服务器（使用with_graceful_shutdown）
    let server = WebServer::new(&config.server.host, config.server.port, app_state);

    info!("Server starting on {}:{}", config.server.host, config.server.port);

    // 如果WebServer支持graceful shutdown，使用它
    // 否则需要修改WebServer::run方法来支持
    tokio::select! {
        result = server.run() => {
            if let Err(e) = result {
                error!("Server error: {}", e);
            }
        }
        _ = shutdown_signal => {
            info!("Shutting down gracefully...");
        }
    }

    info!("Server stopped");
    Ok(())
}
```

**Step 2: 更新WebServer支持优雅关闭**

修改 `artemis-web/src/server.rs`:

```rust
pub async fn run(self) -> anyhow::Result<()> {
    info!("Starting Artemis Web Server on {}", self.addr);
    let listener = tokio::net::TcpListener::bind(self.addr).await?;

    axum::serve(listener, self.app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    use tokio::signal;

    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("Signal received, starting graceful shutdown");
}
```

**Step 3: 提交**

```bash
git add artemis/src/commands/server.rs artemis-web/src/server.rs
git commit -m "feat: add graceful shutdown support

- Handle CTRL+C and SIGTERM signals
- Clean shutdown of web server
- Log shutdown events

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 8.5: 创建Docker镜像和文档

**Files:**
- Create: `Dockerfile`
- Create: `.dockerignore`
- Create: `docs/deployment.md`

**Step 1: 创建Dockerfile**

```dockerfile
# Dockerfile
FROM rust:1.85 as builder

WORKDIR /app
COPY . .

RUN cargo build --release -p artemis

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/artemis /usr/local/bin/artemis
COPY artemis.toml.example /etc/artemis/artemis.toml

EXPOSE 8080

CMD ["artemis", "server", "--config", "/etc/artemis/artemis.toml"]
```

**Step 2: 创建.dockerignore**

```
# .dockerignore
target/
.git/
.gitignore
*.md
artemis-java/
.worktrees/
docs/
benches/
tests/
```

**Step 3: 创建部署文档**

```markdown
# Artemis 部署指南

## Docker部署

### 构建镜像

\`\`\`bash
docker build -t artemis:latest .
\`\`\`

### 运行容器

\`\`\`bash
docker run -d \
  --name artemis \
  -p 8080:8080 \
  -v /path/to/artemis.toml:/etc/artemis/artemis.toml \
  artemis:latest
\`\`\`

### Docker Compose

\`\`\`yaml
version: '3.8'

services:
  artemis:
    image: artemis:latest
    ports:
      - "8080:8080"
    volumes:
      - ./artemis.toml:/etc/artemis/artemis.toml
    environment:
      - RUST_LOG=info
    restart: unless-stopped
\`\`\`

## 性能调优

### 推荐配置

- CPU: 2核心以上
- 内存: 2GB以上（支持10万实例）
- 网络: 1Gbps

### 配置优化

\`\`\`toml
[registry]
lease_ttl = "30s"          # 租约时长
eviction_interval = "10s"   # 清理间隔
rate_limit_rps = 10000      # 限流QPS

[server]
port = 8080
\`\`\`

## 监控

### Prometheus

访问 `http://localhost:8080/metrics` 获取指标

关键指标:
- `artemis_register_requests_total` - 注册请求总数
- `artemis_heartbeat_requests_total` - 心跳请求总数
- `artemis_discovery_requests_total` - 发现请求总数
- `artemis_active_instances` - 活跃实例数

### 健康检查

\`\`\`bash
curl http://localhost:8080/health
\`\`\`

## 高可用部署

TODO: 集群模式配置
```

**Step 4: 创建README**

更新根目录 `README.md`:

```markdown
# Artemis - Service Registry in Rust

高性能服务注册中心，Rust重写版本。

## 特性

- ⚡️ **高性能**: P99延迟 < 10ms
- 🔒 **无锁设计**: 使用DashMap实现零GC
- 📊 **可观测**: Prometheus指标导出
- 🔄 **实时更新**: WebSocket推送
- 🛡️ **限流保护**: Token bucket限流
- 🐳 **容器化**: Docker支持

## 快速开始

### 安装

\`\`\`bash
cargo build --release -p artemis
\`\`\`

### 启动服务器

\`\`\`bash
./target/release/artemis server --config artemis.toml
\`\`\`

### 使用客户端SDK

参见 `artemis-client/examples/simple_client.rs`

## 文档

- [部署指南](docs/deployment.md)
- [产品规格](docs/artemis-rust-rewrite-specification.md)
- [详细设计](docs/plans/2026-02-13-artemis-rust-design.md)

## 许可证

MIT OR Apache-2.0
```

**Step 5: 提交**

```bash
git add Dockerfile .dockerignore docs/deployment.md README.md
git commit -m "docs: add Docker support and deployment guide

- Add multi-stage Dockerfile
- Add .dockerignore
- Create deployment guide with Docker/monitoring
- Update README with quick start

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## 阶段8完成标准

- ✅ 端到端集成测试
- ✅ 性能基准测试
- ✅ Prometheus指标
- ✅ 优雅关闭
- ✅ Docker支持
- ✅ 部署文档
- ✅ `cargo test` 全部通过
- ✅ `cargo bench` 生成报告
- ✅ Docker镜像构建成功
