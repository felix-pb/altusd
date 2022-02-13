//! A simple websocket client to test the websocket server.

use futures::StreamExt;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

const ENDPOINT: &str = "ws://localhost:8080";

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let mut websocket_stream = connect_async(ENDPOINT).await.unwrap().0;
    tracing::info!("websocket connection established");
    while let Some(result) = websocket_stream.next().await {
        let message = result.unwrap();
        if let Message::Text(json) = message {
            tracing::info!("websocket message received: {}", json);
        } else {
            tracing::warn!("unexpected websocket message: {}", message);
        }
    }
}
