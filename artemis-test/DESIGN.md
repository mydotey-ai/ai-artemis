# Artemis Java/Rust 混合集群集成测试设计方案

> 文档编号: ARTEMIS-HT-001
> 版本: 1.0
> 日期: 2026-03-03

## 1. 设计目标

### 1.1 测试目标

- **集群互通性**: 验证 Java 节点和 Rust 节点可以组成对等集群，数据通过 replication API 双向同步
- **客户端互通**: 验证 Java Client 和 Rust Client 可以互相注册、发现、调用服务
- **Management 共享**: 验证 management 配置通过共享 SQLite 数据库同步
- **负载均衡**: 验证简单的轮询负载均衡在多客户端、多服务端场景下工作正常

### 1.2 成功标准

- [x] 6 节点集群（3 Java + 3 Rust）可以同时启动并互相发现
- [x] 在任一节点注册的服务，可以在所有节点被发现
- [x] Java 客户端注册的服务可以被 Rust 客户端发现并调用
- [x] 4 个 job 应用（每 200ms 一次调用）持续 10 分钟无错误
- [x] Web Console 可以管理所有节点，查看集群状态

---

## 2. 架构设计

### 2.1 集群拓扑

```
┌─────────────────────────────────────────────────────────────────┐
│                      Artemis Hybrid Cluster                      │
├─────────────────────────────────────────────────────────────────┤
│                                                                   │
│   ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐   │
│   │   Java Node 1   │  │   Java Node 2   │  │   Java Node 3   │   │
│   │   :8081         │  │   :8082         │  │   :8083         │   │
│   └────────┬────────┘  └────────┬────────┘  └────────┬────────┘   │
│            │                    │                    │             │
│            │    ┌───────────────┼───────────────────┐ │             │
│            │    │               │                   │ │             │
│            ▼    ▼               ▼                   ▼ ▼             │
│   ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐   │
│   │   Rust Node 4   │  │   Rust Node 5   │  │   Rust Node 6   │   │
│   │   :8084         │  │   :8085         │  │   :8086         │   │
│   └─────────────────┘  └─────────────────┘  └─────────────────┘   │
│                                                                   │
│   ┌──────────────────────────────────────────────────────────┐   │
│   │           Shared SQLite Database: artemis.db            │   │
│   │  - Groups, RouteRules, ZoneOperations, CanaryConfigs     │   │
│   │  - Management data (NOT instance/lease data)             │   │
│   └──────────────────────────────────────────────────────────┘   │
│                                                                   │
└─────────────────────────────────────────────────────────────────┘
```

### 2.2 端口分配

| 节点 | 类型 | HTTP 端口 | Peer 端口 (内部) | 用途 |
|------|------|----------|-----------------|------|
| node-j1 | Java | 8081 | 9091 | Java 节点 1 |
| node-j2 | Java | 8082 | 9092 | Java 节点 2 |
| node-j3 | Java | 8083 | 9093 | Java 节点 3 |
| node-r1 | Rust | 8084 | 9094 | Rust 节点 1 |
| node-r2 | Rust | 8085 | 9095 | Rust 节点 2 |
| node-r3 | Rust | 8086 | 9096 | Rust 节点 3 |
| console | Rust | 5173 | - | Web Console |

### 2.3 测试应用拓扑

```
┌─────────────────────────────────────────────────────────────────┐
│                       Test Applications                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                   │
│   ┌───────────────────────────────────────────────────────────┐   │
│   │                   Service Providers                       │   │
│   ├─────────────────────┬─────────────────────────────────────┤   │
│   │                     │                                     │   │
│   │  ┌─────────────┐   │   ┌─────────────┐                  │   │
│   │  │  Java Web 1 │   │   │  Rust Web 1 │                  │   │
│   │  │  :8087      │   │   │  :8089      │                  │   │
│   │  └──────┬──────┘   │   └──────┬──────┘                  │   │
│   │         │         │          │                         │   │
│   │  ┌─────────────┐   │   ┌─────────────┐                  │   │
│   │  │  Java Web 2 │   │   │  Rust Web 2 │                  │   │
│   │  │  :8088      │   │   │  :8090      │                  │   │
│   │  └──────┬──────┘   │   └──────┬──────┘                  │   │
│   │         │         │          │                         │   │
│   │  ┌──────▼──────┬──┴──┬───────▼────────┐                │   │
│   │  │              Service Name:          │                │   │
│   │  │      "hybrid-test-hello-service"   │                │   │
│   │  │              4 Instances          │                │   │
│   │  └─────────────────────────────────────┘                │   │
│   │                                                       │   │
│   └───────────────────────────────────────────────────────┘   │
│                                                               │
│   ┌───────────────────────────────────────────────────────────┐
│   │                   Service Consumers (Jobs)                │
│   ├─────────────────────┬─────────────────────────────────────┤
│   │                     │                                     │
│   │  ┌─────────────┐   │   ┌─────────────┐                  │
│   │  │  Java Job 1 │   │   │  Rust Job 1 │                  │
│   │  │  200ms/req  │   │   │  200ms/req  │                  │
│   │  └──────┬──────┘   │   └──────┬──────┘                  │
│   │         │         │          │                         │
│   │  ┌─────────────┐   │   ┌─────────────┐                  │
│   │  │  Java Job 2 │   │   │  Rust Job 2 │                  │
│   │  │  200ms/req  │   │   │  200ms/req  │                  │
│   │  └──────┬──────┘   │   └──────┬──────┘                  │
│   │         │         │          │                         │
│   │  ┌──────▼────────┴──┬────────▼────────┐                │
│   │  │                                  │                │
│   │  │  负载均衡: 简单轮询 (Round Robin)  │                │
│   │  │  每 200ms 调用一次 sayHello      │                │
│   │  │  每轮循环遍历 4 个 Provider       │                │
│   │  │                                  │                │
│   │  └──────────────────────────────────┘                │
│   │                                                       │
│   └───────────────────────────────────────────────────────┘
│
│   API: GET /sayHello → Response: "Hello from [Java/Rust] [Port] at [Timestamp]"
│
└─────────────────────────────────────────────────────────────────┘
```

---

## 3. 技术方案

### 3.1 集群对等复制协议

#### 关键问题：Java 和 Rust 的 replication API 是否兼容？

**分析：**
- Java 版本的 `/api/replication/*` 端点：需要查看 artemis-java 的 ReplicationController
- Rust 版本的 `/api/replication/*` 端点：需要查看 artemis-server/src/api/replication.rs

**预期方案：**
1. 首先验证两端点参数格式是否一致
2. 如果不完全一致，使用 adapter 层或配置兼容模式

### 3.2 Management 共享方案

```
Shared SQLite Database (artemis.db)
├── groups          - Group 配置
├── route_rules     - 路由规则
├── zone_operations - Zone 操作记录
├── canary_configs  - 灰度配置
├── audit_logs      - 审计日志
└── ...

NOT shared (in-memory):
├── instances       - 服务实例（通过 replication 同步）
├── leases          - 租约（通过 replication 同步）
├── services        - 服务元数据（通过 replication 同步）
```

### 3.3 测试应用技术选型

| 应用 | 语言 | 技术 | 端口 | 备注 |
|------|------|------|------|------|
| Java Web 1-2 | Java 17 | Spring Boot 3 + artemis-java-client | 8087, 8088 | 完整依赖 artemis-java 客户端 |
| Rust Web 1-2 | Rust | Axum + artemis-client crate | 8089, 8090 | 使用 workspace 的本地 crate |
| Java Job 1-2 | Java 17 | artemis-java-client + REST client | - | 控制台应用，定时调用 |
| Rust Job 1-2 | Rust | artemis-client + reqwest | - | 控制台应用，tokio schedule |

---

## 4. 详细设计方案

### 4.1 目录结构

```
artemis-test/
├── DESIGN.md                    # 本设计文档
├── README.md                      # 快速开始指南
├── config/                        # 配置文件
│   ├── java-nodes.yml           # Java 节点配置 (3个)
│   ├── rust-nodes.toml          # Rust 节点配置 (3个)
│   └── shared-db.properties     # 共享数据库配置
├── apps/                          # 测试应用源码
│   ├── java-provider/           # Java Web 服务 (Spring Boot)
│   │   ├── pom.xml
│   │   └── src/
│   │       └── main/
│   │           ├── java/.../HelloServiceApplication.java
│   │           └── resources/application.yml
│   ├── java-consumer/           # Java Job 应用
│   │   └── src/.../HelloJobApplication.java
│   ├── rust-provider/           # Rust Web 服务 (Cargo crate)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── main.rs
│   └── rust-consumer/           # Rust Job 应用
│       ├── Cargo.toml
│       └── src/
│           └── main.rs
├── scripts/                       # 测试脚本
│   ├── setup.sh                 # 环境准备 (编译、打包)
│   ├── start-cluster.sh         # 启动 6 节点集群
│   ├── start-apps.sh            # 启动 8 个测试应用
│   ├── start-console.sh         # 启动 Web Console
│   ├── run-test.sh              # 运行测试 (监控指标)
│   └── cleanup.sh               # 清理资源
├── logs/                          # 日志目录 (gitignore)
├── data/                          # 数据目录 (SQLite, gitignore)
└── reports/                       # 测试报告 (gitignore)
```

### 4.2 集群节点配置

#### Java 节点 1 (node-j1:8081)

```yaml
# artemis-test/config/java-node-j1.yml
server:
  port: 8081
  peer-port: 9091

spring:
  datasource:
    url: jdbc:sqlite:../data/artemis.db
  artemis:
    cluster:
      peers:
        - 127.0.0.1:8082  # node-j2
        - 127.0.0.1:8083  # node-j3
        - 127.0.0.1:8084  # node-r1
        - 127.0.0.1:8085  # node-r2
        - 127.0.0.1:8086  # node-r3
```

#### Rust 节点 1 (node-r1:8084)

```toml
# artemis-test/config/rust-node-r1.toml
[server]
node_id = "node-r1"
listen_addr = "0.0.0.0:8084"
peer_port = 9094

[database]
db_type = "sqlite"
url = "sqlite://../data/artemis.db"

[cluster]
enabled = true
peers = [
    "127.0.0.1:8081",  # node-j1
    "127.0.0.1:8082",  # node-j2
    "127.0.0.1:8083",  # node-j3
    "127.0.0.1:8085",  # node-r2
    "127.0.0.1:8086",  # node-r3
]
```

### 4.3 Java Web Provider (Spring Boot)

```java
// artemis-test/apps/java-provider/src/.../HelloServiceApplication.java
@SpringBootApplication
@RestController
public class HelloServiceApplication {

    @Value("${server.port}")
    private int port;

    @Autowired
    private ArtemisClientManager artemisManager;

    public static void main(String[] args) {
        SpringApplication.run(HelloServiceApplication.class, args);
    }

    @GetMapping("/sayHello")
    public String sayHello() {
        String timestamp = LocalDateTime.now().format(DateTimeFormatter.ISO_LOCAL_DATE_TIME);
        return String.format("Hello from Java [%d] at %s", port, timestamp);
    }

    @PostConstruct
    public void register() {
        // 注册到 Artemis
        Instance instance = new Instance()
            .setServiceId("hybrid-test-hello-service")
            .setInstanceId("java-web-" + port)
            .setHost("127.0.0.1")
            .setPort(port)
            .setHealthCheckUrl("http://127.0.0.1:" + port + "/actuator/health")
            .setMetadata(Map.of(
                "app", "java-provider",
                "language", "java"
            ));

        artemisManager.registryClient().register(instance);
    }
}
```

### 4.4 Rust Web Provider

```rust
// artemis-test/apps/rust-provider/src/main.rs
use axum::{
    routing::get,
    Router,
};
use artemis_client::{ClientConfig, RegistryClient, DiscoveryClient};
use std::net::SocketAddr;
use std::time::Duration;
use chrono::Local;

#[tokio::main]
async fn main() {
    let port: u16 = std::env::var("PORT").unwrap_or("8089".to_string()).parse().unwrap();

    // 注册到 Artemis
    tokio::spawn(register_to_artemis(port));

    // 启动 HTTP 服务
    let app = Router::new()
        .route("/sayHello", get(say_hello));

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    println!("Rust provider listening on {}", addr);

    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app).await.unwrap();
}

async fn say_hello() -> String {
    let port = std::env::var("PORT").unwrap_or("8089".to_string());
    let timestamp = Local::now().to_rfc3339();
    format!("Hello from Rust [{}] at {}", port, timestamp)
}

async fn register_to_artemis(port: u16) {
    tokio::time::sleep(Duration::from_secs(2)).await;

    let config = ClientConfig {
        server_urls: vec![
            "http://127.0.0.1:8081".to_string(),
            "http://127.0.0.1:8084".to_string(),
        ],
        heartbeat_interval_secs: 10,
        heartbeat_ttl_secs: 30,
        ..Default::default()
    };

    let registry = RegistryClient::new(config).await.unwrap();

    let instance = artemis_common::model::Instance {
        service_id: "hybrid-test-hello-service".to_string(),
        instance_id: format!("rust-web-{}", port),
        host: "127.0.0.1".to_string(),
        port: port as u32,
        health_check_url: Some(format!("http://127.0.0.1:{}/health", port)),
        metadata: {
            let mut m = std::collections::HashMap::new();
            m.insert("app".to_string(), "rust-provider".to_string());
            m.insert("language".to_string(), "rust".to_string());
            m
        },
        ..Default::default()
    };

    registry.register(instance).await.unwrap();
    println!("Registered Rust provider at port {} to Artemis", port);
}
```

### 4.5 Java Consumer (Job)

```java
// artemis-test/apps/java-consumer/src/.../HelloJobApplication.java
@SpringBootApplication
public class HelloJobApplication {

    @Autowired
    private ArtemisClientManager artemisManager;

    private final RestTemplate restTemplate = new RestTemplate();
    private final AtomicInteger roundRobin = new AtomicInteger(0);

    public static void main(String[] args) {
        SpringApplication.run(HelloJobApplication.class, args);
    }

    @Scheduled(fixedRate = 200) // 200ms
    public void callHelloService() {
        try {
            // 通过 Artemis 发现服务
            List<Instance> instances = artemisManager.discoveryClient()
                .getService("hybrid-test-hello-service");

            if (instances.isEmpty()) {
                System.out.println("No instances available");
                return;
            }

            // 简单的轮询负载均衡
            int index = roundRobin.getAndIncrement() % instances.size();
            Instance target = instances.get(index);

            // 调用服务
            String url = String.format("http://%s:%d/sayHello",
                target.getHost(), target.getPort());

            long startTime = System.currentTimeMillis();
            String response = restTemplate.getForObject(url, String.class);
            long latency = System.currentTimeMillis() - startTime;

            System.out.printf("[Java Job] Target=%s:%d Response=%s Latency=%dms%n",
                target.getHost(), target.getPort(), response, latency);

        } catch (Exception e) {
            System.err.println("[Java Job] Error: " + e.getMessage());
        }
    }
}
```

### 4.6 Rust Consumer (Job)

```rust
// artemis-test/apps/rust-consumer/src/main.rs
use artemis_client::{ClientConfig, DiscoveryClient};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[tokio::main]
async fn main() {
    println!("Starting Rust Job Consumer...");

    // 配置 DiscoveryClient
    let config = ClientConfig {
        server_urls: vec![
            "http://127.0.0.1:8081".to_string(),
            "http://127.0.0.1:8082".to_string(),
            "http://127.0.0.1:8083".to_string(),
            "http://127.0.0.1:8084".to_string(),
            "http://127.0.0.1:8085".to_string(),
            "http://127.0.0.1:8086".to_string(),
        ],
        ..Default::default()
    };

    let discovery = Arc::new(DiscoveryClient::new(config).await.unwrap());
    let round_robin = Arc::new(AtomicUsize::new(0));

    // 每 200ms 执行一次调用
    let mut interval = tokio::time::interval(Duration::from_millis(200));

    let success_count = Arc::new(AtomicUsize::new(0));
    let error_count = Arc::new(AtomicUsize::new(0));

    loop {
        interval.tick().await;

        let discovery = discovery.clone();
        let round_robin = round_robin.clone();
        let success_count = success_count.clone();
        let error_count = error_count.clone();

        tokio::spawn(async move {
            match run_job(discovery, round_robin).await {
                Ok(_) => {
                    success_count.fetch_add(1, Ordering::Relaxed);
                }
                Err(e) => {
                    error_count.fetch_add(1, Ordering::Relaxed);
                    eprintln!("[Rust Job] Error: {}", e);
                }
            }
        });
    }
}

async fn run_job(
    discovery: Arc<DiscoveryClient>,
    round_robin: Arc<AtomicUsize>,
) -> Result<(), Box<dyn std::error::Error>> {
    // 发现服务实例
    let instances = discovery.get_service("hybrid-test-hello-service").await?;

    if instances.is_empty() {
        return Err("No instances available".into());
    }

    // 轮询选择
    let index = round_robin.fetch_add(1, Ordering::Relaxed) % instances.len();
    let target = &instances[index];

    // 构建 URL
    let url = format!(
        "http://{}:{}/sayHello",
        target.host, target.port
    );

    // 发起调用
    let start = Instant::now();
    let response = reqwest::get(&url).await?;
    let latency = start.elapsed();

    let body = response.text().await?;

    println!(
        "[Rust Job] Target={}:{} Response={} Latency={:?}",
        target.host, target.port, body, latency
    );

    Ok(())
}
```

---

## 5. 脚本设计

### 5.1 setup.sh - 环境准备

```bash
#!/bin/bash
# artemis-test/scripts/setup.sh

set -e

echo "=== Artemis Hybrid Test Setup ==="

# 1. 编译 Rust 服务端
echo "[1/5] Building Rust Artemis Server..."
cd /home/koqizhao/Projects/mydotey-ai/ai-artemis
cargo build --release

# 2. 编译 Java 服务端
echo "[2/5] Building Java Artemis Server..."
cd /home/koqizhao/Projects/mydotey-ai/ai-artemis/artemis-java
mvn package -DskipTests -q

# 3. 创建目录结构
echo "[3/5] Creating directories..."
mkdir -p /home/koqizhao/Projects/mydotey-ai/ai-artemis/artemis-test/{data,logs,reports}

# 4. 初始化共享 SQLite 数据库
echo "[4/5] Initializing shared database..."
cd /home/koqizhao/Projects/mydotey-ai/ai-artemis/artemis-test/data
# 使用 Rust 节点初始化数据库（它会自动创建表）
cargo run --manifest-path ../../artemis-management/Cargo.toml --bin db-init -- sqlite://./artemis.db || true

echo "[5/5] Setup complete!"
echo ""
echo "Next steps:"
echo "  1. ./scripts/start-cluster.sh    # 启动集群"
echo "  2. ./scripts/start-apps.sh       # 启动测试应用"
echo "  3. ./scripts/start-console.sh    # 启动 Web Console"
```

### 5.2 start-cluster.sh - 启动 6 节点集群

```bash
#!/bin/bash
# artemis-test/scripts/start-cluster.sh

set -e

echo "=== Starting Artemis Hybrid Cluster (3 Java + 3 Rust) ==="

# 数据目录
DATA_DIR="$(cd "$(dirname "$0")/.." && pwd)/data"
LOGS_DIR="$(cd "$(dirname "$0")/.." && pwd)/logs"

mkdir -p "$DATA_DIR" "$LOGS_DIR"

# Java 服务端路径
JAVA_JAR="/home/koqizhao/Projects/mydotey-ai/ai-artemis/artemis-java/artemis-server/target/artemis-server.jar"

# Rust 服务端路径
RUST_BIN="/home/koqizhao/Projects/mydotey-ai/ai-artemis/target/release/artemis"

# 检查文件是否存在
if [ ! -f "$JAVA_JAR" ]; then
    echo "ERROR: Java JAR not found: $JAVA_JAR"
    echo "Please run ./scripts/setup.sh first"
    exit 1
fi

if [ ! -f "$RUST_BIN" ]; then
    echo "ERROR: Rust binary not found: $RUST_BIN"
    echo "Please run ./scripts/setup.sh first"
    exit 1
fi

# 启动 Java 节点 1
echo "[1/6] Starting Java Node 1 (port 8081)..."
java -jar "$JAVA_JAR" \
    --server.port=8081 \
    --artemis.peer.port=9091 \
    --spring.datasource.url="jdbc:sqlite:$DATA_DIR/artemis.db" \
    --artemis.cluster.peers=127.0.0.1:8082,127.0.0.1:8083,127.0.0.1:8084,127.0.0.1:8085,127.0.0.1:8086 \
    > "$LOGS_DIR/java-node1.log" 2>&1 &
echo $! > "$LOGS_DIR/java-node1.pid"

# 启动 Java 节点 2
echo "[2/6] Starting Java Node 2 (port 8082)..."
java -jar "$JAVA_JAR" \
    --server.port=8082 \
    --artemis.peer.port=9092 \
    --spring.datasource.url="jdbc:sqlite:$DATA_DIR/artemis.db" \
    --artemis.cluster.peers=127.0.0.1:8081,127.0.0.1:8083,127.0.0.1:8084,127.0.0.1:8085,127.0.0.1:8086 \
    > "$LOGS_DIR/java-node2.log" 2>&1 &
echo $! > "$LOGS_DIR/java-node2.pid"

# 启动 Java 节点 3
echo "[3/6] Starting Java Node 3 (port 8083)..."
java -jar "$JAVA_JAR" \
    --server.port=8083 \
    --artemis.peer.port=9093 \
    --spring.datasource.url="jdbc:sqlite:$DATA_DIR/artemis.db" \
    --artemis.cluster.peers=127.0.0.1:8081,127.0.0.1:8082,127.0.0.1:8084,127.0.0.1:8085,127.0.0.1:8086 \
    > "$LOGS_DIR/java-node3.log" 2>&1 &
echo $! > "$LOGS_DIR/java-node3.pid"

# 启动 Rust 节点 1
echo "[4/6] Starting Rust Node 1 (port 8084)..."
"$RUST_BIN" server \
    --addr "0.0.0.0:8084" \
    --config <(cat <<EOF
[server]
node_id = "node-r1"
listen_addr = "0.0.0.0:8084"
peer_port = 9094

[database]
db_type = "sqlite"
url = "sqlite://$DATA_DIR/artemis.db"

[cluster]
enabled = true
peers = [
    "127.0.0.1:8081",
    "127.0.0.1:8082",
    "127.0.0.1:8083",
    "127.0.0.1:8085",
    "127.0.0.1:8086"
]
EOF
) > "$LOGS_DIR/rust-node1.log" 2>&1 &
echo $! > "$LOGS_DIR/rust-node1.pid"

# 类似方式启动 Rust 节点 2 和 3...

echo ""
echo "All nodes started! Checking health..."
sleep 5

# 健康检查
for port in 8081 8082 8083 8084 8085 8086; do
    if curl -s "http://127.0.0.1:$port/health" > /dev/null 2>&1; then
        echo "✓ Node on port $port is healthy"
    else
        echo "✗ Node on port $port is not responding"
    fi
done

echo ""
echo "Next steps:"
echo "  ./scripts/start-apps.sh      # 启动测试应用"
echo "  ./scripts/start-console.sh   # 启动 Web Console"
```

### 5.3 运行测试

```bash
#!/bin/bash
# artemis-test/scripts/run-test.sh

set -e

echo "=== Running Hybrid Cluster Integration Test ==="

DURATION=${1:-600}  # 默认运行 10 分钟
REPORT_DIR="$(cd "$(dirname "$0")/.." && pwd)/reports"
LOGS_DIR="$(cd "$(dirname "$0")/.." && pwd)/logs"

mkdir -p "$REPORT_DIR"

# 启动测试
START_TIME=$(date +%s)
END_TIME=$((START_TIME + DURATION))

SUCCESS_COUNT=0
ERROR_COUNT=0
LATENCIES=()

echo "Test started at $(date)"
echo "Duration: $DURATION seconds"
echo ""

# 监控各个节点的状态
while [ $(date +%s) -lt $END_TIME ]; do
    for port in 8081 8082 8083 8084 8085 8086; do
        if curl -s "http://127.0.0.1:$port/health" > /dev/null 2>&1; then
            echo "Node $port: OK"
        else
            echo "Node $port: FAILED"
        fi
    done

    sleep 10
done

echo ""
echo "Test completed at $(date)"
echo ""

# 生成报告
echo "=== Test Report ===" > "$REPORT_DIR/test-report.txt"
echo "Test Duration: $DURATION seconds" >> "$REPORT_DIR/test-report.txt"
echo "Completed at: $(date)" >> "$REPORT_DIR/test-report.txt"
echo "" >> "$REPORT_DIR/test-report.txt"
echo "=== Cluster Health Check ===" >> "$REPORT_DIR/test-report.txt"
for port in 8081 8082 8083 8084 8085 8086; do
    if curl -s "http://127.0.0.1:$port/health" > /dev/null 2>&1; then
        echo "Node $port: HEALTHY" >> "$REPORT_DIR/test-report.txt"
    else
        echo "Node $port: UNHEALTHY" >> "$REPORT_DIR/test-report.txt"
    fi
done

cat "$REPORT_DIR/test-report.txt"
```

---

## 6. 执行计划

### 6.1 实施步骤

| 步骤 | 任务 | 估计时间 | 依赖 |
|------|------|----------|------|
| 1 | 复制 API 兼容性分析 | 2h | - |
| 2 | 创建目录结构和脚本框架 | 1h | - |
| 3 | 实现 Java Provider (Spring Boot) | 3h | - |
| 4 | 实现 Java Consumer | 2h | - |
| 5 | 实现 Rust Provider | 2h | - |
| 6 | 实现 Rust Consumer | 2h | - |
| 7 | 完善 start-cluster.sh | 3h | 1-6 |
| 8 | 完善 start-apps.sh | 2h | 1-6 |
| 9 | 测试和调优 | 4h | 1-8 |
| **总计** | | **26h** | |

### 6.2 测试执行流程

```bash
# 1. 进入测试目录
cd /home/koqizhao/Projects/mydotey-ai/ai-artemis/artemis-test

# 2. 环境准备（编译、打包）
./scripts/setup.sh

# 3. 启动 6 节点集群
./scripts/start-cluster.sh

# 4. 启动 Web Console（连接到任意节点）
./scripts/start-console.sh

# 5. 启动 8 个测试应用
./scripts/start-apps.sh

# 6. 运行 10 分钟集成测试
./scripts/run-test.sh 600

# 7. 查看结果
ls reports/

# 8. 清理资源
./scripts/cleanup.sh
```

---

## 7. 风险与应对

| 风险 | 概率 | 影响 | 应对措施 |
|------|------|------|----------|
| Java/Rust replication API 不兼容 | 高 | 高 | 前期验证，可能需要适配层或独立集群方案 |
| SQLite 并发问题（6 节点同时写） | 中 | 中 | 使用 WAL 模式，或改为单节点负责写 |
| Java artemis-java 版本过旧，依赖冲突 | 中 | 中 | 使用隔离的 Maven 项目，明确依赖版本 |
| 端口冲突 | 低 | 低 | 使用高位端口，启动前检测 |

---

## 8. 附录

### 8.1 端口汇总

| 服务 | 端口 |
|------|------|
| Java Node 1 | 8081 |
| Java Node 2 | 8082 |
| Java Node 3 | 8083 |
| Rust Node 1 | 8084 |
| Rust Node 2 | 8085 |
| Rust Node 3 | 8086 |
| Java Provider 1 | 8087 |
| Java Provider 2 | 8088 |
| Rust Provider 1 | 8089 |
| Rust Provider 2 | 8090 |
| Web Console | 5173 |

### 8.2 API 清单

| 端点 | 方法 | 用途 |
|------|------|------|
| /health | GET | 健康检查 |
| /sayHello | GET | 测试服务接口 |
| /api/discovery/service | GET | 服务发现 |
| /api/discovery/instances | GET | 获取实例列表 |
| /api/cluster/status | GET | 集群状态 |

### 8.3 相关文档

- Java 版 Artemis: `/home/koqizhao/Projects/mydotey-ai/ai-artemis/artemis-java/`
- Rust 版 Artemis: `/home/koqizhao/Projects/mydotey-ai/ai-artemis/`
- 本测试项目: `/home/koqizhao/Projects/mydotey-ai/ai-artemis/artemis-test/`
