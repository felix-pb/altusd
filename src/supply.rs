use crate::engine::Input;
use altusd::Coin;
use serde::Deserialize;
use std::time::Duration;
use tokio::sync::mpsc::Sender;

/// The API endpoint from which we retrieve the circulating supplies.
const ENDPOINT: &str = "https://www.coinbase.com/api/v2/assets/search";

/// The interval at which we poll the API endpoint.
const POLL_INTERVAL: Duration = Duration::from_secs(60);

/// This struct represents the top-level JSON object returned by the API endpoint.
/// Irrelevant fields are omitted.
#[derive(Deserialize)]
struct ApiResponse {
    data: Vec<ApiResponseData>,
}

/// This struct represents the JSON object of an element in the `data` array of the API response.
/// Irrelevant fields are omitted.
#[derive(Deserialize)]
struct ApiResponseData {
    symbol: String,
    circulating_supply: String,
}

/// This function is responsible for feeding the current circulating supply of our index's
/// altcoins to the core engine. It does that by polling a Coinbase API endpoint every minute.
pub async fn run(mpsc_tx: Sender<Input>) {
    loop {
        poll_api_endpoint(&mpsc_tx).await;
        tokio::time::sleep(POLL_INTERVAL).await;
    }
}

/// This function is responsible for polling the API endpoint once and for trying to extract
/// the circulating supplies of all the altcoins in the index from the response.
async fn poll_api_endpoint(mpsc_tx: &Sender<Input>) {
    // Send a GET request.
    let response = match reqwest::get(ENDPOINT).await {
        Ok(response) => response,
        Err(error) => {
            tracing::error!("failed to poll api endpoint: {}", error);
            return;
        }
    };

    // Download the response body and try to parse it into an `ApiResponse`.
    let response = match response.json::<ApiResponse>().await {
        Ok(response) => response,
        Err(error) => {
            tracing::error!("failed to parse api response: {}", error);
            return;
        }
    };

    // Try to extract the circulating supply of each coin.
    try_extract_for_coin(mpsc_tx, &response, Coin::ADA).await;
    try_extract_for_coin(mpsc_tx, &response, Coin::DOGE).await;
    try_extract_for_coin(mpsc_tx, &response, Coin::DOT).await;
    try_extract_for_coin(mpsc_tx, &response, Coin::ETH).await;
    try_extract_for_coin(mpsc_tx, &response, Coin::SOL).await;
}

/// This function is responsible for extracting the circulating supply of a given coin from the
/// API response and sending it to the core engine.
async fn try_extract_for_coin(mpsc_tx: &Sender<Input>, response: &ApiResponse, coin: Coin) {
    let target_symbol = match coin {
        Coin::ADA => "ADA",
        Coin::DOGE => "DOGE",
        Coin::DOT => "DOT",
        Coin::ETH => "ETH",
        Coin::SOL => "SOL",
    };

    // Try to find the entry in the `data` array.
    let entry = match response
        .data
        .iter()
        .find(|entry| entry.symbol == target_symbol)
    {
        Some(entry) => entry,
        None => {
            tracing::error!("failed to find target symbol: {}", target_symbol);
            return;
        }
    };

    // Try to parse the circulating supply as a 64-bit floating-point value.
    let supply = match entry.circulating_supply.parse::<f64>() {
        Ok(supply) => supply,
        Err(error) => {
            tracing::error!("failed to parse circulating supply: {}", error);
            return;
        }
    };

    // Send the circulating supply to the core engine.
    let input = Input::supply(coin, supply);
    if let Err(error) = mpsc_tx.send(input).await {
        tracing::error!("failed to send message in mpsc channel: {}", error);
    }
}
