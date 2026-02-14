# 阶段9: WebSocket完整实现

> **For Claude:** 实现WebSocket实时推送功能，包括服务端和客户端。参考Java实现: `artemis-java/artemis-server/`

**目标:** 完整实现WebSocket实时推送，支持服务变更通知

**预计任务数:** 4个Task

---

## Task 9.1: 实现SessionManager（会话管理）

**Files:**
- Create: `artemis-web/src/websocket/session.rs`
- Update: `artemis-web/src/websocket/mod.rs`

**Step 1: 实现SessionManager**

```rust
// artemis-web/src/websocket/mod.rs
pub mod handler;
pub mod session;

pub use handler::ws_handler;
pub use session::SessionManager;
```

```rust
// artemis-web/src/websocket/session.rs
use dashmap::DashMap;
use futures::stream::SplitSink;
use axum::extract::ws::{WebSocket, Message};
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

pub type SessionId = String;
pub type Sender = Arc<Mutex<SplitSink<WebSocket, Message>>>;

/// WebSocket会话管理器
pub struct SessionManager {
    /// 会话映射: SessionId -> Sender
    sessions: Arc<DashMap<SessionId, Sender>>,

    /// 服务订阅: ServiceId -> Vec<SessionId>
    subscriptions: Arc<DashMap<String, Vec<SessionId>>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(DashMap::new()),
            subscriptions: Arc::new(DashMap::new()),
        }
    }

    /// 注册新会话
    pub fn register_session(&self, sender: Sender) -> SessionId {
        let session_id = Uuid::new_v4().to_string();
        self.sessions.insert(session_id.clone(), sender);
        tracing::info!("WebSocket session registered: {}", session_id);
        session_id
    }

    /// 注销会话
    pub fn unregister_session(&self, session_id: &str) {
        self.sessions.remove(session_id);

        // 清理订阅
        self.subscriptions.iter_mut().for_each(|mut entry| {
            entry.value_mut().retain(|sid| sid != session_id);
        });

        tracing::info!("WebSocket session unregistered: {}", session_id);
    }

    /// 订阅服务
    pub fn subscribe(&self, session_id: SessionId, service_id: String) {
        self.subscriptions
            .entry(service_id.clone())
            .or_insert_with(Vec::new)
            .push(session_id.clone());

        tracing::info!("Session {} subscribed to service {}", session_id, service_id);
    }

    /// 取消订阅
    pub fn unsubscribe(&self, session_id: &str, service_id: &str) {
        if let Some(mut subs) = self.subscriptions.get_mut(service_id) {
            subs.retain(|sid| sid != session_id);
        }

        tracing::info!("Session {} unsubscribed from service {}", session_id, service_id);
    }

    /// 向订阅了某服务的所有会话推送消息
    pub async fn broadcast_to_service(&self, service_id: &str, message: Message) {
        if let Some(session_ids) = self.subscriptions.get(service_id) {
            for session_id in session_ids.value() {
                if let Some(sender) = self.sessions.get(session_id) {
                    let sender = sender.value().clone();
                    let msg = message.clone();

                    tokio::spawn(async move {
                        if let Err(e) = sender.lock().await.send(msg).await {
                            tracing::error!("Failed to send message: {}", e);
                        }
                    });
                }
            }
        }
    }

    /// 获取活跃会话数
    pub fn active_sessions(&self) -> usize {
        self.sessions.len()
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_registration() {
        let manager = SessionManager::new();
        assert_eq!(manager.active_sessions(), 0);

        // 注意：实际sender需要WebSocket，这里只是结构测试
    }
}
```

**Step 2: 提交**

```bash
git add artemis-web/src/websocket/
git commit -m "feat(web): implement SessionManager for WebSocket

- Add SessionManager for connection management
- Support session registration/unregistration
- Support service subscription/unsubscription
- Support broadcast to subscribed sessions
- Thread-safe with DashMap

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 9.2: 实现WebSocket Handler

**Files:**
- Create: `artemis-web/src/websocket/handler.rs`

**Step 1: 实现WebSocket处理器**

```rust
// artemis-web/src/websocket/handler.rs
use super::session::SessionManager;
use crate::state::AppState;
use axum::{
    extract::{ws::{WebSocket, WebSocketUpgrade, Message}, State},
    response::Response,
};
use futures::{stream::StreamExt, SinkExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
enum ClientMessage {
    #[serde(rename = "subscribe")]
    Subscribe { service_id: String },

    #[serde(rename = "unsubscribe")]
    Unsubscribe { service_id: String },

    #[serde(rename = "ping")]
    Ping,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
enum ServerMessage {
    #[serde(rename = "subscribed")]
    Subscribed { service_id: String },

    #[serde(rename = "unsubscribed")]
    Unsubscribed { service_id: String },

    #[serde(rename = "service_change")]
    ServiceChange {
        service_id: String,
        changes: Vec<serde_json::Value>,
    },

    #[serde(rename = "pong")]
    Pong,

    #[serde(rename = "error")]
    Error { message: String },
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let (sender, mut receiver) = socket.split();
    let sender = Arc::new(tokio::sync::Mutex::new(sender));

    // 注册会话
    let session_id = state.session_manager.register_session(sender.clone());

    // 处理消息
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                if let Err(e) = handle_text_message(&text, &session_id, &state, &sender).await {
                    tracing::error!("Error handling message: {}", e);

                    let error_msg = ServerMessage::Error {
                        message: e.to_string(),
                    };

                    if let Ok(json) = serde_json::to_string(&error_msg) {
                        let _ = sender.lock().await.send(Message::Text(json)).await;
                    }
                }
            }
            Ok(Message::Close(_)) => {
                tracing::info!("Client closed connection: {}", session_id);
                break;
            }
            Err(e) => {
                tracing::error!("WebSocket error: {}", e);
                break;
            }
            _ => {}
        }
    }

    // 注销会话
    state.session_manager.unregister_session(&session_id);
}

async fn handle_text_message(
    text: &str,
    session_id: &str,
    state: &AppState,
    sender: &Arc<tokio::sync::Mutex<futures::stream::SplitSink<WebSocket, Message>>>,
) -> anyhow::Result<()> {
    let client_msg: ClientMessage = serde_json::from_str(text)?;

    match client_msg {
        ClientMessage::Subscribe { service_id } => {
            state.session_manager.subscribe(session_id.to_string(), service_id.clone());

            let response = ServerMessage::Subscribed { service_id };
            let json = serde_json::to_string(&response)?;
            sender.lock().await.send(Message::Text(json)).await?;
        }

        ClientMessage::Unsubscribe { service_id } => {
            state.session_manager.unsubscribe(session_id, &service_id);

            let response = ServerMessage::Unsubscribed { service_id };
            let json = serde_json::to_string(&response)?;
            sender.lock().await.send(Message::Text(json)).await?;
        }

        ClientMessage::Ping => {
            let response = ServerMessage::Pong;
            let json = serde_json::to_string(&response)?;
            sender.lock().await.send(Message::Text(json)).await?;
        }
    }

    Ok(())
}
```

**Step 2: 添加SessionManager到AppState**

```rust
// artemis-web/src/state.rs
use crate::websocket::SessionManager;

pub struct AppState {
    pub registry_service: Arc<RegistryServiceImpl>,
    pub discovery_service: Arc<DiscoveryServiceImpl>,
    pub rate_limiter: Arc<RateLimiter>,
    pub cache: Arc<VersionedCacheManager>,
    pub session_manager: Arc<SessionManager>,  // 新增
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
            session_manager: Arc::new(SessionManager::new()),  // 新增
        }
    }
}
```

**Step 3: 添加WebSocket路由**

```rust
// artemis-web/src/server.rs
use crate::websocket;

let app = Router::new()
    // ... 现有路由 ...

    // WebSocket endpoint
    .route("/ws", get(websocket::ws_handler))

    // ... middleware ...
```

**Step 4: 提交**

```bash
git add artemis-web/
git commit -m "feat(web): implement WebSocket handler

- Add WebSocket upgrade handler
- Support subscribe/unsubscribe/ping messages
- Handle client connections and disconnections
- Integrate SessionManager into AppState
- Add /ws endpoint

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 9.3: 实现InstanceChangeManager（变更推送）

**Files:**
- Create: `artemis-server/src/change/mod.rs`
- Create: `artemis-server/src/change/manager.rs`

**Step 1: 实现ChangeManager**

```rust
// artemis-server/src/change/mod.rs
pub mod manager;

pub use manager::InstanceChangeManager;
```

```rust
// artemis-server/src/change/manager.rs
use artemis_core::model::{InstanceChange, InstanceKey};
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::mpsc;

type ChangeReceiver = mpsc::UnboundedReceiver<InstanceChange>;
type ChangeSender = mpsc::UnboundedSender<InstanceChange>;

/// 实例变更管理器
pub struct InstanceChangeManager {
    /// 服务变更通道: ServiceId -> Sender
    channels: Arc<DashMap<String, ChangeSender>>,
}

impl InstanceChangeManager {
    pub fn new() -> Self {
        Self {
            channels: Arc::new(DashMap::new()),
        }
    }

    /// 订阅服务变更
    pub fn subscribe(&self, service_id: &str) -> ChangeReceiver {
        let (tx, rx) = mpsc::unbounded_channel();
        self.channels.insert(service_id.to_string(), tx);
        rx
    }

    /// 发布实例变更
    pub fn publish(&self, service_id: &str, change: InstanceChange) {
        if let Some(sender) = self.channels.get(service_id) {
            if let Err(e) = sender.send(change) {
                tracing::error!("Failed to publish change: {}", e);
            }
        }
    }

    /// 发布实例注册事件
    pub fn publish_register(&self, instance: &artemis_core::model::Instance) {
        let change = InstanceChange {
            instance: instance.clone(),
            change_type: artemis_core::model::ChangeType::New,
            change_time: chrono::Utc::now(),
        };

        self.publish(&instance.service_id, change);
    }

    /// 发布实例注销事件
    pub fn publish_unregister(&self, key: &InstanceKey, instance: &artemis_core::model::Instance) {
        let change = InstanceChange {
            instance: instance.clone(),
            change_type: artemis_core::model::ChangeType::Delete,
            change_time: chrono::Utc::now(),
        };

        self.publish(&key.service_id, change);
    }
}

impl Default for InstanceChangeManager {
    fn default() -> Self {
        Self::new()
    }
}
```

**Step 2: 集成到RegistryServiceImpl**

```rust
// artemis-server/src/registry/service_impl.rs
use crate::change::InstanceChangeManager;

pub struct RegistryServiceImpl {
    repository: RegistryRepository,
    lease_manager: Arc<LeaseManager>,
    change_manager: Arc<InstanceChangeManager>,  // 新增
}

impl RegistryServiceImpl {
    pub fn new(
        repository: RegistryRepository,
        lease_manager: Arc<LeaseManager>,
        change_manager: Arc<InstanceChangeManager>,  // 新增
    ) -> Self {
        Self {
            repository,
            lease_manager,
            change_manager,
        }
    }
}

#[async_trait]
impl RegistryService for RegistryServiceImpl {
    async fn register(&self, request: RegisterRequest) -> RegisterResponse {
        for instance in &request.instances {
            let key = instance.key();

            self.repository.register(instance.clone());
            self.lease_manager.create_lease(key);

            // 发布变更事件
            self.change_manager.publish_register(instance);
        }
        // ...
    }

    async fn unregister(&self, request: UnregisterRequest) -> UnregisterResponse {
        for key in &request.instance_keys {
            if let Some(instance) = self.repository.remove(key) {
                self.lease_manager.remove_lease(key);

                // 发布变更事件
                self.change_manager.publish_unregister(key, &instance);
            }
        }
        // ...
    }
}
```

**Step 3: 更新lib.rs**

```rust
// artemis-server/src/lib.rs
pub mod change;

pub use change::InstanceChangeManager;
```

**Step 4: 提交**

```bash
git add artemis-server/
git commit -m "feat(server): implement InstanceChangeManager

- Add InstanceChangeManager for change event publishing
- Support subscribe/publish pattern
- Integrate with RegistryServiceImpl
- Publish register/unregister events

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 9.4: 实现WebSocket客户端

**Files:**
- Create: `artemis-client/src/websocket/client.rs`
- Update: `artemis-client/src/websocket/mod.rs`

**Step 1: 实现WebSocket客户端**

```rust
// artemis-client/src/websocket/mod.rs
pub mod client;

pub use client::WebSocketClient;
```

```rust
// artemis-client/src/websocket/client.rs
use crate::{config::ClientConfig, error::Result};
use artemis_core::model::InstanceChange;
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{error, info};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
enum ClientMessage {
    #[serde(rename = "subscribe")]
    Subscribe { service_id: String },

    #[serde(rename = "unsubscribe")]
    Unsubscribe { service_id: String },

    #[serde(rename = "ping")]
    Ping,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
enum ServerMessage {
    #[serde(rename = "subscribed")]
    Subscribed { service_id: String },

    #[serde(rename = "service_change")]
    ServiceChange {
        service_id: String,
        changes: Vec<InstanceChange>,
    },

    #[serde(rename = "pong")]
    Pong,

    #[serde(rename = "error")]
    Error { message: String },
}

pub type ChangeListener = Box<dyn Fn(Vec<InstanceChange>) + Send + Sync>;

pub struct WebSocketClient {
    config: ClientConfig,
    change_tx: mpsc::UnboundedSender<Vec<InstanceChange>>,
}

impl WebSocketClient {
    pub fn new(config: ClientConfig) -> (Self, mpsc::UnboundedReceiver<Vec<InstanceChange>>) {
        let (change_tx, change_rx) = mpsc::unbounded_channel();

        (
            Self {
                config,
                change_tx,
            },
            change_rx,
        )
    }

    /// 连接WebSocket并订阅服务
    pub async fn connect_and_subscribe(self: Arc<Self>, service_id: String) -> Result<()> {
        let ws_url = self.config.server_url.replace("http://", "ws://").replace("https://", "wss://");
        let url = format!("{}/ws", ws_url);

        info!("Connecting to WebSocket: {}", url);

        let (ws_stream, _) = connect_async(&url).await?;
        let (mut write, mut read) = ws_stream.split();

        // 发送订阅消息
        let subscribe_msg = ClientMessage::Subscribe {
            service_id: service_id.clone(),
        };
        let json = serde_json::to_string(&subscribe_msg)?;
        write.send(Message::Text(json)).await?;

        info!("Subscribed to service: {}", service_id);

        // 接收消息
        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    if let Ok(server_msg) = serde_json::from_str::<ServerMessage>(&text) {
                        match server_msg {
                            ServerMessage::ServiceChange { changes, .. } => {
                                info!("Received {} changes", changes.len());
                                let _ = self.change_tx.send(changes);
                            }
                            ServerMessage::Subscribed { service_id } => {
                                info!("Confirmed subscription to: {}", service_id);
                            }
                            ServerMessage::Error { message } => {
                                error!("Server error: {}", message);
                            }
                            _ => {}
                        }
                    }
                }
                Ok(Message::Close(_)) => {
                    info!("WebSocket connection closed");
                    break;
                }
                Err(e) => {
                    error!("WebSocket error: {}", e);
                    break;
                }
                _ => {}
            }
        }

        Ok(())
    }
}
```

**Step 2: 添加示例**

```rust
// artemis-client/examples/websocket_client.rs
use artemis_client::{ClientConfig, websocket::WebSocketClient};
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let config = ClientConfig::builder()
        .server_url("http://localhost:8080")
        .region_id("test")
        .zone_id("zone")
        .build();

    let (client, mut change_rx) = WebSocketClient::new(config);
    let client = Arc::new(client);

    // 连接并订阅
    let client_clone = client.clone();
    tokio::spawn(async move {
        if let Err(e) = client_clone
            .connect_and_subscribe("my-service".to_string())
            .await
        {
            eprintln!("WebSocket error: {}", e);
        }
    });

    // 接收变更
    while let Some(changes) = change_rx.recv().await {
        println!("Received changes: {:?}", changes);
    }

    Ok(())
}
```

**Step 3: 提交**

```bash
git add artemis-client/
git commit -m "feat(client): implement WebSocketClient

- Add WebSocketClient for real-time updates
- Support service subscription
- Receive and process InstanceChange events
- Add usage example

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## 阶段9完成标准

- ✅ SessionManager完整实现
- ✅ WebSocket Handler实现
- ✅ InstanceChangeManager实现
- ✅ WebSocketClient实现
- ✅ 集成测试验证
- ✅ 示例程序运行成功
- ✅ `cargo test --workspace` 全部通过
