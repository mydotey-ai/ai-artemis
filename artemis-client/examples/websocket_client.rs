use artemis_client::{ClientConfig, websocket::WebSocketClient};
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Use DEBUG level to observe ping/pong health check messages
    tracing_subscriber::fmt().with_max_level(tracing::Level::DEBUG).init();

    let config = ClientConfig {
        server_urls: vec!["http://localhost:8080".to_string()],
        websocket_ping_interval_secs: 10, // Ping every 10 seconds
        ..Default::default()
    };

    let (client, mut change_rx) = WebSocketClient::new(config);
    let client = Arc::new(client);

    // Connect and subscribe
    let client_clone = client.clone();
    tokio::spawn(async move {
        if let Err(e) = client_clone.connect_and_subscribe("my-service".to_string()).await {
            eprintln!("WebSocket error: {}", e);
        }
    });

    // Receive changes
    while let Some(changes) = change_rx.recv().await {
        println!("Received {} changes:", changes.len());
        for change in changes {
            println!("  - {:?}: {}", change.change_type, change.instance.instance_id);
        }
    }

    Ok(())
}
