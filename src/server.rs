use crate::engine::Output;
use futures::SinkExt;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::watch::Receiver;
use tokio_tungstenite::tungstenite::Message;

/// The address of the websocket server.
const ADDR: &str = "0.0.0.0:8080";

/// This function is responsible for running the websocket server.
pub async fn run(watch_rx: Receiver<Output>) {
    let listener = TcpListener::bind(ADDR).await.unwrap();
    tracing::info!("websocket server started: {}", ADDR);

    // Accept tcp connections in a loop. Each connection is handled in its owned spawned task.
    loop {
        let (stream, addr) = match listener.accept().await {
            Ok((stream, addr)) => (stream, addr),
            Err(error) => {
                tracing::error!("failed to accept tcp connection: {}", error);
                continue;
            }
        };
        tokio::spawn(handle_connection(stream, addr, watch_rx.clone()));
    }
}

/// This function is responsible for handling a single websocket connection.
async fn handle_connection(stream: TcpStream, addr: SocketAddr, mut watch_rx: Receiver<Output>) {
    // Try to upgrade the tcp connection to a websocket connection.
    let mut websocket_stream = match tokio_tungstenite::accept_async(stream).await {
        Ok(websocket_stream) => websocket_stream,
        Err(error) => {
            tracing::warn!("failed to upgrade tcp connection: {}: {}", addr, error);
            return;
        }
    };
    tracing::info!("websocket client connected: {}", addr);

    // Watch for changes in the index price and forward it to the connected client.
    while watch_rx.changed().await.is_ok() {
        let message = match serde_json::to_string(&*watch_rx.borrow()) {
            Ok(message) => message,
            Err(error) => {
                tracing::error!("failed to serialize websocket message: {}", error);
                continue;
            }
        };

        if websocket_stream.send(Message::Text(message)).await.is_err() {
            tracing::info!("websocket client disconnected: {}", addr);
            return;
        }
    }
}
