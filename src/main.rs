mod binance;
mod coinbase;
mod engine;
mod kraken;
mod price;
mod server;
mod supply;

use engine::Output;

#[tokio::main]
async fn main() {
    // Initialize the tracing subscriber with default settings.
    tracing_subscriber::fmt::init();

    // This mpsc channel is used to send altcoin prices and supplies to the core index engine.
    let (mpsc_tx, mpsc_rx) = tokio::sync::mpsc::channel(100_000);

    // This watch channel is used to notify changes in the index to the connected websocket clients.
    let (watch_tx, watch_rx) = tokio::sync::watch::channel(Output::init());

    // These tasks are responsible for feeding the current price of our index's altcoins.
    // Each task is responsible for one particular exchange: Binance, Coinbase, or Kraken.
    tokio::spawn(binance::run(mpsc_tx.clone()));
    tokio::spawn(coinbase::run(mpsc_tx.clone()));
    tokio::spawn(kraken::run(mpsc_tx.clone()));

    // This task is responsible for feeding the current circulating supply of our index's altcoins.
    tokio::spawn(supply::run(mpsc_tx));

    // This task is responsible for running the core index engine.
    tokio::spawn(engine::run(mpsc_rx, watch_tx));

    // The current task is responsible for serving our index's price stream over websockets.
    // Internally, it spawns a new tokio task for each connected websocket client.
    server::run(watch_rx).await;
}
