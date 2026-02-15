use axum::extract::ws::{Message, WebSocket};
use dashmap::DashMap;
use futures::SinkExt;
use futures::stream::SplitSink;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

pub type SessionId = String;
pub type Sender = Arc<Mutex<SplitSink<WebSocket, Message>>>;

/// WebSocket会话管理器
#[derive(Clone)]
pub struct SessionManager {
    /// 会话映射: SessionId -> Sender
    sessions: Arc<DashMap<SessionId, Sender>>,

    /// 服务订阅: ServiceId -> Vec<SessionId>
    subscriptions: Arc<DashMap<String, Vec<SessionId>>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self { sessions: Arc::new(DashMap::new()), subscriptions: Arc::new(DashMap::new()) }
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
        self.subscriptions.entry(service_id.clone()).or_default().push(session_id.clone());

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
                        let mut sender_guard = sender.lock().await;
                        if let Err(e) = sender_guard.send(msg).await {
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
    fn test_session_manager_creation() {
        let manager = SessionManager::new();
        assert_eq!(manager.active_sessions(), 0);
    }

    #[test]
    fn test_session_manager_default() {
        let manager = SessionManager::default();
        assert_eq!(manager.active_sessions(), 0);
    }

    #[test]
    fn test_subscribe() {
        let manager = SessionManager::new();
        let session_id = "test-session-1".to_string();
        let service_id = "test-service".to_string();

        manager.subscribe(session_id.clone(), service_id.clone());

        // 验证订阅存在
        assert!(manager.subscriptions.contains_key(&service_id));
    }

    #[test]
    fn test_subscribe_multiple_sessions() {
        let manager = SessionManager::new();
        let session_1 = "test-session-1".to_string();
        let session_2 = "test-session-2".to_string();
        let service_id = "test-service".to_string();

        manager.subscribe(session_1.clone(), service_id.clone());
        manager.subscribe(session_2.clone(), service_id.clone());

        // 验证订阅包含两个会话
        if let Some(subs) = manager.subscriptions.get(&service_id) {
            assert_eq!(subs.len(), 2);
            assert!(subs.contains(&session_1));
            assert!(subs.contains(&session_2));
        } else {
            panic!("Service subscriptions not found");
        }
    }

    #[test]
    fn test_unsubscribe() {
        let manager = SessionManager::new();
        let session_id = "test-session-1".to_string();
        let service_id = "test-service".to_string();

        manager.subscribe(session_id.clone(), service_id.clone());
        manager.unsubscribe(&session_id, &service_id);

        // 验证订阅已清空
        if let Some(subs) = manager.subscriptions.get(&service_id) {
            assert!(subs.is_empty());
        }
    }

    #[test]
    fn test_unsubscribe_nonexistent_service() {
        let manager = SessionManager::new();
        let session_id = "test-session-1".to_string();
        let service_id = "nonexistent-service".to_string();

        // 取消订阅不存在的服务,应该不会panic
        manager.unsubscribe(&session_id, &service_id);
    }

    #[test]
    fn test_unregister_session_cleans_subscriptions() {
        let manager = SessionManager::new();
        let session_id = "test-session-1".to_string();
        let service_1 = "service-1".to_string();
        let service_2 = "service-2".to_string();

        // 订阅多个服务
        manager.subscribe(session_id.clone(), service_1.clone());
        manager.subscribe(session_id.clone(), service_2.clone());

        // 注销会话
        manager.unregister_session(&session_id);

        // 验证订阅已清空
        if let Some(subs) = manager.subscriptions.get(&service_1) {
            assert!(!subs.contains(&session_id), "Session should be removed from service-1");
        }
        if let Some(subs) = manager.subscriptions.get(&service_2) {
            assert!(!subs.contains(&session_id), "Session should be removed from service-2");
        }
    }

    #[test]
    fn test_multiple_sessions_same_service() {
        let manager = SessionManager::new();
        let session_1 = "session-1".to_string();
        let session_2 = "session-2".to_string();
        let session_3 = "session-3".to_string();
        let service_id = "test-service".to_string();

        // 3个会话订阅同一服务
        manager.subscribe(session_1.clone(), service_id.clone());
        manager.subscribe(session_2.clone(), service_id.clone());
        manager.subscribe(session_3.clone(), service_id.clone());

        // 验证订阅数量
        if let Some(subs) = manager.subscriptions.get(&service_id) {
            assert_eq!(subs.len(), 3);
        } else {
            panic!("Service subscriptions not found");
        }

        // 注销一个会话
        manager.unregister_session(&session_2);

        // 验证还有2个订阅
        if let Some(subs) = manager.subscriptions.get(&service_id) {
            assert_eq!(subs.len(), 2);
            assert!(subs.contains(&session_1));
            assert!(!subs.contains(&session_2));
            assert!(subs.contains(&session_3));
        }
    }

    #[test]
    fn test_session_manager_clone() {
        let manager = SessionManager::new();
        let session_id = "test-session-1".to_string();
        let service_id = "test-service".to_string();

        manager.subscribe(session_id.clone(), service_id.clone());

        // 克隆管理器
        let manager_clone = manager.clone();

        // 验证克隆的管理器可以访问相同的订阅
        assert!(manager_clone.subscriptions.contains_key(&service_id));
    }
}
