# Artemis 本地集群管理

本文档介绍如何在本地快速启动和管理多节点 Artemis 集群。

## 快速开始

### 启动集群

```bash
# 启动默认 3 节点集群
./cluster.sh start

# 启动 5 节点集群
./cluster.sh start 5

# 自定义端口启动
./cluster.sh start 3 8080 9090
```

### 查看状态

```bash
# 查看集群状态
./cluster.sh status
```

### 查看日志

```bash
# 查看所有节点日志
./cluster.sh logs

# 查看特定节点日志
./cluster.sh logs 1
```

### 停止集群

```bash
# 停止所有节点
./cluster.sh stop
```

### 重启集群

```bash
# 重启集群
./cluster.sh restart
```

### 清理文件

```bash
# 停止集群并清理所有文件
./cluster.sh clean
```

## 命令详解

### start - 启动集群

```bash
./cluster.sh start [节点数] [基础端口] [对等节点基础端口]
```

**参数:**
- `节点数`: 集群中的节点数量,默认 3
- `基础端口`: HTTP API 端口起始值,默认 8080
- `对等节点基础端口`: 集群节点间通信端口起始值,默认 9090

**示例:**
```bash
# 启动 3 节点集群,端口 8080-8082
./cluster.sh start

# 启动 5 节点集群,端口 8000-8004
./cluster.sh start 5 8000 9000
```

**节点分配:**
- 节点 1: HTTP 端口 8080, 对等节点端口 9090
- 节点 2: HTTP 端口 8081, 对等节点端口 9091
- 节点 3: HTTP 端口 8082, 对等节点端口 9092
- ...

### stop - 停止集群

```bash
./cluster.sh stop
```

优雅地停止所有运行中的节点。

### restart - 重启集群

```bash
./cluster.sh restart [节点数] [基础端口] [对等节点基础端口]
```

停止现有集群并重新启动,参数同 `start` 命令。

### status - 查看状态

```bash
./cluster.sh status [基础端口]
```

显示所有节点的运行状态,包括:
- 节点 ID
- 运行状态 (运行中/启动中/已停止)
- 进程 PID
- HTTP 端口

### logs - 查看日志

```bash
./cluster.sh logs [节点ID]
```

**参数:**
- `节点ID`: 可选,指定要查看日志的节点编号

**示例:**
```bash
# 查看所有节点日志
./cluster.sh logs

# 仅查看节点 1 的日志
./cluster.sh logs 1
```

### clean - 清理文件

```bash
./cluster.sh clean
```

停止集群并删除所有生成的文件,包括:
- 配置文件
- 日志文件
- PID 文件

## 目录结构

集群运行时会在项目根目录下创建 `.cluster` 目录:

```
.cluster/
├── config/          # 节点配置文件
│   ├── node1.toml
│   ├── node2.toml
│   └── node3.toml
├── logs/            # 节点日志文件
│   ├── node1.log
│   ├── node2.log
│   └── node3.log
└── pids/            # 节点进程 PID 文件
    ├── node1.pid
    ├── node2.pid
    └── node3.pid
```

## 配置说明

每个节点的配置文件包含以下部分:

### Server 配置

```toml
[server]
node_id = "node1"                  # 节点ID
listen_addr = "127.0.0.1:8080"     # HTTP API 监听地址
peer_port = 9090                   # 集群通信端口
region = "local"                   # 区域
zone = "zone1"                     # 可用区
```

### 集群配置

```toml
[cluster]
enabled = true                     # 启用集群模式
peers = [                          # 对等节点列表
    "127.0.0.1:9091",
    "127.0.0.1:9092"
]
```

### 复制配置

```toml
[replication]
enabled = true                     # 启用数据复制
timeout_secs = 5                   # 复制超时时间
batch_size = 100                   # 批量大小
```

### 租约配置

```toml
[lease]
ttl_secs = 30                      # 租约 TTL
cleanup_interval_secs = 60         # 清理间隔
```

### 缓存配置

```toml
[cache]
enabled = true                     # 启用缓存
expiry_secs = 300                  # 缓存过期时间
```

### 限流配置

```toml
[ratelimit]
enabled = true                     # 启用限流
requests_per_second = 10000        # 每秒请求数
burst_size = 5000                  # 突发流量大小
```

### 日志配置

```toml
[logging]
level = "info"                     # 日志级别: trace, debug, info, warn, error
format = "pretty"                  # 日志格式: json, pretty
```

## 数据库配置

cluster.sh 支持通过环境变量配置数据库持久化:

### 环境变量

```bash
DB_TYPE=sqlite      # 数据库类型: none (默认), sqlite, mysql
DB_URL=...          # 自定义数据库连接URL (可选)
DB_MAX_CONN=10      # 最大连接数 (默认10)
```

### 使用示例

#### SQLite 模式 (共享数据库)

```bash
# 启动集群,所有节点共享同一个 SQLite 数据库
DB_TYPE=sqlite ./cluster.sh start

# 首次启动需要创建 schema
sqlite3 .cluster/data/shared.db < artemis-management/migrations/001_initial_schema.sql

# 数据持久化在 .cluster/data/shared.db
# 优点: 数据持久化,配置简单
# 缺点: SQLite 并发写入性能有限,适合开发测试
```

#### MySQL 模式 (生产环境)

```bash
# 使用 MySQL 数据库 (适合生产环境)
DB_TYPE=mysql DB_URL="mysql://user:pass@host:3306/artemis" ./cluster.sh start

# 优点: 高并发性能,适合生产环境
# 需要: 提前创建数据库和用户
```

#### 无数据库模式 (默认)

```bash
# 纯内存模式,重启后数据丢失
./cluster.sh start

# 优点: 快速启动,无需配置
# 缺点: 重启后数据丢失
```

### 目录结构 (SQLite 模式)

```
.cluster/
├── config/          # 节点配置文件
├── logs/            # 节点日志文件
├── pids/            # 节点进程 PID 文件
└── data/            # SQLite 数据库文件 (仅 SQLite 模式)
    └── shared.db    # 所有节点共享的数据库
```

## 测试工具

### 自动化 API 测试

项目提供了 `test-cluster-api.sh` 脚本,用于自动化测试集群 API 功能:

```bash
# 使用默认配置测试 (基础端口 8080,3 个节点)
./test-cluster-api.sh

# 自定义基础端口和节点数
./test-cluster-api.sh 8080 3

# 测试 5 节点集群
./test-cluster-api.sh 8080 5
```

**测试内容包括:**
1. ✓ 健康检查 - 验证所有节点运行正常
2. ✓ 服务注册 - 在第一个节点注册实例
3. ✓ 数据复制 - 验证实例复制到所有节点
4. ✓ 服务发现 - 在所有节点查询服务
5. ✓ 心跳续约 - 测试租约续期
6. ✓ Prometheus 指标 - 检查监控指标
7. ✓ 服务注销 - 删除实例并验证同步

**依赖要求:**
- `curl` - HTTP 客户端
- `jq` - JSON 处理工具

**安装依赖:**
```bash
# Ubuntu/Debian
sudo apt-get install curl jq

# macOS
brew install curl jq
```

## 使用场景

### 开发测试

```bash
# 启动小型集群进行开发测试
./cluster.sh start 3

# 运行自动化测试验证功能
./test-cluster-api.sh

# 或手动测试服务注册
curl -X POST http://127.0.0.1:8080/api/registry/register.json \
  -H "Content-Type: application/json" \
  -d '{
    "instances": [{
      "region_id": "local",
      "zone_id": "zone1",
      "service_id": "test-service",
      "instance_id": "instance-1",
      "ip": "192.168.1.100",
      "port": 8080,
      "url": "http://192.168.1.100:8080",
      "status": "up"
    }]
  }'

# 在其他节点验证数据复制
curl -X POST http://127.0.0.1:8081/api/discovery/service.json \
  -H "Content-Type: application/json" \
  -d '{
    "discovery_config": {
      "service_id": "test-service",
      "region_id": "local",
      "zone_id": "zone1"
    }
  }'
```

### 性能测试

```bash
# 启动大规模集群进行性能测试
./cluster.sh start 10

# 使用压测工具测试
# ...

# 查看各节点日志
./cluster.sh logs
```

### 故障测试

```bash
# 启动集群
./cluster.sh start 3

# 停止某个节点模拟故障
kill $(cat .cluster/pids/node2.pid)

# 验证其他节点继续工作
curl http://127.0.0.1:8080/health
curl http://127.0.0.1:8082/health
```

## API 端点

每个节点提供以下 API 端点:

### 注册服务

```bash
POST http://127.0.0.1:8080/api/registry/register.json
Content-Type: application/json

{
  "instances": [{
    "region_id": "local",
    "zone_id": "zone1",
    "service_id": "my-service",
    "instance_id": "instance-1",
    "ip": "192.168.1.100",
    "port": 8080,
    "url": "http://192.168.1.100:8080",
    "status": "up"
  }]
}
```

### 心跳续约

```bash
POST http://127.0.0.1:8080/api/registry/heartbeat.json
Content-Type: application/json

{
  "instance_keys": [{
    "region_id": "local",
    "zone_id": "zone1",
    "service_id": "my-service",
    "group_id": "",
    "instance_id": "instance-1"
  }]
}
```

### 发现服务

```bash
POST http://127.0.0.1:8080/api/discovery/service.json
Content-Type: application/json

{
  "discovery_config": {
    "service_id": "my-service",
    "region_id": "local",
    "zone_id": "zone1"
  }
}
```

### 查询所有服务

```bash
GET http://127.0.0.1:8080/api/discovery/services.json?region_id=local&zone_id=zone1
```

### 注销服务

```bash
POST http://127.0.0.1:8080/api/registry/unregister.json
Content-Type: application/json

{
  "instance_keys": [{
    "region_id": "local",
    "zone_id": "zone1",
    "service_id": "my-service",
    "group_id": "",
    "instance_id": "instance-1"
  }]
}
```

### 健康检查

```bash
GET http://127.0.0.1:8080/health
```

### Prometheus 指标

```bash
GET http://127.0.0.1:8080/metrics
```

### WebSocket 实时推送

```bash
# 连接 WebSocket
WS ws://127.0.0.1:8080/ws

# 发送订阅消息
{
  "subscribe": {
    "service_id": "my-service"
  }
}

# 接收服务变更通知
{
  "service_update": {
    "service": {
      "region_id": "local",
      "zone_id": "zone1",
      "service_id": "my-service",
      "instances": [...]
    }
  }
}

# 取消订阅
{
  "unsubscribe": {
    "service_id": "my-service"
  }
}
```

## 故障排查

### 节点启动失败

1. 检查端口是否被占用:
   ```bash
   lsof -i :8080
   ```

2. 查看节点日志:
   ```bash
   cat .cluster/logs/node1.log
   ```

3. 检查编译是否成功:
   ```bash
   cargo build --release
   ```

### 集群通信问题

1. 检查对等节点端口是否开放:
   ```bash
   lsof -i :9090-9092
   ```

2. 查看配置文件中的 peers 列表是否正确:
   ```bash
   cat .cluster/config/node1.toml
   ```

### 进程残留

如果 `./cluster.sh stop` 后仍有残留进程:

```bash
# 查找并终止 artemis 进程
pkill -f artemis

# 或清理所有文件
./cluster.sh clean
```

## 注意事项

1. **端口占用**: 确保指定的端口范围未被其他程序占用
2. **资源限制**: 启动大量节点时注意系统资源限制
3. **数据持久化**: 当前版本使用内存存储,重启后数据会丢失
4. **网络隔离**: 所有节点运行在 127.0.0.1,仅供本地测试使用
5. **日志文件**: 长时间运行会产生大量日志,注意定期清理

## 进阶用法

### 自定义配置

1. 启动集群生成默认配置:
   ```bash
   ./cluster.sh start
   ```

2. 停止集群:
   ```bash
   ./cluster.sh stop
   ```

3. 编辑配置文件:
   ```bash
   vim .cluster/config/node1.toml
   ```

4. 重启集群应用新配置:
   ```bash
   ./cluster.sh start
   ```

### 多区域部署模拟

修改各节点配置文件中的 `region` 和 `zone` 字段,模拟多区域部署:

```toml
# node1.toml
[server]
region = "us-east"
zone = "zone1"

# node2.toml
[server]
region = "us-west"
zone = "zone1"

# node3.toml
[server]
region = "eu-central"
zone = "zone1"
```

## 相关文档

- [Artemis 设计文档](docs/plans/2026-02-13-artemis-rust-design.md)
- [实现计划](docs/plans/2026-02-13-artemis-rust-implementation.md)
- [产品规格说明](docs/artemis-rust-rewrite-specification.md)
