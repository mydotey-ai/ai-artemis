# 阶段4: artemis-web实现

> **For Claude:** HTTP/WebSocket API层，使用Axum框架。参考Java实现: `artemis-java/artemis-server/`

**优先级**: P0 (必须完成)
**状态**: ✅ **已完成** (2026-02-13)
**目标:** 实现完整的Web API层
**任务数:** 5个Task

---

## Task 4.1: 实现Web Server和路由

**Files:**
- Create: `artemis-web/src/server.rs`
- Create: `artemis-web/src/state.rs`
- Update: `artemis-web/src/lib.rs`

**Step 1: 实现AppState**

```rust
// artemis-web/src/state.rs
use artemis_server::{
    cache::VersionedCacheManager, discovery::DiscoveryServiceImpl,
    ratelimiter::RateLimiter, registry::RegistryServiceImpl,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub registry_service: Arc<RegistryServiceImpl>,
    pub discovery_service: Arc<DiscoveryServiceImpl>,
    pub rate_limiter: Arc<RateLimiter>,
    pub cache: Arc<VersionedCacheManager>,
}

impl AppState {
    pub fn new(
        registry_service: RegistryServiceImpl,
        discovery_service: DiscoveryServiceImpl,
        rate_limiter: RateLimiter,
        cache: Arc<VersionedCacheManager>,
    ) -> Self {
        Self {
            registry_service: Arc::new(registry_service),
            discovery_service: Arc::new(discovery_service),
            rate_limiter: Arc::new(rate_limiter),
            cache,
        }
    }
}
```

**Step 2: 实现Web Server**

```rust
// artemis-web/src/server.rs
use crate::api;
use crate::middleware::rate_limit_middleware;
use crate::state::AppState;
use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use tracing::info;

pub struct WebServer {
    addr: SocketAddr,
    app: Router,
}

impl WebServer {
    pub fn new(host: &str, port: u16, state: AppState) -> Self {
        let addr: SocketAddr = format!("{}:{}", host, port).parse().unwrap();

        let app = Router::new()
            // Registry API (支持.json后缀以兼容Java版本)
            .route("/api/registry/register", post(api::registry::register))
            .route("/api/registry/register.json", post(api::registry::register))
            .route("/api/registry/heartbeat", post(api::registry::heartbeat))
            .route("/api/registry/heartbeat.json", post(api::registry::heartbeat))
            .route("/api/registry/unregister", post(api::registry::unregister))
            .route("/api/registry/unregister.json", post(api::registry::unregister))
            // Discovery API (支持.json后缀和多种命名方式)
            .route("/api/discovery/getservice", post(api::discovery::get_service))
            .route("/api/discovery/service", post(api::discovery::get_service))
            .route("/api/discovery/service.json", post(api::discovery::get_service))
            .route("/api/discovery/getservices", post(api::discovery::get_services))
            .route("/api/discovery/services", post(api::discovery::get_services))
            .route("/api/discovery/services.json", post(api::discovery::get_services))
            .route("/api/discovery/getservices-delta", post(api::discovery::get_services_delta))
            .route("/api/discovery/services-delta", post(api::discovery::get_services_delta))
            .route("/api/discovery/services-delta.json", post(api::discovery::get_services_delta))
            // Health check
            .route("/health", get(api::health::health_check))
            // Middleware
            .layer(axum::middleware::from_fn_with_state(
                state.clone(),
                rate_limit_middleware,
            ))
            .layer(TraceLayer::new_for_http())
            .with_state(state);

        Self { addr, app }
    }

    pub async fn run(self) -> anyhow::Result<()> {
        info!("Starting Artemis Web Server on {}", self.addr);
        let listener = tokio::net::TcpListener::bind(self.addr).await?;
        axum::serve(listener, self.app).await?;
        Ok(())
    }
}
```

**Step 3: 更新lib.rs**

```rust
// artemis-web/src/lib.rs
//! Artemis Web - HTTP/WebSocket API层

pub mod api;
pub mod middleware;
pub mod server;
pub mod state;
pub mod websocket;

pub use server::WebServer;
pub use state::AppState;
```

**Step 4: 提交**

```bash
git add artemis-web/src/server.rs artemis-web/src/state.rs artemis-web/src/lib.rs
git commit -m "feat(web): implement WebServer and AppState

- Add AppState with all service dependencies
- Create WebServer with Axum router
- Define API routes (registry, discovery, health)
- Add TraceLayer middleware

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 4.2: 实现Registry API Handlers

**Files:**
- Create: `artemis-web/src/api/mod.rs`
- Create: `artemis-web/src/api/registry.rs`

**Step 1: 创建api模块**

```rust
// artemis-web/src/api/mod.rs
pub mod discovery;
pub mod health;
pub mod registry;
```

**Step 2: 实现Registry Handlers**

```rust
// artemis-web/src/api/registry.rs
use crate::state::AppState;
use artemis_core::model::{HeartbeatRequest, RegisterRequest, UnregisterRequest};
use artemis_core::traits::RegistryService;
use axum::{extract::State, http::StatusCode, Json};

pub async fn register(
    State(state): State<AppState>,
    Json(request): Json<RegisterRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    let response = state.registry_service.register(request).await;
    let status = match response.response_status.error_code {
        artemis_core::model::ErrorCode::Success => StatusCode::OK,
        _ => StatusCode::BAD_REQUEST,
    };
    (status, Json(serde_json::to_value(response).unwrap()))
}

pub async fn heartbeat(
    State(state): State<AppState>,
    Json(request): Json<HeartbeatRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    let response = state.registry_service.heartbeat(request).await;
    let status = match response.response_status.error_code {
        artemis_core::model::ErrorCode::Success => StatusCode::OK,
        _ => StatusCode::BAD_REQUEST,
    };
    (status, Json(serde_json::to_value(response).unwrap()))
}

pub async fn unregister(
    State(state): State<AppState>,
    Json(request): Json<UnregisterRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    let response = state.registry_service.unregister(request).await;
    (
        StatusCode::OK,
        Json(serde_json::to_value(response).unwrap()),
    )
}
```

**Step 3: 验证编译**

```bash
cargo check -p artemis-web
```

Expected: 编译成功

**Step 4: 提交**

```bash
git add artemis-web/src/api/
git commit -m "feat(web): implement Registry API handlers

- Add register/heartbeat/unregister endpoints
- Return appropriate HTTP status codes
- JSON request/response serialization

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 4.3: 实现Discovery API Handlers

**Files:**
- Create: `artemis-web/src/api/discovery.rs`
- Create: `artemis-web/src/api/health.rs`

**Step 1: 实现Discovery Handlers**

```rust
// artemis-web/src/api/discovery.rs
use crate::state::AppState;
use artemis_core::model::{
    GetServiceRequest, GetServicesDeltaRequest, GetServicesRequest,
};
use artemis_core::traits::DiscoveryService;
use axum::{extract::State, http::StatusCode, Json};

pub async fn get_service(
    State(state): State<AppState>,
    Json(request): Json<GetServiceRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    let response = state.discovery_service.get_service(request).await;
    let status = match response.response_status.error_code {
        artemis_core::model::ErrorCode::Success => StatusCode::OK,
        _ => StatusCode::NOT_FOUND,
    };
    (status, Json(serde_json::to_value(response).unwrap()))
}

pub async fn get_services(
    State(state): State<AppState>,
    Json(request): Json<GetServicesRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    let response = state.discovery_service.get_services(request).await;
    (
        StatusCode::OK,
        Json(serde_json::to_value(response).unwrap()),
    )
}

pub async fn get_services_delta(
    State(state): State<AppState>,
    Json(request): Json<GetServicesDeltaRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    let response = state.discovery_service.get_services_delta(request).await;
    (
        StatusCode::OK,
        Json(serde_json::to_value(response).unwrap()),
    )
}
```

**Step 2: 实现Health Check**

```rust
// artemis-web/src/api/health.rs
use axum::{http::StatusCode, Json};
use serde_json::json;

pub async fn health_check() -> (StatusCode, Json<serde_json::Value>) {
    (StatusCode::OK, Json(json!({ "status": "UP" })))
}
```

**Step 3: 提交**

```bash
git add artemis-web/src/api/
git commit -m "feat(web): implement Discovery API and health check

- Add get_service/get_services/get_services_delta endpoints
- Add health check endpoint
- Proper status code handling

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 4.4: 实现Middleware（限流和日志）

**Files:**
- Create: `artemis-web/src/middleware/mod.rs`

**Step 1: 实现Middleware**

```rust
// artemis-web/src/middleware/mod.rs
use crate::state::AppState;
use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};

/// 限流中间件
pub async fn rate_limit_middleware<B>(
    State(state): State<AppState>,
    request: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    if !state.rate_limiter.check() {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }
    Ok(next.run(request).await)
}
```

**Step 2: 提交**

```bash
git add artemis-web/src/middleware/
git commit -m "feat(web): implement rate limiting middleware

- Check rate limiter before processing requests
- Return 429 Too Many Requests on limit exceeded

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 4.5: 实现WebSocket支持（占位）

**Files:**
- Create: `artemis-web/src/websocket/mod.rs`

**Step 1: 创建WebSocket模块占位**

```rust
// artemis-web/src/websocket/mod.rs
//! WebSocket实时推送模块
//!
//! 用于向客户端实时推送服务变更
//! TODO: 在后续迭代中实现
```

**Step 2: 验证整体编译**

```bash
cargo check --workspace
```

Expected: 所有crate编译成功

**Step 3: 提交**

```bash
git add artemis-web/src/websocket/
git commit -m "feat(web): add WebSocket module placeholder

- Create module structure for future real-time push
- Add TODO for implementation

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## 阶段4完成标准

- ✅ WebServer和路由配置
- ✅ AppState管理
- ✅ Registry API实现
- ✅ Discovery API实现
- ✅ Health check实现
- ✅ 限流中间件
- ✅ WebSocket占位
- ✅ `cargo check -p artemis-web` 通过
