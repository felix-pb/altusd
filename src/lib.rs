/// This enum contains the 5 altcoins needed to compute the index.
#[derive(Clone, Copy, Debug)]
pub enum Coin {
    ADA,
    DOGE,
    DOT,
    ETH,
    SOL,
}

/// This enum contains the 3 exchanges needed to compute the index.
/// The median of the market prices on these exchanges is taken.
#[derive(Clone, Copy, Debug)]
pub enum Exchange {
    Binance,
    Coinbase,
    Kraken,
}

/// This struct represents the "core engine" of the altcoin index and encapsulates all the
/// business logic needed to calculate and update it over time.
pub struct Engine {
    ada: Cache,
    doge: Cache,
    dot: Cache,
    eth: Cache,
    sol: Cache,
}

/// This struct is an internal data structure of the `Engine`, and thus a private implementation
/// detail. It caches the values needed by the index for a particular altcoin.
struct Cache {
    circulating_supply: f64,
    market_cap: f64,
    median_price: f64,
    price_binance: f64,
    price_coinbase: f64,
    price_kraken: f64,
}

impl Cache {
    /// Default contructor. All values are set to NaN until they get updated.
    fn init() -> Self {
        Self {
            circulating_supply: f64::NAN,
            market_cap: f64::NAN,
            median_price: f64::NAN,
            price_binance: f64::NAN,
            price_coinbase: f64::NAN,
            price_kraken: f64::NAN,
        }
    }

    /// Update the market capitalize of this altcoin.
    fn update_market_cap(&mut self) {
        self.market_cap = self.circulating_supply * self.median_price;
    }

    /// Update the current median price of this altcoin.
    ///
    /// The market price on all exchanges must be set.
    /// If an exchange price hasn't been set, the current median price stays NaN.
    fn update_median_price(&mut self) {
        let mut prices = [self.price_binance, self.price_coinbase, self.price_kraken];
        if prices.iter().all(|price| price.is_finite()) {
            // Safe unwrap: our slice doesn't contain a NaN. See this link for reference.
            // https://doc.rust-lang.org/std/primitive.slice.html#method.sort_unstable_by
            prices.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
            self.median_price = prices[1];
        }
    }
}

/// This divisor is used to normalize the index price.
/// Effectively, the index price is the total market cap of the 5 altcoins in billions of USD.
const DIVISOR: f64 = 1_000_000_000.0;

impl Engine {
    /// Default contructor. All values are set to NaN until they get updated.
    pub fn init() -> Self {
        Self {
            ada: Cache::init(),
            doge: Cache::init(),
            dot: Cache::init(),
            eth: Cache::init(),
            sol: Cache::init(),
        }
    }

    /// Get the current index price.
    pub fn get_index(&self) -> f64 {
        (self.ada.market_cap
            + self.doge.market_cap
            + self.dot.market_cap
            + self.eth.market_cap
            + self.sol.market_cap)
            / DIVISOR
    }

    /// Update the current price of an altcoin in the index for a particular exchange.
    pub fn update_price(&mut self, coin: Coin, exchange: Exchange, price: f64) -> f64 {
        let cache = self.get_mut_cache(coin);
        match exchange {
            Exchange::Binance => cache.price_binance = price,
            Exchange::Coinbase => cache.price_coinbase = price,
            Exchange::Kraken => cache.price_kraken = price,
        };
        cache.update_median_price();
        cache.update_market_cap();
        self.get_index()
    }

    /// Update the current circulating supply of an altcoin in the index.
    pub fn update_supply(&mut self, coin: Coin, supply: f64) -> f64 {
        let cache = self.get_mut_cache(coin);
        cache.circulating_supply = supply;
        cache.update_market_cap();
        self.get_index()
    }

    /// Get a mutable reference to the cache for a given altcoin.
    fn get_mut_cache(&mut self, coin: Coin) -> &mut Cache {
        match coin {
            Coin::ADA => &mut self.ada,
            Coin::DOGE => &mut self.doge,
            Coin::DOT => &mut self.dot,
            Coin::ETH => &mut self.eth,
            Coin::SOL => &mut self.sol,
        }
    }
}
