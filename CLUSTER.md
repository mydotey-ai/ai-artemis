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

## 使用场景

### 开发测试

```bash
# 启动小型集群进行开发测试
./cluster.sh start 3

# 模拟服务注册
curl -X POST http://127.0.0.1:8080/api/v1/registry/instances \
  -H "Content-Type: application/json" \
  -d '{
    "serviceId": "test-service",
    "instanceId": "instance-1",
    "ip": "192.168.1.100",
    "port": 8080
  }'

# 在其他节点验证数据复制
curl http://127.0.0.1:8081/api/v1/discovery/instances/test-service
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
POST http://127.0.0.1:8080/api/v1/registry/instances
```

### 心跳续约

```bash
PUT http://127.0.0.1:8080/api/v1/registry/instances/{serviceId}/{instanceId}
```

### 发现服务

```bash
GET http://127.0.0.1:8080/api/v1/discovery/instances/{serviceId}
```

### 健康检查

```bash
GET http://127.0.0.1:8080/health
```

### WebSocket 实时推送

```bash
WS ws://127.0.0.1:8080/api/v1/discovery/subscribe/{serviceId}
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
