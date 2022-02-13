use crate::engine::Input;
use crate::price::WebSocketPriceFeed;
use altusd::{Coin, Exchange};
use serde::Deserialize;
use serde_json::json;
use tokio::sync::mpsc::Sender;

/// This function is responsible to subscribe to the Coinbase websocket price feed.
///
/// See `price.rs` for the details on the implementation of `WebSocketPriceFeed`.
pub async fn run(mpsc_tx: Sender<Input>) {
    let websocket_price_feed = WebSocketPriceFeed {
        endpoint: "wss://ws-feed.exchange.coinbase.com",
        exchange: Exchange::Coinbase,
        subscribe: json!({
            "type": "subscribe",
            "product_ids": [
                "ADA-USD",
                "DOGE-USD",
                "DOT-USD",
                "ETH-USD",
                "SOL-USD",
            ],
            "channels": ["ticker"],
        }),
        message_handler,
    };
    websocket_price_feed.run(mpsc_tx).await;
}

/// This struct represents a message from the `ticker` channel. See this link for reference:
/// https://docs.cloud.coinbase.com/exchange/docs/websocket-channels#the-ticker-channel
#[derive(Deserialize)]
struct Message<'a> {
    r#type: &'a str,
    product_id: &'a str,
    price: &'a str,
    best_bid: &'a str,
    best_ask: &'a str,
}

/// This message handler tries to parse the last price, best bid, and best ask for an altcoin.
///
/// Effectively, this is a callback used for every websocket received from the exchange.
fn message_handler(message: String) -> Option<(Coin, [f64; 3])> {
    // Deserialize.
    let message = match serde_json::from_str::<Message>(&message) {
        Ok(message) => message,
        Err(_) => {
            tracing::warn!("discarded message: {}", message);
            return None;
        }
    };

    // Validate message type.
    if message.r#type != "ticker" {
        tracing::error!("unexpected message type: {}", message.r#type);
        return None;
    }

    // Extract coin.
    let coin = match message.product_id {
        "ADA-USD" => Coin::ADA,
        "DOGE-USD" => Coin::DOGE,
        "DOT-USD" => Coin::DOT,
        "ETH-USD" => Coin::ETH,
        "SOL-USD" => Coin::SOL,
        _ => {
            tracing::error!("unexpected message coin: {}", message.product_id);
            return None;
        }
    };

    // Extract last price, best bid, and best ask.
    let last_price = crate::price::str_to_f64(message.price)?;
    let best_bid = crate::price::str_to_f64(message.best_bid)?;
    let best_ask = crate::price::str_to_f64(message.best_ask)?;
    Some((coin, [last_price, best_bid, best_ask]))
}
