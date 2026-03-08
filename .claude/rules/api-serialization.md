# API 序列化规范

## 核心原则

**所有 API 接口、内部通信、数据交换场景，统一使用 camelCase 序列化格式。**

这是前后端通信的契约基础，确保数据格式一致性，避免字段名不匹配导致的解析错误。

---

## 适用范围

### 必须使用 camelCase 的场景

| 场景 | 说明 |
|------|------|
| **REST API 响应** | 所有 HTTP JSON 响应体 |
| **REST API 请求** | 所有 HTTP JSON 请求体 |
| **WebSocket 消息** | 实时推送的 JSON 消息 |
| **集群内部通信** | 节点间数据复制、心跳同步 |
| **配置文件** | JSON 格式的配置文件 |
| **日志输出** | 结构化 JSON 日志中的业务字段 |

### 不受此规则约束的场景

| 场景 | 格式 | 说明 |
|------|------|------|
| **数据库字段** | snake_case | 数据库命名约定 |
| **Rust 代码字段** | snake_case | Rust 命名约定 |
| **URL 路径参数** | kebab-case | URL 规范 |
| **HTTP Header** | Header-Case | HTTP 规范 |

---

## 后端实现规范 (Rust)

### 所有需要进行序列化的结构体必须添加 serde 注解

**适用范围**：
- API Request/Response 结构体
- WebSocket 消息结构体
- 集群复制/同步结构体
- 其他需要 JSON 序列化的结构体

**不适用**：
- 纯内部计算结构体（无 Serialize/Deserialize）
- 数据库实体（使用数据库字段命名 snake_case）
- 配置结构体（使用配置文件命名约定）

**Request/Response/Model 结构体**:

```rust
use serde::{Deserialize, Serialize};

/// ✅ 正确: 使用 rename_all = "camelCase"
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceInstance {
    pub instance_id: String,
    pub service_id: String,
    pub ip_addr: String,
    pub port: u16,
    pub status: InstanceStatus,
}

/// ❌ 错误: 缺少 serde 注解会导致 snake_case 输出
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BadExample {
    pub instance_id: String,  // 会输出 "instance_id" 而非 "instanceId"
}
```

### 枚举类型同样需要配置

```rust
/// ✅ 正确: 枚举变体也使用 camelCase
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum InstanceStatus {
    Up,
    Down,
    Starting,
    OutOfService,
}

// 序列化结果: "up", "down", "starting", "outOfService"
```

### 嵌套结构体

```rust
/// ✅ 正确: 每个结构体都需要单独配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiResponse<T> {
    pub code: i32,
    pub message: String,
    pub data: T,  // T 也必须是 camelCase 序列化的结构体
}
```

### 特殊字段处理

```rust
/// 使用 skip 跳过不需要序列化的字段
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceInstance {
    pub instance_id: String,

    #[serde(skip)]
    pub internal_cache: HashMap<String, String>,  // 不序列化到 API

    /// 自定义字段名
    #[serde(rename = "vipAddress")]
    pub vip_address: Option<String>,
}
```

---

## 前端实现规范 (TypeScript)

### 类型定义

```typescript
// ✅ 正确: 直接使用 camelCase 定义接口
interface ServiceInstance {
  instanceId: string;
  serviceId: string;
  ipAddr: string;
  port: number;
  status: InstanceStatus;
}

// ❌ 错误: 不要使用 snake_case
interface BadExample {
  instance_id: string;  // 与后端 API 格式不一致
}
```

### API 调用

```typescript
// ✅ 正确: 直接使用 camelCase 对象
const response = await fetch('/api/instances', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    instanceId: 'instance-1',
    serviceId: 'service-a',
    ipAddr: '192.168.1.1',
    port: 8080,
  }),
});

// ✅ 正确: 响应直接解析为 camelCase
const data: ServiceInstance = await response.json();
```

---

## 验证检查清单

### 新增 API 时必须验证

- [ ] Rust 结构体添加 `#[serde(rename_all = "camelCase")]`
- [ ] 嵌套结构体同样配置 serde 注解
- [ ] 枚举类型配置 serde 注解
- [ ] 前端 TypeScript 类型定义使用 camelCase
- [ ] 使用 API 测试工具验证 JSON 输出格式

### Code Review 检查点

```bash
# 检查 Rust 结构体是否配置 serde
# 新增结构体应包含此注解
grep -r "derive.*Serialize.*Deserialize" --include="*.rs" | grep -v "rename_all"
```

---

## 常见错误示例

### 错误 1: 遗漏 serde 注解

```rust
// ❌ 错误: 输出 {"instance_id": "...", "service_id": "..."}
#[derive(Serialize, Deserialize)]
pub struct Instance {
    pub instance_id: String,
}

// ✅ 正确: 输出 {"instanceId": "...", "serviceId": "..."}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Instance {
    pub instance_id: String,
}
```

### 错误 2: 前后端字段名不一致

```typescript
// ❌ 前端使用 snake_case
interface Instance {
  instance_id: string;  // API 返回 "instanceId"，解析失败
}

// ✅ 前端使用 camelCase
interface Instance {
  instanceId: string;  // 与 API 格式一致
}
```

---

## 相关文档

- **开发规范**: [dev-standards.md](dev-standards.md)
- **项目上下文**: [project.md](project.md)
- **API 设计**: [docs/plans/design.md](../../docs/plans/design.md)