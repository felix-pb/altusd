# ALT/USD

The ALT/USD index, tracking the performance of the top 5 altcoins.

## Requirements

This project is built with:

- Docker (latest): https://docs.docker.com/get-docker
- Rust (1.58.1): https://www.rust-lang.org/tools/install

However, to ensure a consistent developer experience, it's maintained such that
only the latest version of Docker is required to build and run the app.

## How to build and run?

1. Build the app as a docker image.
```
docker build -t altusd .
```

2. Run the app as a docker container. The websocket server listens on port 8080.
```
docker run -it --init --name altusd --network host --rm altusd
```

3. Connect one or more websocket clients.
```
docker run -it --init --network host --rm altusd client
```

## How is the index calculated?

The ALT/USD index is based on the following 5 altcoins:

- ADA (Cardano)
- DOGE (Dogecoin)
- DOT (Polkadot)
- ETH (Ethereum)
- SOL (Solana)

In this case, the choice of altcoins is arbitrary. However, the code is written
such that it would be easy to change or expand this list.

The index price is calculated with the following formula:
```
index price = Σ(current price * current circulating supply) / 1,000,000,000
```

In short, it's the total market cap of the 5 altcoins in billions of USD.

I chose this formula because market-capitalization-weighted indexes (e.g. S&P 500)
are generally regarded as better gauges than price-weighted indexes (e.g. DJIA).

For each altcoin, the "current price" is determined by the median of the
"market price" on the following 3 exchanges:

- Binance: `wss://stream.binance.com:9443/ws`
- Coinbase: `wss://ws-feed.exchange.coinbase.com`
- Kraken: `wss://ws.kraken.com`

Moreover, for each exchange, the "market price" is determined by the median of
the last price, best bid, and best ask. This is the same methodology used by
[FTX][1].

Finally, for each altcoin, the "current circulating supply" is determined by
polling this [Coinbase API endpoint][2]. However, this is not an officially
supported endpoint in their [documentation][3]. I chose it because it was
quick for this prototype, but there are alternatives such as [CoinMarketCap][4].
The ideal solution would be to run a node and get the current circulating supply
from the blockchain itself, but this would be overkill for a simple project.

## Architecture

Although `altusd` is a single process, it's conceptually separated in 3 layers:

- Input: this layer is responsible for retrieving the current price and
circulating supply of the altcoins, and for feeding them to the engine layer.
- Engine: this layer is responsible for calculating and updating the index
price, and for feeding it to the output layer.
- Output: this layer is responsible for serving the index price stream over
websockets.

This architecture makes it easy to work on each layer separately. For example,
the websocket server currently sends a message to the connected clients every
time the index price is updated. That said, we could batch the updates in a
time period (e.g. 1 second) by working on the output layer only.

Here's what the architecture looks like in a diagram:

![architecture](/doc/altusd.drawio.png)

## Further Improvements

There's a number of things that could be improved:

- As mentioned above, we could retrieve the current circulating supply from a
more authoritative source (ideally from the blockchain itself).
- Certain exchanges (e.g. Coinbase) send a sequence number for every message to
counteract out-of-order delivery. Right now, we ignore it but ideally we should
cache the highest sequence number and discard older messages.
- Better logging! Ideally, we should create more structured log messages and we
should make it configurable. That said, tracing-subscriber sets up well for that.
- Better testing! Right now, there are no tests except for the simple websocket
client. All the core business logic for the engine is in the library (`lib.rs`),
so that's where I would start adding some tests.

[1]: https://help.ftx.com/hc/en-us/articles/360027668812-Index-Calculation
[2]: https://www.coinbase.com/api/v2/assets/search
[3]: https://docs.cloud.coinbase.com
[4]: https://coinmarketcap.com/api
