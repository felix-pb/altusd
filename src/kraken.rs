use crate::engine::Input;
use crate::price::WebSocketPriceFeed;
use altusd::{Coin, Exchange};
use serde::Deserialize;
use serde_json::json;
use tokio::sync::mpsc::Sender;

/// This function is responsible to subscribe to the Kraken websocket price feed.
///
/// See `price.rs` for the details on the implementation of `WebSocketPriceFeed`.
pub async fn run(mpsc_tx: Sender<Input>) {
    let websocket_price_feed = WebSocketPriceFeed {
        endpoint: "wss://ws.kraken.com",
        exchange: Exchange::Kraken,
        subscribe: json!({
            "event": "subscribe",
            "pair": [
                "ADA/USD",
                "DOGE/USD",
                "DOT/USD",
                "ETH/USD",
                "SOL/USD",
            ],
            "subscription": {
                "name": "ticker",
            },
        }),
        message_handler,
    };
    websocket_price_feed.run(mpsc_tx).await;
}

/// These structs represent a message from the `ticker` channel. See this link for reference:
/// https://docs.kraken.com/websockets/#message-ticker
#[derive(Deserialize)]
struct Message<'a>(u64, MessageDetail<'a>, &'a str, &'a str);

#[derive(Deserialize)]
struct MessageDetail<'a> {
    #[serde(borrow)]
    a: (&'a str, u64, &'a str),
    b: (&'a str, u64, &'a str),
    c: (&'a str, &'a str),
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
    if message.2 != "ticker" {
        tracing::error!("unexpected message type: {}", message.2);
        return None;
    }

    // Extract coin.
    let coin = match message.3 {
        "ADA/USD" => Coin::ADA,
        "XDG/USD" => Coin::DOGE,
        "DOT/USD" => Coin::DOT,
        "ETH/USD" => Coin::ETH,
        "SOL/USD" => Coin::SOL,
        _ => {
            tracing::error!("unexpected message coin: {}", message.3);
            return None;
        }
    };

    // Extract last price, best bid, and best ask.
    let last_price = crate::price::str_to_f64(message.1.c.0)?;
    let best_bid = crate::price::str_to_f64(message.1.b.0)?;
    let best_ask = crate::price::str_to_f64(message.1.a.0)?;
    Some((coin, [last_price, best_bid, best_ask]))
}
