# Artemis 数据库配置指南

Artemis 使用 **SeaORM 1.1** 作为数据库 ORM,支持运行时在 **SQLite** 和 **MySQL** 之间切换,用于持久化管理配置数据(分组、路由规则、Zone操作、金丝雀配置等)。

**核心优势**:
- 🔄 运行时数据库切换 - 无需重新编译
- 📦 统一 API - 同一代码支持多种数据库
- ⚡ 高性能 - 基于 async/await 的异步查询

## 支持的数据库

| 数据库 | 使用场景 | 优势 | 限制 |
|--------|---------|------|------|
| **SQLite** | 开发、测试、单节点部署 | 零配置、轻量级、文件存储 | 不支持多节点并发写入 |
| **MySQL** | 生产环境、集群部署 | 高并发、高可用、分布式支持 | 需要独立数据库服务 |

---

## 快速开始

### 开发环境快速配置

根据不同的开发需求，可以选择以下三种方式：

| 方式 | 命令 | 优点 | 缺点 | 适用场景 |
|------|------|------|------|---------|
| **无数据库** | `./scripts/cluster.sh start` | 快速启动，无需配置 | 重启后数据丢失 | 快速测试、开发 |
| **SQLite 共享** | `DB_TYPE=sqlite ./scripts/cluster.sh start` | 数据持久化，集群模式 | SQLite 并发写入有限 | 开发、测试 |
| **SQLite 单节点** | `artemis server --config config/examples/artemis-sqlite.toml` | 完整配置，单节点 | 需要配置文件 | 单节点开发 |

#### 方式1: 无数据库（推荐用于快速测试）

```bash
# 启动3节点集群（纯内存，无数据库）
./scripts/cluster.sh start

# 优点: 快速启动，无需配置
# 缺点: 重启后数据丢失
```

#### 方式2: SQLite 共享模式（推荐用于开发）

```bash
# 启动3节点集群（SQLite 共享模式）
DB_TYPE=sqlite ./scripts/cluster.sh start

# 数据持久化在 .cluster/data/shared.db
# 所有节点自动共享同一个数据库文件

# 首次启动需要创建 schema
sqlite3 .cluster/data/shared.db < artemis-management/migrations/001_initial_schema.sql

# 优点: 数据持久化，集群模式
# 缺点: SQLite 并发写入性能有限，适合开发测试
```

#### 方式3: SQLite 单节点

```bash
# 启动单节点服务
artemis server --config config/examples/artemis-sqlite.toml

# 数据持久化在 artemis.db 文件
```

---

## 配置方式

### 1. SQLite 配置 (开发/测试)

**配置文件**: `config/examples/artemis-sqlite.toml`

```toml
[database]
db_type = "sqlite"
# SeaORM SQLite URL 格式 (注意: 只有一个冒号)
url = "sqlite:artemis.db?mode=rwc"
max_connections = 10
```

**URL 格式说明**:
- ✅ 正确: `sqlite:artemis.db?mode=rwc` (相对路径)
- ✅ 正确: `sqlite:/absolute/path/to/artemis.db?mode=rwc` (绝对路径)
- ❌ 错误: `sqlite://artemis.db` (SQLx 格式,SeaORM 不支持)

**启动命令**:
```bash
./artemis server --config config/examples/artemis-sqlite.toml
```

**数据文件位置**:
- 数据库文件: `artemis.db`
- WAL 文件: `artemis.db-wal`, `artemis.db-shm`

**备份方式**:
```bash
# 停止服务后备份
cp artemis.db artemis.db.backup

# 或使用 SQLite 命令
sqlite3 artemis.db ".backup artemis.db.backup"
```

---

### 2. MySQL 配置 (生产环境)

**配置文件**: `config/examples/artemis-mysql.toml`

```toml
[database]
db_type = "mysql"
url = "mysql://artemis:artemis_password@localhost:3306/artemis"
max_connections = 20
```

**MySQL 数据库准备**:

```sql
-- 1. 创建数据库
CREATE DATABASE artemis CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;

-- 2. 创建用户
CREATE USER 'artemis'@'%' IDENTIFIED BY 'artemis_password';

-- 3. 授权
GRANT ALL PRIVILEGES ON artemis.* TO 'artemis'@'%';
FLUSH PRIVILEGES;
```

**启动命令**:
```bash
./artemis server --config config/examples/artemis-mysql.toml
```

**高可用配置** (可选):
```toml
[database]
db_type = "mysql"
# 使用主从复制或 Galera 集群的虚拟 IP
url = "mysql://artemis:password@vip.mysql.cluster:3306/artemis"
max_connections = 50
```

---

### 3. 环境变量配置

除了配置文件，也可以通过环境变量覆盖数据库配置：

```bash
# 设置环境变量
export ARTEMIS_DB_TYPE=mysql
export ARTEMIS_DB_URL="mysql://artemis:password@localhost:3306/artemis"
export ARTEMIS_DB_MAX_CONN=20

# 启动服务
artemis server
```

**优先级**: 环境变量 > 配置文件

**使用场景**:
- Docker 容器部署
- Kubernetes ConfigMap
- 临时覆盖配置

---

### 4. 配置文件位置

Artemis 提供了多个配置文件模板：

```
ai-artemis/
├── config/examples/
│   ├── artemis-sqlite.toml          # SQLite 配置模板
│   └── artemis-mysql.toml           # MySQL 配置模板
├── scripts/
│   └── cluster.sh                   # 自动生成集群配置
└── .cluster/                        # cluster.sh 生成的配置
    ├── node1.toml
    ├── node2.toml
    └── node3.toml
```

---

## 数据库Schema

Artemis 在首次启动时会自动运行数据库迁移,创建以下表:

| 表名 | 说明 | 记录数预估 |
|------|------|------------|
| `service_group` | 服务分组 | 10-100 |
| `service_group_tag` | 分组标签 | 100-1000 |
| `service_route_rule` | 路由规则 | 10-50 |
| `service_route_rule_group` | 路由规则分组关联 | 50-200 |
| `zone_operation` | Zone操作记录 | 10-50 |
| `canary_config` | 金丝雀配置 | 10-50 |
| `audit_log` | 审计日志 | 1000+ |
| `service_group_instance` | 分组实例关联 | 1000-10000 |
| `config_version` | 配置版本 | 100-1000 |
| `instance_operation` | 实例操作记录 | 100-1000 |
| `server_operation` | 服务器操作记录 | 10-100 |
| `instance_operation_log` | 实例操作历史 | 1000+ |

**总存储预估**: 10-100 MB (取决于配置数量和审计日志保留时间)

---

## 数据持久化特性

### 自动持久化

所有管理操作会自动持久化到数据库:

- ✅ 分组创建/更新/删除
- ✅ 路由规则创建/更新/删除/发布/停用
- ✅ Zone 拉入/拉出操作
- ✅ 金丝雀配置创建/更新/删除

### 启动恢复

服务器启动时会自动从数据库恢复配置:

```
[INFO] Initializing database: sqlite://artemis.db (type: SQLite)
[INFO] Running database migrations for SQLite
[INFO] Database migrations completed
[INFO] Loading persisted configurations from database...
[INFO] Loaded 5 service groups
[INFO] Loaded 3 route rules
[INFO] Loaded 2 zone operations
[INFO] Loaded 1 canary config
[INFO] Configurations loaded successfully
```

### 数据一致性

- **写入策略**: 内存优先 + 异步持久化
- **读取策略**: 从内存读取 (启动时从数据库加载)
- **故障恢复**: 服务重启后自动恢复所有配置

---

## 禁用数据库 (纯内存模式)

如果不需要持久化,可以完全禁用数据库:

```toml
# 移除或注释掉 [database] 配置块
# [database]
# ...
```

**注意**:
- ⚠️ 所有管理配置(分组、路由规则等)会在服务重启后丢失
- ✅ 实例注册数据不受影响 (客户端会自动重新注册)

---

## 性能优化建议

### SQLite

```toml
[database]
db_type = "sqlite"
url = "sqlite://artemis.db?mode=rwc&_journal_mode=WAL"
max_connections = 10
```

**WAL 模式优势**:
- 读写并发性更好
- 写入性能提升 50%+

### MySQL

```toml
[database]
db_type = "mysql"
url = "mysql://user:pass@host:3306/artemis?ssl-mode=REQUIRED"
max_connections = 50  # 根据负载调整
```

**连接池配置**:
- 单节点: 10-20 连接
- 集群节点 (3+): 20-50 连接
- 高并发: 50-100 连接

**MySQL 服务器优化**:
```sql
-- 增加连接数
SET GLOBAL max_connections = 200;

-- 优化 InnoDB
SET GLOBAL innodb_buffer_pool_size = 1G;
SET GLOBAL innodb_log_file_size = 256M;
```

---

## 迁移指南

### 从 SQLite 迁移到 MySQL

1. **导出 SQLite 数据**:
```bash
# 使用 SQLite 备份
sqlite3 artemis.db .dump > artemis_backup.sql
```

2. **转换 SQL (处理语法差异)**:
```bash
# 移除 SQLite 特定语法
sed 's/AUTOINCREMENT/AUTO_INCREMENT/g' artemis_backup.sql > artemis_mysql.sql
```

3. **导入到 MySQL**:
```bash
mysql -u artemis -p artemis < artemis_mysql.sql
```

4. **更新配置并重启**:
```toml
[database]
db_type = "mysql"
url = "mysql://artemis:password@localhost:3306/artemis"
```

### 从 MySQL 迁移到 SQLite

1. **导出 MySQL 数据**:
```bash
mysqldump -u artemis -p artemis > artemis_backup.sql
```

2. **转换 SQL**:
```bash
# 移除 MySQL 特定语法
sed 's/AUTO_INCREMENT/AUTOINCREMENT/g' artemis_backup.sql > artemis_sqlite.sql
sed 's/ENGINE=InnoDB//g' -i artemis_sqlite.sql
```

3. **导入到 SQLite**:
```bash
sqlite3 artemis.db < artemis_sqlite.sql
```

4. **更新配置并重启**:
```toml
[database]
db_type = "sqlite"
url = "sqlite://artemis.db"
```

---

## 故障排查

### 数据库连接失败

**SQLite**:
```
Error: unable to open database file
```

**解决方案**:
- 检查文件路径是否正确
- 确保目录存在且有写权限
- 检查磁盘空间

**MySQL**:
```
Error: Can't connect to MySQL server
```

**解决方案**:
- 检查 MySQL 服务是否运行: `systemctl status mysql`
- 验证连接参数 (host, port, user, password)
- 检查防火墙规则
- 测试连接: `mysql -h host -u user -p`

### 迁移失败

```
Error: migration failed
```

**解决方案**:
1. 查看详细错误日志
2. 手动运行迁移 SQL (位于 `artemis-management/migrations/`)
3. 检查数据库权限
4. 清空数据库重新初始化

### 数据不一致

**症状**: 重启后配置未恢复

**排查步骤**:
1. 检查数据库文件是否存在
2. 查看启动日志中的加载信息
3. 手动查询数据库验证数据:
```sql
-- SQLite
sqlite3 artemis.db "SELECT * FROM service_group;"

-- MySQL
mysql -u artemis -p -e "USE artemis; SELECT * FROM service_group;"
```

---

## 监控和维护

### 数据库大小监控

**SQLite**:
```bash
ls -lh artemis.db
```

**MySQL**:
```sql
SELECT
    table_name,
    ROUND((data_length + index_length) / 1024 / 1024, 2) AS size_mb
FROM information_schema.tables
WHERE table_schema = 'artemis'
ORDER BY (data_length + index_length) DESC;
```

### 定期备份

**SQLite**:
```bash
# 每天备份
0 2 * * * cp /path/to/artemis.db /backup/artemis_$(date +\%Y\%m\%d).db
```

**MySQL**:
```bash
# 每天备份
0 2 * * * mysqldump -u artemis -p artemis | gzip > /backup/artemis_$(date +\%Y\%m\%d).sql.gz
```

### 清理审计日志

```sql
-- 保留最近 30 天的审计日志
DELETE FROM audit_log WHERE created_at < DATE_SUB(NOW(), INTERVAL 30 DAY);

-- 保留最近 90 天的操作历史
DELETE FROM instance_operation_log WHERE timestamp < DATE_SUB(NOW(), INTERVAL 90 DAY);
```

---

## 最佳实践

### 开发环境
- ✅ 使用 SQLite
- ✅ 使用文件存储: `sqlite://artemis-dev.db`
- ✅ 定期备份

### 测试环境
- ✅ 使用 SQLite (单节点) 或 MySQL (集群)
- ✅ 使用独立数据库: `artemis_test`
- ✅ 自动化测试前清空数据

### 生产环境
- ✅ 使用 MySQL
- ✅ 配置主从复制或集群
- ✅ 定期备份 (每日)
- ✅ 监控数据库性能
- ✅ 配置连接池大小
- ✅ 启用 SSL/TLS (MySQL)

---

## 配置示例

### 完整的 SQLite 配置

```toml
# config/examples/artemis-sqlite.toml
[server]
node_id = "node-dev"
listen_addr = "0.0.0.0:8080"

[database]
db_type = "sqlite"
url = "sqlite://artemis.db?mode=rwc&_journal_mode=WAL"
max_connections = 10
```

### 完整的 MySQL 配置

```toml
# config/examples/artemis-mysql.toml
[server]
node_id = "node-prod-1"
listen_addr = "0.0.0.0:8080"

[cluster]
enabled = true
peers = ["http://node-2:8080", "http://node-3:8080"]

[database]
db_type = "mysql"
url = "mysql://artemis:secure_password@mysql.example.com:3306/artemis?ssl-mode=REQUIRED"
max_connections = 50
```

---

**更新日期**: 2026-02-15
**适用版本**: Artemis v0.1.0+

Generated with [Claude Code](https://claude.com/claude-code)
via [Happy](https://happy.engineering)
