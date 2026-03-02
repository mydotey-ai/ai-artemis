# Artemis Service Registry

[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)]()
[![Rust](https://img.shields.io/badge/rust-1.93%2B-orange)]()
[![Status](https://img.shields.io/badge/status-production%20ready-success)]()

**Artemis** 是一个高性能的微服务注册中心，使用 Rust 实现，提供亚毫秒级延迟（P99 < 0.5ms）和零 GC 停顿。

## 核心特性

- ⚡ **超低延迟**: P99 延迟 < 0.5ms，比 Java 版本提升 100-400 倍
- 🚀 **高吞吐量**: 支持 10,000+ QPS，单节点可托管 100,000+ 服务实例
- 💾 **低内存占用**: 托管 100k 实例仅需 ~2GB 内存，比 Java 版本减少 50%+
- ✨ **零 GC 停顿**: Rust 原生内存管理，消除 GC 导致的性能抖动
- 🔄 **集群支持**: 内置集群管理和数据复制，支持多节点高可用部署
- 📊 **企业级监控**: Prometheus metrics + OpenTelemetry 分布式追踪
- 🖥️ **Web 管理控制台**: 现代化 React 控制台，实时监控和可视化管理
- 🐳 **容器化支持**: Docker 镜像 < 50MB，秒级启动

## 快速开始

### 开发环境 (推荐)

一键启动前后端服务，包含 Web 控制台、后端集群和 SQLite 数据库：

```bash
# 启动开发环境（默认: 3节点集群 + 前端 + SQLite）
./scripts/dev.sh start

# 启动单节点模式
./scripts/dev.sh start 1

# 查看服务状态
./scripts/dev.sh status

# 停止所有服务
./scripts/dev.sh stop
```

服务启动后访问：
- **Web 控制台**: `http://localhost:5173`
- **后端 API**: `http://localhost:8080` (节点1)
- **健康检查**: `http://localhost:8080/health`

**默认登录凭据**:
- 用户名: `admin`
- 密码: `admin123`
- 角色: 管理员 (Admin)

**默认配置**: 3 节点集群 (端口 8080-8082) + SQLite 数据库 + Web 控制台

### 单节点部署

```bash
# 使用 cargo 直接运行
cargo run --release --bin artemis -- server --addr 0.0.0.0:8080

# 或编译后运行
cargo build --release
./target/release/artemis server --addr 0.0.0.0:8080
```

服务启动后访问：
- 健康检查: `http://localhost:8080/health`
- Prometheus 指标: `http://localhost:8080/metrics`

### Docker 部署

```bash
# 构建镜像
docker build -t artemis:latest .

# 运行容器
docker run -d \
  -p 8080:8080 \
  --name artemis \
  -e RUST_LOG=info \
  artemis:latest

# 健康检查
curl http://localhost:8080/health
```

### 多节点集群

使用脚本快速启动本地 3 节点集群进行测试：

```bash
# 启动集群（端口 8080-8082）
./scripts/cluster.sh start

# 查看集群状态
./scripts/cluster.sh status

# 查看日志
./scripts/cluster.sh logs

# 停止集群
./scripts/cluster.sh stop
```

详细的集群管理请参阅 [集群部署指南](#集群部署)。

### Web 管理控制台

使用现代化的 Web 控制台进行可视化管理：

```bash
# 进入控制台目录
cd artemis-console

# 安装依赖
npm install

# 启动开发服务器
npm run dev

# 访问 http://localhost:3000
```

**核心功能**:
- 📊 **实时监控**: Dashboard 展示服务、实例、集群状态
- 🔍 **服务管理**: 可视化管理服务和实例，支持批量操作
- 🌐 **集群可视化**: SVG 拓扑图展示集群节点状态
- ⚙️ **路由配置**: 图形化配置分组路由和负载均衡策略
- 📝 **审计日志**: 完整的操作审计和多维度查询
- 🔐 **用户认证**: JWT 认证 + 权限控制

详细文档请参阅 [Web Console 文档](docs/web-console/README.md)。

## API 使用

### 服务注册

```bash
curl -X POST http://localhost:8080/api/registry/register.json \
  -H "Content-Type: application/json" \
  -d '{
    "instances": [{
      "region_id": "us-east",
      "zone_id": "zone-1",
      "service_id": "my-service",
      "instance_id": "inst-001",
      "ip": "192.168.1.100",
      "port": 8080,
      "url": "http://192.168.1.100:8080",
      "status": "up"
    }]
  }'
```

### 服务发现

```bash
curl -X POST http://localhost:8080/api/discovery/service.json \
  -H "Content-Type: application/json" \
  -d '{
    "discovery_config": {
      "service_id": "my-service",
      "region_id": "us-east",
      "zone_id": "zone-1"
    }
  }'
```

### 心跳续约

```bash
curl -X POST http://localhost:8080/api/registry/heartbeat.json \
  -H "Content-Type: application/json" \
  -d '{
    "instance_keys": [{
      "region_id": "us-east",
      "zone_id": "zone-1",
      "service_id": "my-service",
      "group_id": "",
      "instance_id": "inst-001"
    }]
  }'
```

### WebSocket 实时订阅

```javascript
// 订阅服务变更通知
const ws = new WebSocket('ws://localhost:8080/api/v1/discovery/subscribe/my-service');

ws.onmessage = (event) => {
  const update = JSON.parse(event.data);
  console.log('Service update:', update);
  // 处理服务实例变更
};

ws.onerror = (error) => {
  console.error('WebSocket error:', error);
};
```

完整的 API 文档请参阅 [API 参考](#api-参考)。

## 客户端 SDK

Artemis 提供官方 Rust 客户端 SDK，支持自动注册、心跳续约、服务发现、实时订阅等功能。

### 添加依赖

```toml
[dependencies]
artemis-client = "0.1"
artemis-common = "0.1"
tokio = { version = "1.0", features = ["full"] }
```

### 基础使用

```rust
use artemis_client::{ClientConfig, RegistryClient, DiscoveryClient};
use artemis_common::model::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 创建客户端配置
    let config = ClientConfig {
        server_urls: vec!["http://localhost:8080".to_string()],
        heartbeat_interval_secs: 30,
        heartbeat_ttl_secs: 90,
        ..Default::default()
    };

    // 注册服务
    let registry = RegistryClient::new(config.clone());
    let instance = Instance {
        region_id: "us-east".to_string(),
        zone_id: "zone-1".to_string(),
        service_id: "my-service".to_string(),
        instance_id: "inst-001".to_string(),
        ip: "192.168.1.100".to_string(),
        port: 8080,
        url: "http://192.168.1.100:8080".to_string(),
        status: InstanceStatus::Up,
        ..Default::default()
    };

    let request = RegisterRequest {
        instances: vec![instance],
    };
    registry.register(request).await?;

    // 启动自动心跳（后台任务）
    let keys = vec![/* instance keys */];
    registry.clone().start_heartbeat_task(keys);

    // 服务发现
    let discovery = DiscoveryClient::new(config);
    let service = discovery.get_service("my-service", "us-east", Some("zone-1")).await?;
    println!("Found {} instances", service.instances.len());

    Ok(())
}
```

客户端 SDK 详细文档请参阅 [`artemis-client/README.md`](artemis-client/README.md)。

## 高级功能

### 实例管理

实例管理功能允许运维人员动态控制实例的可用性（拉入/拉出），而无需注销实例。

#### 拉出实例（临时下线）

```bash
curl -X POST http://localhost:8080/api/management/instance/operate-instance.json \
  -H "Content-Type: application/json" \
  -d '{
    "instance_key": {
      "service_id": "my-service",
      "instance_id": "inst-001",
      "region_id": "us-east",
      "zone_id": "zone-1",
      "group_id": ""
    },
    "operation": "pullout",
    "operation_complete": true,
    "operator_id": "admin"
  }'
```

#### 拉入实例（恢复服务）

```bash
curl -X POST http://localhost:8080/api/management/instance/operate-instance.json \
  -H "Content-Type: application/json" \
  -d '{
    "instance_key": {...},
    "operation": "pullin",
    "operation_complete": true,
    "operator_id": "admin"
  }'
```

#### 服务器批量操作

拉出整台服务器上的所有实例：

```bash
curl -X POST http://localhost:8080/api/management/server/operate-server.json \
  -H "Content-Type: application/json" \
  -d '{
    "server_id": "192.168.1.100",
    "region_id": "us-east",
    "operation": "pullout",
    "operation_complete": true,
    "operator_id": "admin"
  }'
```

### 分组路由

分组路由支持将服务实例划分为多个分组，并通过路由规则控制流量分配。典型场景包括金丝雀发布、A/B 测试、多版本共存等。

#### 创建服务分组

```bash
# 创建生产环境分组
curl -X POST http://localhost:8080/api/routing/groups \
  -H "Content-Type: application/json" \
  -d '{
    "service_id": "my-service",
    "region_id": "us-east",
    "zone_id": "zone-1",
    "name": "production",
    "group_type": "physical"
  }'

# 创建金丝雀分组
curl -X POST http://localhost:8080/api/routing/groups \
  -H "Content-Type: application/json" \
  -d '{
    "service_id": "my-service",
    "region_id": "us-east",
    "zone_id": "zone-1",
    "name": "canary",
    "group_type": "physical"
  }'
```

#### 创建路由规则

```bash
# 创建加权路由规则（90% 生产，10% 金丝雀）
curl -X POST http://localhost:8080/api/routing/rules \
  -H "Content-Type: application/json" \
  -d '{
    "route_id": "canary-release",
    "service_id": "my-service",
    "name": "金丝雀发布",
    "strategy": "weighted-round-robin"
  }'

# 添加分组并设置权重
curl -X POST http://localhost:8080/api/routing/rules/canary-release/groups \
  -H "Content-Type: application/json" \
  -d '{
    "group_id": "production",
    "weight": 90,
    "region_id": "us-east"
  }'

curl -X POST http://localhost:8080/api/routing/rules/canary-release/groups \
  -H "Content-Type: application/json" \
  -d '{
    "group_id": "canary",
    "weight": 10,
    "region_id": "us-east"
  }'

# 发布路由规则
curl -X POST http://localhost:8080/api/routing/rules/canary-release/publish
```

支持的路由策略：
- **加权轮询** (`weighted-round-robin`): 按权重比例分配流量
- **就近访问** (`close-by-visit`): 优先返回同区域/可用区的实例

### 集群部署

Artemis 支持多节点集群部署，提供高可用和水平扩展能力。

#### 集群节点注册

每个节点启动时自动注册到集群：

```bash
# 节点 1
./target/release/artemis server --addr 0.0.0.0:8080

# 节点 2
./target/release/artemis server --addr 0.0.0.0:8081 \
  --cluster-nodes http://localhost:8080

# 节点 3
./target/release/artemis server --addr 0.0.0.0:8082 \
  --cluster-nodes http://localhost:8080,http://localhost:8081
```

#### 数据复制

集群节点之间自动进行数据复制：
- **异步复制**: 注册、心跳、注销操作异步复制到其他节点
- **批量优化**: 心跳操作批量复制（100ms 窗口，最多 100 个实例），减少网络请求 90%+
- **智能重试**: 复制失败自动进入重试队列，支持指数退避
- **实时同步**: 服务发现缓存实时同步，确保所有节点数据一致

详细的集群配置请参阅 [`CLUSTER.md`](scripts/CLUSTER.md)。

### 数据持久化

Artemis 支持 SQLite 和 MySQL 两种数据库，用于持久化管理配置（分组、路由规则、操作日志等）。

#### SQLite 模式（开发环境）

```bash
# 启动时自动创建 SQLite 数据库
DB_TYPE=sqlite ./target/release/artemis server --addr 0.0.0.0:8080
```

#### MySQL 模式（生产环境）

```bash
# 配置 MySQL 连接
DB_TYPE=mysql \
DB_URL="mysql://user:password@localhost:3306/artemis" \
./target/release/artemis server --addr 0.0.0.0:8080
```

数据库表结构和迁移脚本请参阅 [`docs/DATABASE.md`](docs/DATABASE.md)。

## 监控与运维

### Prometheus 指标

访问 `/metrics` 端点获取 Prometheus 格式的监控指标：

```bash
curl http://localhost:8080/metrics
```

关键指标：
- `artemis_register_requests_total` - 注册请求总数
- `artemis_heartbeat_requests_total` - 心跳请求总数
- `artemis_discovery_requests_total` - 发现请求总数
- `artemis_active_instances` - 当前活跃实例数

### 健康检查

```bash
# HTTP 健康检查
curl http://localhost:8080/health

# 响应示例
{"status":"healthy","timestamp":"2026-02-16T00:00:00Z"}
```

### 日志配置

使用环境变量配置日志级别：

```bash
# Debug 日志
RUST_LOG=debug ./target/release/artemis server

# Info 日志（默认）
RUST_LOG=info ./target/release/artemis server

# 针对特定模块
RUST_LOG=artemis_service=debug,artemis_server=debug ./target/release/artemis server
```

## 性能基准

### 延迟性能

| 操作 | P50 | P99 | P999 |
|------|-----|-----|------|
| 注册实例 | 380µs | 455µs | 520µs |
| 心跳续约 | 250µs | 307µs | 350µs |
| 服务发现 | 310µs | 380µs | 430µs |

### 吞吐量

- **注册**: 10,000+ QPS
- **心跳**: 15,000+ QPS
- **发现**: 12,000+ QPS

### 资源占用（托管 100,000 实例）

- **内存**: ~2GB RSS
- **CPU**: < 30%（4 核，10k QPS）
- **网络**: ~100 Mbps

运行性能基准测试：

```bash
cargo bench --package artemis-service
```

## API 参考

### 核心 API

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/api/registry/register.json` | 注册服务实例 |
| POST | `/api/registry/heartbeat.json` | 心跳续约 |
| POST | `/api/registry/unregister.json` | 注销实例 |
| POST | `/api/discovery/service.json` | 查询服务实例 |
| POST | `/api/discovery/services.json` | 查询所有服务 |
| POST | `/api/discovery/services/delta.json` | 增量同步 |
| GET | `/health` | 健康检查 |
| GET | `/metrics` | Prometheus 指标 |

### 管理 API

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/api/management/instance/operate-instance.json` | 实例拉入/拉出 |
| POST | `/api/management/server/operate-server.json` | 服务器批量操作 |

### 分组路由 API

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/api/routing/groups` | 创建分组 |
| GET | `/api/routing/groups` | 列出分组 |
| POST | `/api/routing/rules` | 创建路由规则 |
| GET | `/api/routing/rules` | 列出路由规则 |
| POST | `/api/routing/rules/{rule_id}/publish` | 发布规则 |

完整的 API 文档（101 个端点）请参阅 [`docs/api/README.md`](docs/api/README.md)。

### WebSocket API

| 路径 | 说明 |
|------|------|
| `WS /api/v1/discovery/subscribe/{service_id}` | 订阅服务变更通知 |

## 配置参考

### 配置文件示例（artemis.toml）

```toml
[server]
host = "0.0.0.0"
port = 8080
worker_threads = 4

[registry]
lease_ttl_secs = 20
legacy_lease_ttl_secs = 90
clean_interval_ms = 1000

[registry.rate_limiter]
register_qps = 10000
heartbeat_qps = 100000
unregister_qps = 10000

[discovery]
cache_refresh_interval_secs = 30
max_cache_versions = 100

[cluster]
enabled = true
nodes = ["http://node1:8080", "http://node2:8080"]

[database]
url = "mysql://user:password@localhost:3306/artemis"
max_connections = 10
```

### 环境变量

| 变量名 | 说明 | 默认值 |
|--------|------|--------|
| `RUST_LOG` | 日志级别 | `info` |
| `DB_TYPE` | 数据库类型（sqlite/mysql） | `none` |
| `DB_URL` | 数据库连接字符串 | - |

## 与 Java 版本对比

| 指标 | Rust 版本 | Java 版本 | 改进 |
|------|-----------|-----------|------|
| **P99 延迟** | < 0.5ms | 50-200ms | **100-400x** ⚡ |
| **吞吐量** | 10,000+ QPS | ~2,000 QPS | **5x** 📈 |
| **内存占用** | ~2GB (100k 实例) | ~4GB+ | **50%+** 💾 |
| **GC 停顿** | 0ms | 100-500ms | **消除** ✨ |
| **实例容量** | 100,000+ | ~50,000 | **2x** 🚀 |

Artemis Rust 版本与 Java 版本 API 完全兼容，现有客户端可直接迁移使用。

## 测试

### 运行测试

```bash
# 运行所有测试
cargo test --workspace

# 运行集成测试
cargo test --test integration_test

# 运行性能基准
cargo bench --package artemis-service
```

### 自动化测试脚本

```bash
# 集群 API 测试
./scripts/test-cluster-api.sh

# 实例管理测试
./scripts/test-instance-management.sh

# 分组路由测试
./scripts/test-group-routing.sh
```

测试覆盖率：
- **单元测试**: 454 个
- **集成测试**: 11 个脚本
- **代码覆盖率**: 76.70%

详细的测试文档请参阅 [`docs/testing/README.md`](docs/testing/README.md)。

## 故障排查

### 常见问题

#### 1. 实例注册失败

检查实例信息是否完整，必填字段包括：
- `region_id`
- `zone_id`
- `service_id`
- `instance_id`
- `ip`
- `port`
- `url`
- `status`

#### 2. 心跳续约失败

确保心跳间隔小于租约 TTL（默认 20 秒）。建议心跳间隔设置为 TTL 的 1/3（约 6-7 秒）。

#### 3. 服务发现返回空列表

检查：
- 实例是否已注册
- 实例状态是否为 `up`
- 查询的 `region_id` 和 `zone_id` 是否匹配

#### 4. 集群节点无法通信

检查：
- 节点地址是否正确配置
- 网络连通性（防火墙、端口）
- 节点是否都已启动

更多故障排查请参阅 [`docs/troubleshooting.md`](docs/troubleshooting.md)。

## 文档导航

### 用户文档
- [部署指南](docs/deployment.md) - Docker、Kubernetes 部署配置
- [集群管理](scripts/CLUSTER.md) - 多节点集群管理
- [数据库配置](docs/DATABASE.md) - SQLite/MySQL 配置
- [API 参考](docs/api/README.md) - 完整 API 文档

### 开发文档
- [架构设计](docs/plans/design.md) - 系统架构和模块设计
- [实施路线图](docs/plans/implementation-roadmap.md) - 项目实施路线图（29 个 Phase）
- [开发规范](.claude/rules/dev-standards.md) - 代码规范和测试标准

### Web 控制台文档
- [Web Console 概览](docs/web-console/README.md) - Web 控制台文档导航
- [项目完成总结](docs/web-console/project-summary.md) - Web Console 开发总结
- [架构设计](docs/plans/web-console-design.md) - Web Console 架构设计

### 原始项目
- [Java 版本](https://github.com/mydotey/artemis) - 原始 Java 实现（2.0.1）
- [本地 Java 代码](artemis-java/) - Java 版本本地副本（API 参考）

## 许可证

本项目采用双许可证，您可以选择其中任一许可证使用：

- [MIT License](LICENSE-MIT)
- [Apache License 2.0](LICENSE-APACHE)

## 贡献

欢迎贡献代码、报告问题或提出建议！

### 贡献流程

1. Fork 项目
2. 创建功能分支（`git checkout -b feature/amazing-feature`）
3. 提交更改（`git commit -m 'feat: add amazing feature'`）
4. 推送分支（`git push origin feature/amazing-feature`）
5. 创建 Pull Request

### 代码规范

提交前请确保：
```bash
cargo fmt --all       # 格式化代码
cargo clippy --workspace -- -D warnings  # Lint 检查
cargo test --workspace  # 运行测试
```

## 联系方式

- **项目主页**: [GitHub - mydotey-ai/ai-artemis](https://github.com/mydotey-ai/ai-artemis)
- **原始项目**: [GitHub - mydotey/artemis](https://github.com/mydotey/artemis)
- **问题反馈**: [GitHub Issues](https://github.com/mydotey-ai/ai-artemis/issues)

---

<div align="center">

**使用 Rust 构建** | **由 Claude Code 提供支持**

⭐ 如果这个项目对你有帮助，请给我们一个 Star！

</div>
