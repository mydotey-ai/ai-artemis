# Artemis 项目上下文

## 项目定位

**Artemis** 是一个使用 Rust 重写的微服务注册中心，完全兼容 10 年前在携程(ctrip.com)开发的 Java 版本。

- **原始项目**: [artemis](https://github.com/mydotey/artemis) (Java 1.5.16)
- **类似产品**: Netflix Eureka, Consul, Nacos
- **核心目标**: 解决 Java 版本在托管大量服务实例时的严重 GC 停顿问题

## 核心问题

Java 版本存在以下痛点：
- ⚠️ **GC 停顿**: 100-500ms 的 STW (Stop-The-World),导致服务抖动
- ⚠️ **延迟不可控**: P99 延迟 50-200ms,影响用户体验
- ⚠️ **扩展性受限**: 托管 50k 实例时性能急剧下降
- ⚠️ **内存占用高**: 4GB+ 内存占用,成本高昂

Rust 版本通过零 GC、无锁并发、零拷贝设计完全解决了这些问题。

---

## 技术架构

### Crate 组织结构

```
artemis-workspace/
├── artemis-core/          # 核心数据模型、Trait、错误类型
├── artemis-server/        # 业务逻辑层 (注册、发现、租约、缓存)
├── artemis-web/           # HTTP API 层 (Axum + WebSocket)
├── artemis-management/    # 管理功能和数据持久化
├── artemis-client/        # 客户端 SDK (企业级功能,100%对齐Java版本)
└── artemis/               # CLI 二进制 (服务器 + 管理工具)
```

**模块职责**:
- **artemis-core**: 定义所有数据结构 (ServiceInstance, DiscoveryConfig 等) 和 Trait 接口
- **artemis-server**: 实现核心业务逻辑 (RegistryService, DiscoveryService, LeaseManager 等)
- **artemis-web**: 提供 HTTP REST API 和 WebSocket 实时推送
- **artemis-management**: 高级管理功能 (分组路由、Zone管理、审计日志、持久化)
- **artemis-client**: 客户端 SDK,支持服务注册/发现/心跳/实时订阅
- **artemis**: 可执行文件,提供 CLI 命令

### 技术栈

| 组件 | 技术选型 | 用途 |
|------|----------|------|
| **异步运行时** | Tokio | 高性能异步 I/O 和任务调度 |
| **Web 框架** | Axum | HTTP REST API 和 WebSocket |
| **数据库 ORM** | SeaORM | 支持 SQLite/MySQL 运行时切换 |
| **并发数据结构** | DashMap | Lock-free 并发 HashMap |
| **限流** | Governor | Token Bucket 限流算法 |
| **监控** | Prometheus + OpenTelemetry | 指标采集和分布式追踪 |
| **测试** | Criterion | 性能基准测试 |
| **工具链** | Rust 1.93 | 编译器和工具链版本 |

---

## 核心功能概览

### 服务注册与发现
- **服务注册**: 实例注册、心跳续约、自动过期
- **服务发现**: 实例查询、版本化缓存、增量同步
- **租约管理**: 基于 TTL 的自动过期和清理

### 集群和复制
- **集群节点管理**: 节点注册、健康检查、自动下线
- **数据复制**: 异步复制、批处理、智能重试队列
- **批量优化**: 批处理窗口 100ms,批次大小 100 个实例,网络请求减少 90%+

### 实时推送
- **WebSocket**: 服务变更实时推送给客户端
- **订阅管理**: 按 serviceId 订阅,自动广播变更

### 高级管理功能
- **实例管理**: 实例拉入/拉出、服务器批量操作、状态查询
- **分组路由**: 加权轮询、就近访问、规则引擎
- **Zone管理**: Zone级别批量操作
- **金丝雀发布**: IP白名单机制
- **审计日志**: 操作记录、多维度查询

### 数据持久化
- **ORM**: SeaORM 支持 SQLite/MySQL 运行时切换
- **DAO 层**: GroupDao, RouteRuleDao, ZoneOperationDao, CanaryConfigDao
- **自动加载**: 服务启动时自动恢复配置

---

## 性能优势

| 指标 | Rust 版本 | Java 版本 | 改进 |
|------|-----------|-----------|------|
| **P99 延迟** | < 0.5ms | 50-200ms | **100-400x** |
| **吞吐量** | 10,000+ QPS | ~2,000 QPS | **5x** |
| **内存占用** | ~2GB (100k 实例) | ~4GB+ | **50%+** |
| **GC 停顿** | 0ms (无 GC) | 100-500ms | **消除** |
| **实例容量** | 100,000+ | ~50,000 | **2x** |

---

## 项目状态

✅ **100% 完成** - 所有 25 个 Phase 全部实现
- 101 个 API 端点全部实现
- 454 个单元测试 + 11 个集成测试脚本
- 零编译警告,代码覆盖率 62%+

详见: [`docs/plans/implementation-roadmap.md`](../../docs/plans/implementation-roadmap.md)

---

## 相关文档

- **开发规范**: [.claude/rules/dev-standards.md](dev-standards.md)
- **文档组织**: [.claude/rules/doc.md](doc.md)
- **架构设计**: [`docs/plans/design.md`](../../docs/plans/design.md)
- **实施路线图**: [`docs/plans/implementation-roadmap.md`](../../docs/plans/implementation-roadmap.md)
