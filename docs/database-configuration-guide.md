# Artemis 数据库配置指南

## 概述

Artemis 支持两种数据库后端:
- **SQLite** - 适用于开发/测试环境,单节点部署
- **MySQL** - 适用于生产环境,集群部署

## 数据库选择建议

| 场景 | 推荐数据库 | 原因 |
|------|------------|------|
| 本地开发 | 无数据库 (内存) | 快速启动,无需配置 |
| 单元测试 | SQLite | 轻量级,易于重置 |
| 集成测试 | SQLite | 独立环境,不依赖外部服务 |
| 单节点生产 | SQLite | 简单可靠,性能足够 |
| **集群生产** | **MySQL** | 共享存储,数据一致性 |

## 开发环境配置

### 方式1: 无数据库 (推荐)

使用 `cluster.sh` 启动集群,不启用持久化:

```bash
# 启动3节点集群 (纯内存,无数据库)
./cluster.sh start

# 优点: 快速启动,无需配置
# 缺点: 重启后数据丢失
```

### 方式2: SQLite 单节点

使用现有的 `artemis-sqlite.toml` 配置:

```bash
# 启动单节点服务
artemis server --config artemis-sqlite.toml

# 数据持久化在 artemis.db 文件
```

配置示例 (`artemis-sqlite.toml`):
```toml
[database]
db_type = "sqlite"
url = "sqlite://artemis.db"
max_connections = 10
```

## 生产环境配置

### MySQL 集群部署

#### 1. 准备 MySQL 数据库

```sql
-- 创建数据库
CREATE DATABASE artemis
  CHARACTER SET utf8mb4
  COLLATE utf8mb4_unicode_ci;

-- 创建用户并授权
CREATE USER 'artemis'@'%' IDENTIFIED BY 'your_secure_password';
GRANT ALL PRIVILEGES ON artemis.* TO 'artemis'@'%';
FLUSH PRIVILEGES;
```

#### 2. 配置节点

使用 `config/production-cluster-node*.toml` 模板:

```toml
[server]
node_id = "prod-node-1"  # 每个节点唯一
listen_addr = "0.0.0.0:8080"
region = "us-east"
zone = "zone-a"

[cluster]
enabled = true
peers = [
    "http://prod-node-2:8080",
    "http://prod-node-3:8080",
]

[database]
db_type = "mysql"
url = "mysql://artemis:your_password@mysql-host:3306/artemis"
max_connections = 20
```

**重要**: 所有节点必须共享同一个 MySQL 数据库!

#### 3. 启动集群

```bash
# 节点1
artemis server --config config/production-cluster-node1.toml

# 节点2
artemis server --config config/production-cluster-node2.toml

# 节点3
artemis server --config config/production-cluster-node3.toml
```

首次启动时,数据库 schema 会自动创建。

## 数据库 Schema

Artemis 在首次启动时会自动运行数据库迁移,创建以下表:

| 表名 | 用途 |
|------|------|
| `service_groups` | 服务分组 |
| `group_tags` | 分组标签 |
| `route_rule_groups` | 路由规则组 |
| `route_rules` | 路由规则 |
| `group_rule_associations` | 分组-规则关联 |
| `zone_operations` | Zone 操作记录 |
| `zone_operation_instances` | Zone 操作实例 |
| `canary_configs` | 金丝雀配置 |
| `canary_instances` | 金丝雀实例 |
| `audit_logs` | 审计日志 |
| `_sqlx_migrations` | 迁移历史 |

## 环境变量配置

除了配置文件,也可以通过环境变量覆盖数据库配置:

```bash
# 设置环境变量
export ARTEMIS_DB_TYPE=mysql
export ARTEMIS_DB_URL="mysql://user:pass@host:3306/artemis"
export ARTEMIS_DB_MAX_CONN=20

# 启动服务
artemis server
```

## 连接池配置

### 推荐配置

| 场景 | max_connections | 说明 |
|------|----------------|------|
| SQLite | 10 | SQLite 不支持高并发写入 |
| MySQL 单节点 | 20 | 根据负载调整 |
| MySQL 3节点集群 | 20/节点 | 总共60个连接 |
| MySQL 5节点集群 | 15/节点 | 总共75个连接 |

**注意**: MySQL 的 `max_connections` 参数应大于所有 Artemis 节点的连接池总和。

```sql
-- 查看 MySQL 最大连接数
SHOW VARIABLES LIKE 'max_connections';

-- 调整 MySQL 最大连接数
SET GLOBAL max_connections = 200;
```

## 性能优化

### MySQL 优化建议

1. **索引优化** - Schema 已包含必要的索引
2. **InnoDB 缓冲池**
   ```sql
   SET GLOBAL innodb_buffer_pool_size = 2G;
   ```
3. **连接超时**
   ```sql
   SET GLOBAL wait_timeout = 300;
   SET GLOBAL interactive_timeout = 300;
   ```

### SQLite 优化建议

1. **WAL 模式** - Artemis 自动启用 WAL (Write-Ahead Logging)
2. **同步模式** - 使用 NORMAL 模式平衡性能和可靠性
3. **缓存大小** - 默认配置已优化

## 数据备份

### SQLite 备份

```bash
# 停止服务
systemctl stop artemis

# 备份数据库文件
cp artemis.db artemis.db.backup

# 重启服务
systemctl start artemis
```

### MySQL 备份

```bash
# 使用 mysqldump
mysqldump -u artemis -p artemis > artemis_backup.sql

# 恢复
mysql -u artemis -p artemis < artemis_backup.sql
```

## 监控和维护

### 检查数据库连接

```bash
# MySQL
mysql -u artemis -p -h mysql-host -e "SHOW PROCESSLIST;"

# SQLite (通过应用日志)
tail -f /var/log/artemis.log | grep database
```

### 清理历史数据

```sql
-- 清理30天前的审计日志
DELETE FROM audit_logs
WHERE created_at < DATE_SUB(NOW(), INTERVAL 30 DAY);

-- 清理过期的 Zone 操作记录
DELETE FROM zone_operations
WHERE created_at < DATE_SUB(NOW(), INTERVAL 7 DAY);
```

## 故障排查

### 常见问题

#### 1. "No drivers installed" 错误

**原因**: sqlx 编译时未启用对应数据库驱动

**解决**: 确保 `Cargo.toml` 中包含:
```toml
sqlx = { version = "0.8", features = ["sqlite", "mysql"] }
```

#### 2. MySQL 连接超时

**原因**: 连接池耗尽或数据库不可达

**检查**:
```bash
# 测试连接
mysql -u artemis -p -h mysql-host -e "SELECT 1;"

# 检查连接数
SHOW STATUS LIKE 'Threads_connected';
```

#### 3. SQLite "database is locked"

**原因**: 并发写入冲突

**解决**: 启用 WAL 模式(Artemis 默认已启用),或切换到 MySQL

## 配置文件位置

```
ai-artemis/
├── artemis-sqlite.toml          # 开发环境 SQLite 配置
├── artemis-mysql.toml           # 单节点 MySQL 配置
└── config/
    ├── production-cluster-node1.toml  # 生产集群节点1
    ├── production-cluster-node2.toml  # 生产集群节点2
    └── production-cluster-node3.toml  # 生产集群节点3
```

## 相关文档

- [部署指南](deployment.md)
- [集群管理](../CLUSTER.md)
- [快速开始](../README.md)
