use altusd::{Coin, Engine, Exchange};
use serde::Serialize;
use std::time::SystemTime;
use tokio::sync::mpsc::Receiver;
use tokio::sync::watch::Sender;

/// This struct represents the input of the core engine, which is received through a mpsc channel.
#[derive(Debug)]
pub enum Input {
    Price(Coin, Exchange, f64),
    Supply(Coin, f64),
}

impl Input {
    /// Constructor for the `Price` variant.
    pub fn price(coin: Coin, exchange: Exchange, price: f64) -> Self {
        Self::Price(coin, exchange, price)
    }

    /// Constructor for the `Supply` variant.
    pub fn supply(coin: Coin, supply: f64) -> Self {
        Self::Supply(coin, supply)
    }
}

/// This struct represents the output of the core engine, which is sent through a watch channel.
/// Note that `f64::NAN` deserializes to null in JSON.
#[derive(Debug, Serialize)]
pub struct Output {
    pub epoch: u64,
    pub index: f64,
}

impl Output {
    /// Default contructor. The index is set to NaN.
    pub fn init() -> Self {
        Self {
            epoch: 0,
            index: f64::NAN,
        }
    }
}

/// This function is responsible for running the core index engine.
/// It's a thin wrapper around the library to receive input and send output from channels.
/// However, it's the library itself that contains the core business logic of the engine.
pub async fn run(mut mpsc_rx: Receiver<Input>, watch_tx: Sender<Output>) {
    let mut engine = Engine::init();

    // Wait for input messages from the mpsc channel in a loop...
    while let Some(input) = mpsc_rx.recv().await {
        tracing::info!("input message = {:?}", input);

        // Process the input message (i.e. updated price or supply) in the engine.
        let index = match input {
            Input::Price(coin, exchange, price) => engine.update_price(coin, exchange, price),
            Input::Supply(coin, supply) => engine.update_supply(coin, supply),
        };

        // Timestamp the updated index price with Unix time, i.e. the number of seconds
        // that have elapsed since 00:00:00 UTC on 1 January 1970.
        let epoch = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(duration) => duration.as_secs(),
            Err(error) => {
                tracing::error!("failed to get system time: {}", error);
                continue;
            }
        };

        // Send the output message (i.e. updated index price with timestamp) on the watch channel.
        let output = Output { epoch, index };
        tracing::info!("output message = {:?}", output);
        if let Err(error) = watch_tx.send(output) {
            tracing::error!("failed to send message in watch channel: {}", error);
        }
    }
}
