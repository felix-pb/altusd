use crate::engine::Input;
use altusd::{Coin, Exchange};
use futures::{SinkExt, StreamExt};
use serde_json::Value;
use std::time::Duration;
use tokio::sync::mpsc::Sender;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

/// The interval at which we reconnect to the websocket server if an error occurs.
const RECONNECT_INTERVAL: Duration = Duration::from_secs(10);

/// This struct represent a generic websocket price feed connection to an exchange.
/// It should be implemented by all supported exchanges.
pub struct WebSocketPriceFeed {
    pub endpoint: &'static str,
    pub exchange: Exchange,
    pub subscribe: Value,
    pub message_handler: fn(String) -> Option<(Coin, [f64; 3])>,
}

impl WebSocketPriceFeed {
    /// This function is responsible for feeding the current market price of our index's
    /// altcoins to the core engine for a particular exchange. It does that by subscribing to the
    /// exchange's websocket server. If an error occurs, it tries to reconnect after 10 seconds.
    pub async fn run(mut self, mpsc_tx: Sender<Input>) {
        loop {
            self.subscribe_websocket_endpoint(&mpsc_tx).await;
            tokio::time::sleep(RECONNECT_INTERVAL).await;
        }
    }

    /// This function is responsible for subscribing to the exchange's websocket server, consuming
    /// and parsing the ticker messages, and feeding the updated market prices to the core engine.
    pub async fn subscribe_websocket_endpoint(&mut self, mpsc_tx: &Sender<Input>) {
        // Connect.
        let mut websocket_stream = match connect_async(self.endpoint).await {
            Ok((websocket_stream, _)) => websocket_stream,
            Err(error) => {
                tracing::error!("failed to connect to websocket server: {}", error);
                return;
            }
        };
        tracing::info!("connected to websocket server: {:?}", self.exchange);

        // Subscribe.
        let subscribe_message = Message::Text(self.subscribe.to_string());
        if let Err(error) = websocket_stream.send(subscribe_message).await {
            tracing::error!("failed to send subscribe request: {}", error);
        }

        // Consume and parse websocket messages in a loop.
        while let Some(Result::Ok(Message::Text(json))) = websocket_stream.next().await {
            if let Some((coin, mut prices)) = (self.message_handler)(json) {
                // Find the median and send it to the engine.
                if prices.iter().all(|price| price.is_finite()) {
                    // Safe unwrap: our slice doesn't contain a NaN. See this link for reference.
                    // https://doc.rust-lang.org/std/primitive.slice.html#method.sort_unstable_by
                    prices.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
                    let market_price = prices[1];
                    let input = Input::price(coin, self.exchange, market_price);
                    if let Err(error) = mpsc_tx.send(input).await {
                        tracing::error!("failed to send message in mpsc channel: {}", error);
                    }
                } else {
                    tracing::error!("found non-finite market price: {:?}", prices);
                }
            }
        }

        tracing::info!("disconnected from websocket server: {:?}", self.exchange);
    }
}

/// This function is a helper to parse an f64 from a string slice.
pub fn str_to_f64(string: &str) -> Option<f64> {
    match string.parse() {
        Ok(number) => Some(number),
        Err(error) => {
            tracing::error!("failed to parse number as f64: {}", error);
            None
        }
    }
}
