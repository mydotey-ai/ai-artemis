use crate::state::AppState;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
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

                    let error_msg = ServerMessage::Error {
                        message: e.to_string(),
                    };

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
            state
                .session_manager
                .subscribe(session_id.to_string(), service_id.clone());

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
