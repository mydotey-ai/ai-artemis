use crate::{config::ClientConfig, error::Result};
use artemis_core::model::InstanceChange;
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{debug, error, info, warn};

/// Client-to-server WebSocket message types
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

/// Server-to-client WebSocket message types
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
        changes: Vec<InstanceChange>,
    },

    #[serde(rename = "pong")]
    Pong,

    #[serde(rename = "error")]
    Error { message: String },
}

/// Type alias for change listener callback
#[allow(dead_code)]
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

    /// Create a subscribe message for a given service.
    ///
    /// Returns a JSON string that can be sent through the WebSocket connection.
    pub fn create_subscribe_message(service_id: &str) -> String {
        let msg = ClientMessage::Subscribe {
            service_id: service_id.to_string(),
        };
        serde_json::to_string(&msg).unwrap()
    }

    /// Create an unsubscribe message for a given service.
    ///
    /// Returns a JSON string that can be sent through the WebSocket connection
    /// to cancel an active subscription.
    pub fn create_unsubscribe_message(service_id: &str) -> String {
        let msg = ClientMessage::Unsubscribe {
            service_id: service_id.to_string(),
        };
        serde_json::to_string(&msg).unwrap()
    }

    /// Connect to WebSocket and subscribe to service changes.
    ///
    /// Includes periodic ping/pong health checking to detect broken connections.
    /// Ping messages are sent at the interval configured in `websocket_ping_interval_secs`.
    /// The connection loop breaks on ping failure, server close, or stream errors.
    pub async fn connect_and_subscribe(self: Arc<Self>, service_id: String) -> Result<()> {
        let ws_url = self.config.server_urls[0]
            .replace("http://", "ws://")
            .replace("https://", "wss://");
        let url = format!("{}/ws", ws_url);

        info!("Connecting to WebSocket: {}", url);

        let (ws_stream, _) = connect_async(&url).await?;
        let (mut write, mut read) = ws_stream.split();

        // Send subscribe message
        let json = Self::create_subscribe_message(&service_id);
        write.send(Message::Text(json.into())).await?;

        info!("Subscribed to service: {}", service_id);

        // Ping interval timer
        let mut ping_interval = tokio::time::interval(self.config.websocket_ping_interval());

        loop {
            tokio::select! {
                // Periodically send ping
                _ = ping_interval.tick() => {
                    debug!("Sending WebSocket ping");
                    if let Err(e) = write.send(Message::Ping(vec![].into())).await {
                        error!("Failed to send ping: {}", e);
                        break;
                    }
                }

                // Receive messages
                msg = read.next() => {
                    match msg {
                        Some(Ok(Message::Text(text))) => {
                            if let Ok(server_msg) = serde_json::from_str::<ServerMessage>(&text) {
                                match server_msg {
                                    ServerMessage::ServiceChange { changes, .. } => {
                                        info!("Received {} changes", changes.len());
                                        let _ = self.change_tx.send(changes);
                                    }
                                    ServerMessage::Subscribed { service_id } => {
                                        info!("Confirmed subscription to: {}", service_id);
                                    }
                                    ServerMessage::Unsubscribed { service_id } => {
                                        info!("Confirmed unsubscription from: {}", service_id);
                                    }
                                    ServerMessage::Error { message } => {
                                        error!("Server error: {}", message);
                                    }
                                    ServerMessage::Pong => {
                                        debug!("Received application-level pong from server");
                                    }
                                }
                            }
                        }
                        Some(Ok(Message::Pong(_))) => {
                            debug!("Received pong from server");
                        }
                        Some(Ok(Message::Ping(data))) => {
                            debug!("Received ping from server, sending pong");
                            if let Err(e) = write.send(Message::Pong(data)).await {
                                error!("Failed to send pong: {}", e);
                                break;
                            }
                        }
                        Some(Ok(Message::Close(_))) => {
                            info!("WebSocket connection closed by server");
                            break;
                        }
                        Some(Err(e)) => {
                            error!("WebSocket error: {}", e);
                            break;
                        }
                        None => {
                            warn!("WebSocket stream ended");
                            break;
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_subscribe_message() {
        let msg = WebSocketClient::create_subscribe_message("my-service");
        let parsed: serde_json::Value = serde_json::from_str(&msg).unwrap();
        assert_eq!(parsed["type"], "subscribe");
        assert_eq!(parsed["service_id"], "my-service");
    }

    #[test]
    fn test_create_unsubscribe_message() {
        let msg = WebSocketClient::create_unsubscribe_message("my-service");
        let parsed: serde_json::Value = serde_json::from_str(&msg).unwrap();
        assert_eq!(parsed["type"], "unsubscribe");
        assert_eq!(parsed["service_id"], "my-service");
    }
}
