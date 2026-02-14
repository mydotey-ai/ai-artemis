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
    ServiceChange { service_id: String, changes: Vec<InstanceChange> },

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

        (Self { config, change_tx }, change_rx)
    }

    /// 连接WebSocket并订阅服务
    pub async fn connect_and_subscribe(self: Arc<Self>, service_id: String) -> Result<()> {
        let ws_url =
            self.config.server_url.replace("http://", "ws://").replace("https://", "wss://");
        let url = format!("{}/ws", ws_url);

        info!("Connecting to WebSocket: {}", url);

        let (ws_stream, _) = connect_async(&url).await?;
        let (mut write, mut read) = ws_stream.split();

        // 发送订阅消息
        let subscribe_msg = ClientMessage::Subscribe { service_id: service_id.clone() };
        let json = serde_json::to_string(&subscribe_msg)?;
        write.send(Message::Text(json.into())).await?;

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
