use crate::state::AppState;
use axum::{
    extract::{
        State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::Response,
};
use futures::{SinkExt, stream::StreamExt};
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
    ServiceChange { service_id: String, changes: Vec<serde_json::Value> },

    #[serde(rename = "pong")]
    Pong,

    #[serde(rename = "error")]
    Error { message: String },
}

pub async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
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

                    let error_msg = ServerMessage::Error { message: e.to_string() };

                    if let Ok(json) = serde_json::to_string(&error_msg) {
                        let _ = sender.lock().await.send(Message::Text(json.into())).await;
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
            sender.lock().await.send(Message::Text(json.into())).await?;
        }

        ClientMessage::Unsubscribe { service_id } => {
            state.session_manager.unsubscribe(session_id, &service_id);

            let response = ServerMessage::Unsubscribed { service_id };
            let json = serde_json::to_string(&response)?;
            sender.lock().await.send(Message::Text(json.into())).await?;
        }

        ClientMessage::Ping => {
            let response = ServerMessage::Pong;
            let json = serde_json::to_string(&response)?;
            sender.lock().await.send(Message::Text(json.into())).await?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========== ClientMessage 序列化测试 ==========

    #[test]
    fn test_subscribe_message_serialization() {
        let msg = ClientMessage::Subscribe {
            service_id: "my-service".to_string(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""type":"subscribe""#));
        assert!(json.contains(r#""service_id":"my-service""#));
    }

    #[test]
    fn test_subscribe_message_deserialization() {
        let json = r#"{"type":"subscribe","service_id":"my-service"}"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();
        match msg {
            ClientMessage::Subscribe { service_id } => {
                assert_eq!(service_id, "my-service");
            }
            _ => panic!("Expected Subscribe message"),
        }
    }

    #[test]
    fn test_unsubscribe_message_serialization() {
        let msg = ClientMessage::Unsubscribe {
            service_id: "my-service".to_string(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""type":"unsubscribe""#));
        assert!(json.contains(r#""service_id":"my-service""#));
    }

    #[test]
    fn test_unsubscribe_message_deserialization() {
        let json = r#"{"type":"unsubscribe","service_id":"my-service"}"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();
        match msg {
            ClientMessage::Unsubscribe { service_id } => {
                assert_eq!(service_id, "my-service");
            }
            _ => panic!("Expected Unsubscribe message"),
        }
    }

    #[test]
    fn test_ping_message_serialization() {
        let msg = ClientMessage::Ping;
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""type":"ping""#));
    }

    #[test]
    fn test_ping_message_deserialization() {
        let json = r#"{"type":"ping"}"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();
        assert!(matches!(msg, ClientMessage::Ping));
    }

    // ========== ServerMessage 序列化测试 ==========

    #[test]
    fn test_subscribed_message_serialization() {
        let msg = ServerMessage::Subscribed {
            service_id: "my-service".to_string(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""type":"subscribed""#));
        assert!(json.contains(r#""service_id":"my-service""#));
    }

    #[test]
    fn test_unsubscribed_message_serialization() {
        let msg = ServerMessage::Unsubscribed {
            service_id: "my-service".to_string(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""type":"unsubscribed""#));
        assert!(json.contains(r#""service_id":"my-service""#));
    }

    #[test]
    fn test_pong_message_serialization() {
        let msg = ServerMessage::Pong;
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""type":"pong""#));
    }

    #[test]
    fn test_error_message_serialization() {
        let msg = ServerMessage::Error {
            message: "Invalid request".to_string(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""type":"error""#));
        assert!(json.contains(r#""message":"Invalid request""#));
    }

    #[test]
    fn test_service_change_message_serialization() {
        let changes = vec![
            serde_json::json!({"instance_id": "inst-1", "status": "up"}),
            serde_json::json!({"instance_id": "inst-2", "status": "down"}),
        ];
        let msg = ServerMessage::ServiceChange {
            service_id: "my-service".to_string(),
            changes,
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""type":"service_change""#));
        assert!(json.contains(r#""service_id":"my-service""#));
        assert!(json.contains(r#""instance_id":"inst-1""#));
        assert!(json.contains(r#""instance_id":"inst-2""#));
    }

    // ========== 错误处理测试 ==========

    #[test]
    fn test_invalid_message_type() {
        let json = r#"{"type":"unknown","data":"test"}"#;
        let result: Result<ClientMessage, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_malformed_json() {
        let json = r#"{"type":"subscribe","service_id":}"#;
        let result: Result<ClientMessage, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_required_field() {
        let json = r#"{"type":"subscribe"}"#;
        let result: Result<ClientMessage, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    // ========== 消息往返测试 (序列化 → 反序列化) ==========

    #[test]
    fn test_subscribe_roundtrip() {
        let original = ClientMessage::Subscribe {
            service_id: "test-service".to_string(),
        };
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: ClientMessage = serde_json::from_str(&json).unwrap();
        match deserialized {
            ClientMessage::Subscribe { service_id } => {
                assert_eq!(service_id, "test-service");
            }
            _ => panic!("Expected Subscribe message"),
        }
    }

    #[test]
    fn test_server_error_roundtrip() {
        let original = ServerMessage::Error {
            message: "Connection failed".to_string(),
        };
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: ServerMessage = serde_json::from_str(&json).unwrap();
        match deserialized {
            ServerMessage::Error { message } => {
                assert_eq!(message, "Connection failed");
            }
            _ => panic!("Expected Error message"),
        }
    }
}
