use crate::engine::Input;
use crate::price::WebSocketPriceFeed;
use altusd::{Coin, Exchange};
use serde::Deserialize;
use serde_json::json;
use tokio::sync::mpsc::Sender;

/// This function is responsible to subscribe to the Binance websocket price feed.
///
/// See `price.rs` for the details on the implementation of `WebSocketPriceFeed`.
pub async fn run(mpsc_tx: Sender<Input>) {
    let websocket_price_feed = WebSocketPriceFeed {
        endpoint: "wss://stream.binance.com:9443/ws",
        exchange: Exchange::Binance,
        subscribe: json!({
            "method": "SUBSCRIBE",
            "params": [
                "adausdt@ticker",
                "dogeusdt@ticker",
                "dotusdt@ticker",
                "ethusdt@ticker",
                "solusdt@ticker",
            ],
            "id": 1,
        }),
        message_handler,
    };
    websocket_price_feed.run(mpsc_tx).await;
}

/// This struct represents a message from the `ticker` channel. See this link for reference:
/// https://binance-docs.github.io/apidocs/spot/en/#individual-symbol-ticker-streams
#[derive(Deserialize)]
struct Message<'a> {
    e: &'a str,
    s: &'a str,
    c: &'a str,
    b: &'a str,
    a: &'a str,
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
    if message.e != "24hrTicker" {
        tracing::error!("unexpected message type: {}", message.e);
        return None;
    }

    // Extract coin.
    let coin = match message.s {
        "ADAUSDT" => Coin::ADA,
        "DOGEUSDT" => Coin::DOGE,
        "DOTUSDT" => Coin::DOT,
        "ETHUSDT" => Coin::ETH,
        "SOLUSDT" => Coin::SOL,
        _ => {
            tracing::error!("unexpected message coin: {}", message.s);
            return None;
        }
    };

    // Extract last price, best bid, and best ask.
    let last_price = crate::price::str_to_f64(message.c)?;
    let best_bid = crate::price::str_to_f64(message.b)?;
    let best_ask = crate::price::str_to_f64(message.a)?;
    Some((coin, [last_price, best_bid, best_ask]))
}
