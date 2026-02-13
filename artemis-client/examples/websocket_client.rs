use artemis_client::{websocket::WebSocketClient, ClientConfig};
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let config = ClientConfig {
        server_url: "http://localhost:8080".to_string(),
        heartbeat_interval_secs: 30,
    };

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
        println!("Received {} changes:", changes.len());
        for change in changes {
            println!("  - {:?}: {}", change.change_type, change.instance.instance_id);
        }
    }

    Ok(())
}
