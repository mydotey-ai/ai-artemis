# Artemis 脚本工具集

本目录包含 Artemis 项目的所有脚本工具,用于集群管理、测试和开发。

## 📁 目录结构

```
scripts/
├── README.md                           # 本文件 - 脚本说明
│
├── cluster.sh                          # 集群管理 (核心工具)
├── run-tests.sh                        # 测试运行工具
│
├── test-cluster-api.sh                 # 集群 API 测试
├── test-instance-management.sh         # 实例管理测试
├── test-group-routing.sh               # 分组路由测试
├── test-persistence.sh                 # 数据持久化测试
├── test-management.sh                  # 管理功能测试
│
├── test-group-instance-binding.sh      # 分组实例绑定测试
├── test-discovery-lookup.sh            # 服务发现查询测试
├── test-status-api.sh                  # 状态查询 API 测试
├── test-get-query-params.sh            # GET 查询参数测试
├── test-audit-logs.sh                  # 审计日志测试
├── test-all-operations.sh              # 批量操作查询测试
└── test-batch-replication.sh           # 批量复制测试
```

## 🚀 核心脚本

### cluster.sh - 集群管理工具

**用途**: 本地多节点集群的启动、停止和管理

**使用方法**:
```bash
# 从项目根目录运行

# 启动 3 节点集群
./scripts/cluster.sh start

# 启动 5 节点集群
./scripts/cluster.sh start 5

# 查看集群状态
./scripts/cluster.sh status

# 查看日志
./scripts/cluster.sh logs

# 停止集群
./scripts/cluster.sh stop

# 清理所有文件
./scripts/cluster.sh clean
```

**详细文档**: 参见 [CLUSTER.md](CLUSTER.md)

---

### run-tests.sh - 测试运行工具

**用途**: 便捷的测试命令封装,支持多种测试模式

**使用方法**:
```bash
# 从项目根目录运行

# 运行所有测试
./scripts/run-tests.sh all

# 仅运行单元测试
./scripts/run-tests.sh unit

# 仅运行 Web API 测试
./scripts/run-tests.sh web

# 生成代码覆盖率报告
./scripts/run-tests.sh coverage

# 显示帮助信息
./scripts/run-tests.sh help
```

**可用命令**:
- `all` - 运行所有测试 (默认)
- `unit` - 仅运行单元测试
- `web` - 仅运行 Web API 测试
- `registry` - 仅运行 Registry API 测试
- `discovery` - 仅运行 Discovery API 测试
- `integration` - 运行集成测试
- `coverage` - 生成代码覆盖率报告
- `bench` - 运行性能基准测试
- `watch` - 监视模式 (自动重新运行测试)
- `clean` - 清理测试缓存
- `summary` - 显示测试统计摘要

---

## 🧪 集成测试脚本

### Phase 1-8: 核心功能测试

#### test-cluster-api.sh - 集群 API 测试
**测试内容**: 7 步完整流程
1. 健康检查
2. 服务注册
3. 数据复制验证
4. 服务发现
5. 心跳续约
6. Prometheus 指标
7. 服务注销

**使用方法**:
```bash
./scripts/test-cluster-api.sh [基础端口] [节点数]
# 示例: ./scripts/test-cluster-api.sh 8080 3
```

**前置条件**: 集群已启动 (`./scripts/cluster.sh start`)

---

### Phase 12: 实例管理测试

#### test-instance-management.sh - 实例管理功能测试
**测试内容**: 13 步实例拉入/拉出测试
- 实例注册
- 实例拉出 (下线)
- 状态查询
- 服务发现过滤验证
- 实例拉入 (恢复)
- 服务器级别批量操作

**使用方法**:
```bash
./scripts/test-instance-management.sh
```

---

### Phase 13: 分组路由测试

#### test-group-routing.sh - 分组路由功能测试
**测试内容**: 13 步加权路由测试
- 创建分组
- 注册实例到不同分组
- 创建路由规则
- 配置分组权重 (70% vs 30%)
- 验证加权轮询
- 标签管理测试

**使用方法**:
```bash
./scripts/test-group-routing.sh
```

---

### Phase 14: 数据持久化测试

#### test-persistence.sh - 数据持久化功能测试
**测试内容**: SQLite/MySQL 持久化测试
- 配置写入数据库
- 服务重启
- 配置自动恢复验证

**使用方法**:
```bash
./scripts/test-persistence.sh
```

---

#### test-management.sh - 管理功能测试
**测试内容**: 高级管理功能测试
- 审计日志
- Zone 管理
- 金丝雀发布

**使用方法**:
```bash
# 测试所有模块
./scripts/test-management.sh all

# 仅测试审计日志
./scripts/test-management.sh audit

# 仅测试 Zone 管理
./scripts/test-management.sh zone

# 仅测试金丝雀发布
./scripts/test-management.sh canary
```

---

### Phase 19-25: 完整功能对齐测试

#### test-group-instance-binding.sh - 分组实例绑定测试
**测试内容**: 9 步绑定测试
- 手动绑定实例到分组
- 批量添加
- 查询绑定实例

**使用方法**:
```bash
./scripts/test-group-instance-binding.sh
```

---

#### test-discovery-lookup.sh - 服务发现查询测试
**测试内容**: 服务查询功能测试
- POST/GET 双模式查询
- 多服务批量查询

**使用方法**:
```bash
./scripts/test-discovery-lookup.sh
```

---

#### test-status-api.sh - 状态查询 API 测试
**测试内容**: 12 步状态查询测试
- 集群状态
- 配置状态
- 部署状态
- 租约状态

**使用方法**:
```bash
./scripts/test-status-api.sh
```

---

#### test-get-query-params.sh - GET 查询参数测试
**测试内容**: 7 步 GET 请求测试
- GET /api/discovery/service.json?serviceId=X
- GET /api/discovery/services.json?regionId=X
- camelCase 参数命名兼容性

**使用方法**:
```bash
./scripts/test-get-query-params.sh
```

---

#### test-audit-logs.sh - 审计日志测试
**测试内容**: 11 步审计日志测试
- 分组日志查询
- 路由规则日志查询
- Zone 操作日志查询
- 多维度过滤测试

**使用方法**:
```bash
./scripts/test-audit-logs.sh
```

---

#### test-all-operations.sh - 批量操作查询测试
**测试内容**: 11 步批量操作测试
- 查询所有实例操作
- 查询所有服务器操作
- POST/GET 双模式支持

**使用方法**:
```bash
./scripts/test-all-operations.sh
```

---

#### test-batch-replication.sh - 批量复制测试
**测试内容**: 8 步批量复制测试
- 批量注册/心跳/注销
- 增量同步 (services-delta)
- 全量同步 (sync-full)
- 防复制循环验证

**使用方法**:
```bash
./scripts/test-batch-replication.sh
```

---

## 📊 测试覆盖统计

| 脚本 | 测试步骤 | 覆盖功能 | Phase |
|-----|---------|---------|-------|
| test-cluster-api.sh | 7 步 | 集群 API 核心流程 | 1-11 |
| test-instance-management.sh | 13 步 | 实例拉入/拉出 | 12 |
| test-group-routing.sh | 13 步 | 分组路由、加权轮询 | 13 |
| test-persistence.sh | - | 数据持久化 | 14 |
| test-management.sh | - | 审计日志、Zone、金丝雀 | 15-17 |
| test-group-instance-binding.sh | 9 步 | 分组实例绑定 | 19 |
| test-discovery-lookup.sh | - | 服务发现查询 | 20 |
| test-status-api.sh | 12 步 | 状态查询 API | 21 |
| test-get-query-params.sh | 7 步 | GET 查询参数 | 22 |
| test-audit-logs.sh | 11 步 | 审计日志细分 | 24 |
| test-all-operations.sh | 11 步 | 批量操作查询 | 25 |
| test-batch-replication.sh | 8 步 | 批量复制 | 23 |

**总计**: 12 个集成测试脚本,覆盖 25 个 Phase 的所有核心功能

---

## 💡 使用建议

### 开发测试
```bash
# 1. 启动集群
./scripts/cluster.sh start

# 2. 运行单元测试
./scripts/run-tests.sh unit

# 3. 运行集成测试
./scripts/test-cluster-api.sh

# 4. 停止集群
./scripts/cluster.sh stop
```

### 完整测试
```bash
# 1. 启动集群
./scripts/cluster.sh start

# 2. 运行所有集成测试
./scripts/test-cluster-api.sh
./scripts/test-instance-management.sh
./scripts/test-group-routing.sh
./scripts/test-persistence.sh
./scripts/test-management.sh all
./scripts/test-group-instance-binding.sh
./scripts/test-status-api.sh
./scripts/test-get-query-params.sh
./scripts/test-audit-logs.sh
./scripts/test-all-operations.sh
./scripts/test-batch-replication.sh

# 3. 停止集群
./scripts/cluster.sh stop
```

### 覆盖率报告
```bash
# 生成 HTML 覆盖率报告
./scripts/run-tests.sh coverage
```

---

## 🔧 依赖工具

运行这些脚本需要以下工具:

- **必需**:
  - `curl` - HTTP 客户端
  - `jq` - JSON 处理工具
  - `cargo` - Rust 构建工具

- **可选**:
  - `cargo-llvm-cov` - 覆盖率工具 (`run-tests.sh coverage`)
  - `cargo-watch` - 监视模式 (`run-tests.sh watch`)
  - `sqlite3` - SQLite 数据库工具 (数据库测试)

**安装依赖** (Ubuntu/Debian):
```bash
sudo apt-get install curl jq sqlite3
cargo install cargo-llvm-cov cargo-watch
```

**安装依赖** (macOS):
```bash
brew install curl jq sqlite3
cargo install cargo-llvm-cov cargo-watch
```

---

## 📚 相关文档

- [CLUSTER.md](CLUSTER.md) - 集群管理详细指南
- [README.md](../README.md) - 项目首页和 API 使用
- [docs/TEST_STRATEGY.md](../docs/TEST_STRATEGY.md) - 测试策略和计划
- [docs/reports/TEST_STATUS.md](../docs/reports/TEST_STATUS.md) - 测试状态报告

---

## 🐛 故障排查

### 端口占用错误
```bash
# 检查端口占用
lsof -i :8080-8082

# 停止所有集群进程
./scripts/cluster.sh stop

# 清理残留进程
pkill -f artemis
```

### 测试失败
```bash
# 清理缓存重新构建
cargo clean
cargo build --release

# 重启集群
./scripts/cluster.sh restart

# 查看节点日志
./scripts/cluster.sh logs
```

### 数据库连接错误
```bash
# SQLite 模式 - 检查数据库文件
ls -la scripts/.cluster/data/shared.db

# 初始化 Schema
sqlite3 scripts/.cluster/data/shared.db < artemis-management/migrations/001_initial_schema.sql

# MySQL 模式 - 检查连接
mysql -u user -p -h host artemis
```

---

**维护**: Claude Sonnet 4.5 + koqizhao
**最后更新**: 2026-02-16
