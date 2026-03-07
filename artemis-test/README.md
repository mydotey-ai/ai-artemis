# Artemis Hybrid Integration Test

Artemis Java/Rust 混合集群集成测试框架

## 概述

本项目是一个完整的集成测试框架，用于验证 Artemis Java 版本和 Rust 版本之间的互操作性。

### 测试目标

- ✅ **集群互通性**: Java 节点和 Rust 节点组成对等集群，数据双向同步
- ✅ **客户端互通**: Java Client 和 Rust Client 互相注册、发现、调用服务
- ✅ **Management 共享**: Management 配置通过共享 SQLite 数据库同步
- ✅ **负载均衡**: 简单轮询负载均衡在多客户端、多服务端场景下工作正常

### 测试拓扑

```
┌─────────────────────────────────────────────────────────────────┐
│                      Artemis Hybrid Cluster                      │
├─────────────────────────────────────────────────────────────────┤
│  Java Nodes: 8081, 8082, 8083                                     │
│  Rust Nodes: 8084, 8085, 8086                                     │
│  Shared DB:  SQLite (artemis.db)                                  │
├─────────────────────────────────────────────────────────────────┤
│  Java Providers:  8087, 8088                                    │
│  Rust Providers:  8089, 8090                                    │
│  Service Name:  "hybrid-test-hello-service"                     │
├─────────────────────────────────────────────────────────────────┤
│  Java Consumers:  job-1, job-2  (200ms interval)                │
│  Rust Consumers:  job-1, job-2  (200ms interval)                │
│  Load Balancer:   Round Robin                                   │
└─────────────────────────────────────────────────────────────────┘
```

## 快速开始

### 前置要求

- **Java**: JDK 8+ 和 Maven 3.6+
- **Rust**: 1.70+ 和 Cargo
- **Node.js**: 18+ (用于 Web Console)
- **系统工具**: curl, lsof

### 1. 环境准备

```bash
cd artemis-test

# 一键准备环境（编译 Java/Rust 服务端、创建目录结构）
./scripts/setup.sh
```

### 2. 启动集群

```bash
# 启动 6 节点混合集群（3 Java + 3 Rust）
./scripts/start-cluster.sh
```

集群启动后会显示健康检查结果：
```
✓ Node port 8081: HEALTHY
✓ Node port 8082: HEALTHY
...
```

### 3. 启动 Web Console（可选）

```bash
# 启动 Web Console 管理集群
./scripts/start-console.sh

# 访问 http://localhost:5173
# 默认账号: admin / admin123
```

### 4. 启动测试应用

```bash
# 启动 4 个 Provider + 4 个 Consumer
./scripts/start-apps.sh
```

### 5. 运行测试

```bash
# 运行 10 分钟集成测试（收集指标）
./scripts/run-test.sh 600

# 或运行 1 分钟快速测试
./scripts/run-test.sh 60
```

### 6. 查看结果

```bash
# 查看测试报告
cat reports/test-report-*.txt

# 查看实时日志
tail -f logs/java-node1.log
tail -f logs/rust-provider1.log
```

### 7. 清理环境

```bash
# 停止所有进程并清理
./scripts/cleanup.sh
```

## 项目结构

```
artemis-test/
├── DESIGN.md                 # 详细设计文档
├── README.md                 # 本文件
├── config/                   # 配置文件
│   ├── rust-node1.toml
│   ├── rust-node2.toml
│   └── rust-node3.toml
├── apps/                     # 测试应用源码
│   ├── java-provider/       # Java Web Provider
│   ├── java-consumer/       # Java Job Consumer
│   ├── rust-provider/       # Rust Web Provider
│   └── rust-consumer/       # Rust Job Consumer
├── scripts/                  # 测试脚本
│   ├── setup.sh             # 环境准备
│   ├── start-cluster.sh     # 启动集群
│   ├── start-apps.sh        # 启动测试应用
│   ├── start-console.sh     # 启动 Web Console
│   ├── run-test.sh          # 运行测试
│   ├── cleanup.sh           # 清理资源
│   └── status.sh           # 状态检查
├── data/                     # SQLite 数据库 (gitignore)
├── logs/                     # 日志文件 (gitignore)
└── reports/                  # 测试报告 (gitignore)
```

## API 端点

### 集群节点

| 节点 | 地址 | 健康检查 |
|------|------|----------|
| Java Node 1 | http://localhost:8081 | /health |
| Java Node 2 | http://localhost:8082 | /health |
| Java Node 3 | http://localhost:8083 | /health |
| Rust Node 1 | http://localhost:8084 | /health |
| Rust Node 2 | http://localhost:8085 | /health |
| Rust Node 3 | http://localhost:8086 | /health |

### 服务提供者

| Provider | 地址 | API |
|----------|------|-----|
| Java Provider 1 | http://localhost:8087 | /sayHello |
| Java Provider 2 | http://localhost:8088 | /sayHello |
| Rust Provider 1 | http://localhost:8089 | /sayHello |
| Rust Provider 2 | http://localhost:8090 | /sayHello |

### 服务发现

```bash
# 查询服务实例
curl http://localhost:8081/api/discovery/instances?serviceId=hybrid-test-hello-service

# 查询集群状态
curl http://localhost:8081/api/cluster/status
```

## 故障排查

### 端口被占用

```bash
# 查找占用端口的进程
lsof -i :8081

# 终止进程
kill -9 <PID>
```

### Java 节点无法启动

```bash
# 检查日志
tail -f logs/java-node1.log

# 检查依赖
java -version
mvn -version
```

### Rust 节点无法启动

```bash
# 检查日志
tail -f logs/rust-node1.log

# 重新编译
cd ..
cargo build --release
```

### 数据库锁定

```bash
# 如果出现数据库锁定错误，可能是多个节点同时写入
# 解决方案：使用 WAL 模式或改为内存模式测试

# 检查数据库文件
ls -la data/artemis.db*
```

## 贡献

欢迎提交 Issue 和 PR 来改进测试框架。

## 许可证

MIT OR Apache-2.0
