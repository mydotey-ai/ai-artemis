use artemis_client::{ClientConfig, websocket::WebSocketClient};
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let config = ClientConfig {
        server_urls: vec!["http://localhost:8080".to_string()],
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
