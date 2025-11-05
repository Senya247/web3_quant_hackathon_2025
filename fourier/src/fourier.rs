use serde::Deserialize;

#[derive(Debug, Deserialize, Copy, Clone)]
pub struct Candle {
    #[serde(rename = "datetime")]
    pub time: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub trade_count: i64,
}

pub struct FourierStrat {
    capital: f64, // money
    candles: Vec<Candle>,
    open_position: (f64, f64), // bought at, bought how much
    fees: f64,
}

impl FourierStrat {
    pub fn should_long(&self) -> bool {
        if self.sma(3) > self.sma(5) {
            return true;
            println!("SHOULD LONG");
        };
        return false;
    }

    pub fn update_position(&mut self) {
        if self.pnl().abs() <= 0.5 {
            self.liquidiate();
        }
    }

    pub fn create(capital: f64, fees: f64) -> Self {
        FourierStrat {
            capital,
            candles: Vec::new(),
            open_position: (0.0, 0.0),
            fees,
        }
    }

    pub fn pnl(&self) -> f64 {
        assert!(self.has_open_position());
        return (self.price() - self.open_position.0) / self.open_position.0;
    }

    pub fn liquidiate(&mut self) {
        assert!(self.has_open_position()); // only if there is an open position
        self.sell(self.open_position.1);
    }

    pub fn has_open_position(&self) -> bool {
        return self.open_position != (0.0, 0.0);
    }

    fn price(&self) -> f64 {
        return self.candles.last().unwrap().close;
    }

    fn buy(&mut self, qty: f64) {
        self.capital -= qty * self.price() * (1.0 - self.fees); // FEES, also no limit orders

        // self.open_position = (self.price(), );
    }

    fn sell(&mut self, qty: f64) {
        assert!(self.has_open_position());
        assert!(qty <= self.open_position.1);
        self.capital += qty * self.price() * (1.0 - self.fees);
        self.holding -= qty;

        self.open_position.1 -= qty;
    }

    pub fn add_candle(&mut self, candle: Candle) {
        self.candles.push(candle);
    }
    pub fn capital(&self) -> f64 {
        return self.capital;
    }

    pub fn go_long(&mut self) {
        let p = self.price();
        if p == 0.0 {
            return; // avoid division by zero / nonsensical buy
        }
        let qty = (self.capital / 2.0) / p;
        self.buy(qty);
    }

    pub fn candles(&self) -> &Vec<Candle> {
        return &self.candles;
    }
}
