# Artemis 开发环境指南

本文档介绍如何快速启动 Artemis 开发环境。

## 开发环境一键启动

使用 `dev.sh` 脚本可以一键启动前后端服务：

```bash
# 基本使用 (默认: 3节点集群 + 前端 + SQLite)
./scripts/dev.sh start              # 启动开发环境
./scripts/dev.sh status             # 查看服务状态
./scripts/dev.sh logs               # 查看所有日志
./scripts/dev.sh stop               # 停止所有服务
```

**默认配置**:
- 后端: 3 节点集群 (端口 8080-8082)
- 数据库: SQLite (共享数据库文件)
- 前端: 开发服务器 (端口 5173)

### 服务访问

启动后可以访问以下服务：

- **Web 控制台**: http://localhost:5173
- **后端 API**: http://localhost:8080
- **健康检查**: http://localhost:8080/health
- **Prometheus 指标**: http://localhost:8080/metrics

### 常用命令

#### 启动开发环境（默认配置）

```bash
./scripts/dev.sh start
```

这将启动：
- 3 个后端节点（端口 8080-8082）
- SQLite 数据库（共享模式）
- 1 个前端开发服务器（端口 5173）
- 自动打开浏览器

#### 启动单节点模式

```bash
./scripts/dev.sh start 1
```

#### 启动更大集群

```bash
# 启动 5 节点集群
./scripts/dev.sh start 5

# 启动 5 节点集群，自定义端口
./scripts/dev.sh start 5 9000
```

#### 查看服务状态

```bash
./scripts/dev.sh status
```

输出示例：
```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Artemis 开发环境状态
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

后端服务状态:
[INFO] Artemis 集群状态:

节点 1: 运行中 (PID: 12345, 端口: 8080)
节点 2: 运行中 (PID: 12346, 端口: 8081)
节点 3: 运行中 (PID: 12347, 端口: 8082)

前端服务状态:
前端服务: 运行中 (PID: 12348, 端口: 5173)
```

#### 查看日志

```bash
# 查看所有日志
./scripts/dev.sh logs

# 只查看后端日志
./scripts/dev.sh logs backend

# 只查看前端日志
./scripts/dev.sh logs frontend
```

#### 重启服务

```bash
# 使用默认配置重启（3节点 + SQLite）
./scripts/dev.sh restart

# 重启为单节点
./scripts/dev.sh restart 1

# 重启为5节点集群
./scripts/dev.sh restart 5
```

#### 清理环境

```bash
./scripts/dev.sh clean
```

这将：
1. 停止所有服务（前端 + 后端）
2. 清理日志文件
3. 清理临时文件和 PID 文件

## 使用数据库

开发环境默认使用 SQLite 数据库，也可以通过环境变量切换到其他数据库：

### SQLite 模式（默认）

```bash
# 默认就是 SQLite，无需额外配置
./scripts/dev.sh start

# 或显式指定
DB_TYPE=sqlite ./scripts/dev.sh start
```

**特点**:
- 所有节点共享同一个数据库文件
- 零配置，开箱即用
- 适合开发和测试

### MySQL 模式

```bash
DB_TYPE=mysql \
DB_URL="mysql://user:pass@localhost:3306/artemis" \
DB_MAX_CONN=20 \
./scripts/dev.sh start
```

**特点**:
- 更好的并发性能
- 适合生产环境
- 需要预先创建数据库

### 纯内存模式

```bash
DB_TYPE=none ./scripts/dev.sh start
```

**特点**:
- 不使用数据库，所有数据在内存中
- 最快的性能
- 重启后数据丢失

**注意**:
- SQLite 模式下所有节点共享同一个数据库文件
- 生产环境建议使用 MySQL 数据库

## 分别启动前后端

如果需要分别启动前后端服务（例如调试）：

### 仅启动后端

```bash
# 单节点
./scripts/cluster.sh start 1

# 多节点集群
./scripts/cluster.sh start 3
```

### 仅启动前端

```bash
cd artemis-console

# 安装依赖（首次运行）
npm install

# 启动开发服务器
npm run dev
```

## 目录结构

开发环境相关文件：

```
scripts/
├── dev.sh                          # 开发环境一键启动脚本
├── cluster.sh                      # 后端集群管理脚本
├── .dev/                           # 开发环境临时文件（不纳入版本控制）
│   ├── frontend.pid                # 前端进程 PID
│   └── frontend.log                # 前端日志
└── .cluster/                       # 集群临时文件（不纳入版本控制）
    ├── config/                     # 节点配置文件
    ├── logs/                       # 节点日志
    ├── pids/                       # 节点 PID 文件
    └── data/                       # SQLite 数据文件（如果启用）
```

## 故障排查

### 端口被占用

如果端口 8080 或 5173 被占用：

```bash
# 查找占用进程
lsof -i :8080
lsof -i :5173

# 停止开发环境
./scripts/dev.sh stop

# 或使用自定义端口启动后端
./scripts/dev.sh start 1 9000
```

### 前端依赖问题

```bash
cd artemis-console
rm -rf node_modules package-lock.json
npm install
```

### 服务无法启动

```bash
# 查看日志
./scripts/dev.sh logs

# 清理并重新启动
./scripts/dev.sh clean
./scripts/dev.sh start
```

### 数据库连接失败

如果使用数据库模式：

```bash
# 检查数据库连接
mysql -u user -p -h localhost

# 确保数据库已创建
CREATE DATABASE artemis;

# 重新启动
DB_TYPE=mysql DB_URL="mysql://user:pass@localhost:3306/artemis" ./scripts/dev.sh start
```

## 测试 API

启动开发环境后，可以使用以下命令测试 API：

```bash
# 健康检查
curl http://localhost:8080/health

# 注册服务实例
curl -X POST http://localhost:8080/api/v1/services/my-service/instances \
  -H "Content-Type: application/json" \
  -d '{
    "instance_id": "instance-1",
    "ip": "192.168.1.100",
    "port": 8081,
    "metadata": {}
  }'

# 查询服务实例
curl http://localhost:8080/api/v1/services/my-service/instances

# 发送心跳
curl -X PUT http://localhost:8080/api/v1/services/my-service/instances/instance-1/heartbeat
```

更多 API 示例请参阅 [README.md](../README.md)。

## 生产环境部署

开发环境仅用于本地开发和测试，生产环境部署请参阅：

- [部署指南](deployment.md)
- [Docker 部署](../README.md#docker-部署)
- [集群部署](../README.md#集群部署)

## 相关文档

- **项目首页**: [README.md](../README.md)
- **架构设计**: [plans/design.md](plans/design.md)
- **Web Console**: [web-console/README.md](web-console/README.md)
- **开发规范**: [../.claude/rules/dev-standards.md](../.claude/rules/dev-standards.md)
