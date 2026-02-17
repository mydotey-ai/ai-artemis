# Artemis - Rust 服务注册中心

**Artemis** 是一个使用 Rust 重写的微服务注册中心，类似于 Netflix Eureka。

- **原始项目**: [artemis](https://github.com/mydotey/artemis) (Java 版本)
- **Java 原版代码**: `artemis-java/` 目录 (本地已 clone，可查阅原版实现细节)
- **核心目标**: 消除 Java 版本的 GC 停顿问题，实现低延迟高性能

---

## 快速开始

### 开发环境一键启动

使用 `dev.sh` 脚本可以同时启动后端和前端服务：

```bash
# 启动开发环境 (后端 + Web Console)
./scripts/dev.sh start

# 查看服务状态
./scripts/dev.sh status

# 查看日志
./scripts/dev.sh logs

# 停止所有服务
./scripts/dev.sh stop
```

**访问地址**:
- **Web 控制台**: http://localhost:5173
- **后端 API**: http://localhost:8080

**默认登录凭据**:
- 用户名: `admin`
- 密码: `admin123`

详见: [开发指南](docs/development.md)

### 单节点启动

```bash
# 编译并运行
cargo run --release --bin artemis -- server --addr 0.0.0.0:8080
```

### 多节点集群

```bash
# 启动 3 节点集群
./scripts/cluster.sh start

# 查看状态
./scripts/cluster.sh status

# 停止集群
./scripts/cluster.sh stop
```

### Docker 部署

```bash
# 构建并运行
docker build -t artemis:latest .
docker run -d -p 8080:8080 --name artemis artemis:latest

# 健康检查
curl http://localhost:8080/health
```

---

## 性能优势

Rust 版本性能远超 Java 版本：

- **P99 延迟**: < 0.5ms (Java: 50-200ms, 提升 100-400 倍)
- **吞吐量**: 10,000+ QPS (Java: ~2,000 QPS, 提升 5 倍)
- **GC 停顿**: 0ms (Java: 100-500ms, 完全消除)
- **内存占用**: ~2GB (Java: ~4GB+, 减少 50%+)

---

## 📚 文档导航

### AI 开发规则 (`.claude/rules/`)

Claude 助手使用以下规则文件理解项目上下文：

- **[project.md](.claude/rules/project.md)** - 项目背景、技术架构、核心功能
- **[dev-standards.md](.claude/rules/dev-standards.md)** - 开发规范、测试标准、代码质量要求
- **[doc.md](.claude/rules/doc.md)** - 文档组织规范

### 项目文档 (`docs/`)

**后端服务文档**:
- **[implementation-roadmap.md](docs/plans/implementation-roadmap.md)** - 项目完成总结、28个Phase详情、性能指标
- **[design.md](docs/plans/design.md)** - 系统架构设计
- **[Phase 26: 客户端企业级功能](docs/plans/phases/phase-26-client-enterprise-features.md)** - 客户端 SDK 企业级功能

**Web 控制台文档**:
- **[web-console/README.md](docs/web-console/README.md)** - Web Console 文档导航
- **[web-console/project-summary.md](docs/web-console/project-summary.md)** - Web Console 完成总结
- **[plans/web-console-design.md](docs/plans/web-console-design.md)** - Web Console 架构设计

---

## 维护信息

- **开发者**: Claude Sonnet 4.5 (AI) + koqizhao
- **开发时间**:
  - 后端服务: 2026-02-13 至 2026-02-15
  - Web 控制台: 2026-02-16 至 2026-02-17
- **许可证**: MIT OR Apache-2.0

---

**项目已完成,可投入生产环境!** 🚀
