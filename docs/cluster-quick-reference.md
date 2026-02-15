# cluster.sh 快速参考指南

## 基本用法

### 启动集群

```bash
# 默认: 3节点, 端口 8080-8082
./cluster.sh start

# 5节点集群
./cluster.sh start 5

# 自定义端口 (9000-9002)
./cluster.sh start 3 9000
```

### 查看状态

```bash
# 查看集群状态
./cluster.sh status

# 查看自定义端口集群状态
./cluster.sh status 9000
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
# 重启集群(保持配置)
./cluster.sh restart

# 重启并修改节点数
./cluster.sh restart 5
```

### 清理环境

```bash
# 停止集群并删除所有文件
./cluster.sh clean
```

## 集群配置

### 端口分配

每个节点使用两个端口:
- **HTTP API 端口**: `基础端口 + 节点编号 - 1`
  - 用于: 客户端请求、集群间通信
- **Peer 端口**: `9090 + 节点编号 - 1`
  - 预留字段,当前未使用

示例 (3节点,基础端口8080):
| 节点 | HTTP API | Peer Port |
|------|----------|-----------|
| 1    | 8080     | 9090      |
| 2    | 8081     | 9091      |
| 3    | 8082     | 9092      |

### peers 配置

每个节点的 `peers` 列表包含其他所有节点的 HTTP API 地址:

```toml
# 节点1 (8080) 的配置
[cluster]
enabled = true
peers = [
    "127.0.0.1:8081",  # 节点2
    "127.0.0.1:8082",  # 节点3
]
```

## 文件位置

```
.cluster/
├── config/        # 节点配置文件
│   ├── node1.toml
│   ├── node2.toml
│   └── node3.toml
├── logs/          # 节点日志
│   ├── node1.log
│   ├── node2.log
│   └── node3.log
└── pids/          # PID 文件
    ├── node1.pid
    ├── node2.pid
    └── node3.pid
```

## 常见操作

### 测试集群复制

```bash
# 1. 启动集群
./cluster.sh start

# 2. 在节点1注册服务
curl -X POST http://localhost:8080/api/registry/register.json \
  -H "Content-Type: application/json" \
  -d '{
    "instances": [{
      "region_id": "local",
      "zone_id": "zone1",
      "service_id": "test-svc",
      "instance_id": "inst-1",
      "ip": "192.168.1.100",
      "port": 8000,
      "url": "http://192.168.1.100:8000",
      "status": "up"
    }]
  }'

# 3. 从节点2查询(验证复制)
curl -X POST http://localhost:8081/api/discovery/service.json \
  -H "Content-Type: application/json" \
  -d '{
    "discovery_config": {
      "service_id": "test-svc",
      "region_id": "local",
      "zone_id": "zone1"
    }
  }'
```

### 健康检查

```bash
# 检查所有节点健康状态
for port in 8080 8081 8082; do
  echo -n "节点 $port: "
  curl -s http://localhost:$port/health
  echo
done
```

### 查看复制日志

```bash
# 查看节点1的复制事件
grep "Replicating" .cluster/logs/node1.log
```

## 故障排查

### 节点启动失败

```bash
# 查看节点日志
cat .cluster/logs/node1.log

# 检查端口占用
lsof -i :8080
```

### 集群通信问题

```bash
# 检查peers配置
cat .cluster/config/node1.toml | grep -A 5 "peers ="

# 查看集群日志
grep "peer" .cluster/logs/node*.log
```

### 清理残留进程

```bash
# 清理所有artemis进程
pkill -f "artemis server"

# 清理目录
rm -rf .cluster
```

## 性能指标

- **启动时间**: ~2秒/节点
- **健康检查延迟**: < 10ms
- **集群复制延迟**: < 100ms (batch_interval_ms = 100)
- **优雅关闭**: ~1秒

## 配置参数

### Replication 配置

```toml
[replication]
enabled = true
timeout_secs = 5          # 复制请求超时
batch_size = 100          # 批处理大小
batch_interval_ms = 100   # 批处理窗口
max_retries = 3           # 最大重试次数
```

### Lease 配置

```toml
[lease]
ttl_secs = 30                  # 租约TTL
cleanup_interval_secs = 60     # 清理间隔
```

### Cache 配置

```toml
[cache]
enabled = true
expiry_secs = 300  # 缓存过期时间
```

## 相关文档

- [CLUSTER.md](../CLUSTER.md) - 集群功能详细说明
- [docs/reports/features/cluster-replication.md](reports/features/cluster-replication.md) - 集群复制实现
- [README.md](../README.md) - 项目总览
