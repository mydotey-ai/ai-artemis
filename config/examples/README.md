# Artemis 配置文件示例

本目录包含 Artemis 服务器的配置文件示例,适用于不同的部署场景。

---

## 📁 配置文件列表

| 配置文件 | 数据库 | 场景 | 说明 |
|---------|--------|------|------|
| `artemis-sqlite.toml` | SQLite | 开发/测试 | 单节点,轻量级,快速启动 |
| `artemis-mysql.toml` | MySQL | 生产环境 | 集群模式,高性能,数据持久化 |
| `artemis-test-with-db.toml` | SQLite | 集成测试 | 测试数据库持久化功能 |

---

## 🚀 快速开始

### 1. artemis-sqlite.toml - 开发环境

**适用场景**: 本地开发、快速测试
**特点**:
- ✅ 无需外部数据库,使用 SQLite
- ✅ 集群功能关闭
- ✅ 日志格式友好 (pretty)
- ✅ 数据持久化到 `artemis.db`

**使用方法**:
```bash
# 复制并编辑配置
cp config/examples/artemis-sqlite.toml config/my-dev.toml

# 启动服务器
./target/release/artemis server --config config/my-dev.toml

# 或直接使用示例配置
./target/release/artemis server --config config/examples/artemis-sqlite.toml
```

**配置重点**:
```toml
[server]
node_id = "node-dev"
listen_addr = "0.0.0.0:8080"

[cluster]
enabled = false  # 单节点模式

[database]
db_type = "sqlite"
url = "sqlite://artemis.db"
max_connections = 10
```

---

### 2. artemis-mysql.toml - 生产环境

**适用场景**: 生产部署、多节点集群
**特点**:
- ✅ MySQL 数据库,高性能
- ✅ 集群功能启用,支持数据复制
- ✅ JSON 日志格式,便于日志采集
- ✅ 适合大规模部署

**使用方法**:
```bash
# 1. 准备 MySQL 数据库
mysql -u root -p <<EOF
CREATE DATABASE artemis CHARACTER SET utf8mb4;
CREATE USER 'artemis'@'%' IDENTIFIED BY 'your_secure_password';
GRANT ALL PRIVILEGES ON artemis.* TO 'artemis'@'%';
FLUSH PRIVILEGES;
EOF

# 2. 初始化 Schema
mysql -u artemis -p artemis < artemis-management/migrations/001_initial_schema.sql

# 3. 复制并编辑配置
cp config/examples/artemis-mysql.toml config/node1.toml

# 编辑配置,修改:
# - node_id
# - peers (集群节点列表)
# - database.url (数据库连接字符串)

# 4. 启动节点
./target/release/artemis server --config config/node1.toml
```

**配置重点**:
```toml
[server]
node_id = "node-prod-1"
listen_addr = "0.0.0.0:8080"

[cluster]
enabled = true
peers = ["http://node-2:8080", "http://node-3:8080"]

[replication]
enabled = true
batch_size = 100
batch_interval_ms = 100

[database]
db_type = "mysql"
url = "mysql://artemis:password@localhost:3306/artemis"
max_connections = 20
```

---

### 3. artemis-test-with-db.toml - 测试配置

**适用场景**: 集成测试、持久化功能测试
**特点**:
- ✅ SQLite 数据库,便于测试
- ✅ 单节点,无集群
- ✅ 启用复制逻辑 (用于测试)
- ✅ 测试数据持久化

**使用方法**:
```bash
# 运行测试
./target/release/artemis server --config config/examples/artemis-test-with-db.toml

# 测试数据持久化
curl -X POST http://localhost:8080/api/routing/groups \
  -H "Content-Type: application/json" \
  -d '{...}'

# 重启服务器,验证数据恢复
pkill artemis
./target/release/artemis server --config config/examples/artemis-test-with-db.toml

# 查询数据,验证持久化成功
curl http://localhost:8080/api/routing/groups
```

---

## 📊 配置对比

### 数据库选择

| 特性 | SQLite | MySQL |
|------|--------|-------|
| **部署复杂度** | 低 (无需外部服务) | 中 (需要 MySQL 服务器) |
| **性能** | 中 (单节点足够) | 高 (支持高并发) |
| **数据安全** | 中 (文件备份) | 高 (主从复制、备份策略) |
| **适用场景** | 开发、测试、小规模 | 生产、大规模、集群 |
| **并发写入** | 有限 | 优秀 |

### 集群模式对比

| 配置 | 集群 | 复制 | 适用场景 |
|------|------|------|----------|
| `artemis-sqlite.toml` | ❌ | ❌ | 单节点开发 |
| `artemis-mysql.toml` | ✅ | ✅ | 多节点生产 |
| `artemis-test-with-db.toml` | ❌ | ✅ | 测试持久化 |

---

## ⚙️ 配置文件结构

所有配置文件包含以下部分:

### [server] - 服务器配置
```toml
[server]
node_id = "unique-node-id"      # 节点唯一标识
listen_addr = "0.0.0.0:8080"    # HTTP 监听地址
peer_port = 9090                # Peer 端口 (预留)
region = "us-east"              # 区域
zone = "zone-1"                 # 可用区
```

### [cluster] - 集群配置
```toml
[cluster]
enabled = true                   # 是否启用集群
peers = [                        # 对等节点列表
    "http://node-2:8080",
    "http://node-3:8080"
]
```

### [replication] - 数据复制配置
```toml
[replication]
enabled = true                   # 是否启用数据复制
timeout_secs = 5                 # 复制请求超时
batch_size = 100                 # 批量大小
batch_interval_ms = 100          # 批处理窗口
max_retries = 3                  # 最大重试次数
```

### [lease] - 租约配置
```toml
[lease]
ttl_secs = 30                    # 租约 TTL
cleanup_interval_secs = 60       # 清理间隔
```

### [cache] - 缓存配置
```toml
[cache]
enabled = true                   # 是否启用缓存
expiry_secs = 300                # 缓存过期时间
```

### [ratelimit] - 限流配置
```toml
[ratelimit]
enabled = true                   # 是否启用限流
requests_per_second = 10000      # 每秒请求数
burst_size = 5000                # 突发流量大小
```

### [logging] - 日志配置
```toml
[logging]
level = "info"                   # 日志级别: trace, debug, info, warn, error
format = "pretty"                # 日志格式: json, pretty
```

### [database] - 数据库配置 (可选)
```toml
[database]
db_type = "sqlite"               # 数据库类型: sqlite, mysql
url = "sqlite://artemis.db"      # 连接字符串
max_connections = 10             # 最大连接数
```

---

## 📝 自定义配置

### 步骤1: 选择模板
根据你的场景选择合适的配置文件作为模板:
- 开发环境: `artemis-sqlite.toml`
- 生产环境: `artemis-mysql.toml`

### 步骤2: 复制配置
```bash
cp config/examples/artemis-sqlite.toml config/my-config.toml
```

### 步骤3: 编辑配置
根据你的需求修改:
1. **server.node_id** - 节点唯一标识
2. **server.listen_addr** - 监听地址和端口
3. **cluster.peers** - 集群节点列表 (如启用集群)
4. **database.url** - 数据库连接字符串
5. **logging.level** - 日志级别 (开发用 `debug`, 生产用 `info`)

### 步骤4: 启动服务
```bash
./target/release/artemis server --config config/my-config.toml
```

---

## 🔒 安全建议

### 1. 数据库密码
**不要在配置文件中硬编码密码!**

使用环境变量:
```toml
[database]
url = "${DATABASE_URL}"  # 从环境变量读取
```

启动时设置:
```bash
export DATABASE_URL="mysql://artemis:secure_password@localhost:3306/artemis"
./target/release/artemis server --config config/my-config.toml
```

### 2. 网络监听
生产环境建议:
- 使用反向代理 (Nginx, HAProxy)
- 启用 TLS/SSL (待实现)
- 限制监听地址 (如 `127.0.0.1:8080` 仅本地访问)

### 3. 文件权限
限制配置文件权限:
```bash
chmod 600 config/my-config.toml
```

---

## 📚 相关文档

- [DATABASE.md](../../docs/DATABASE.md) - 数据库配置完整指南
- [database-configuration-guide.md](../../docs/database-configuration-guide.md) - 数据库配置详细说明
- [CLUSTER.md](../../scripts/CLUSTER.md) - 集群管理指南
- [deployment.md](../../docs/deployment.md) - 部署指南

---

## 🆘 故障排查

### 数据库连接失败
```bash
# 检查 MySQL 是否运行
systemctl status mysql

# 测试连接
mysql -h localhost -u artemis -p artemis

# 检查防火墙
sudo ufw status
```

### 端口被占用
```bash
# 查看端口占用
lsof -i :8080

# 修改配置文件中的 listen_addr 端口
```

### 配置文件格式错误
```bash
# 验证 TOML 格式
cargo run --bin artemis -- server --config config/my-config.toml

# 查看错误信息
```

---

**维护**: Claude Sonnet 4.5 + koqizhao
**最后更新**: 2026-02-16
