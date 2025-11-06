use std::sync::mpsc;

use serde::Deserialize;

#[derive(Debug, Deserialize, Copy, Clone)]
pub struct Candle {
    #[serde(rename = "datetime")]
    pub time: u64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub trade_count: i64,
}

#[derive(Debug, Default)]
pub struct Position {
    qty: f64,       // base asset quantity (BTC)
    avg_price: f64, // quote per base (USDT per BTC)
}

impl Position {
    const FEE_RATE: f64 = 0.001;
    fn total_cost(&self) -> f64 {
        self.qty * self.avg_price
    }

    fn buy(&mut self, q: f64, p: f64) {
        assert!(q > 0.0, "buy quantity must be positive");
        let notional = q * p;
        let fee = notional * Self::FEE_RATE;
        let new_total_cost = self.total_cost() + notional + fee;
        self.qty += q;
        self.avg_price = new_total_cost / self.qty;
    }

    fn sell(&mut self, s: f64, p: f64) -> f64 {
        assert!(s > 0.0, "sell quantity must be positive");
        assert!(s <= self.qty + 1e-12, "selling more than held");
        let notional = s * p;
        let fee = notional * Self::FEE_RATE;
        let gross_realized = s * (p - self.avg_price);
        let realized_after_fee = gross_realized - fee;
        self.qty -= s;
        if self.qty <= 1e-12 {
            self.qty = 0.0;
            self.avg_price = 0.0;
        }
        realized_after_fee
    }

    fn is_open(&self) -> bool {
        self.qty > 0.0
    }

    fn unrealized_pnl(&self, mark_price: f64) -> f64 {
        self.qty * (mark_price - self.avg_price)
    }
    fn liquidate_all(&mut self, price: f64) -> (f64, f64) {
        if self.qty <= 0.0 {
            return (0.0, 0.0);
        }
        let s = self.qty;
        let notional = s * price;
        let fee = notional * Self::FEE_RATE;
        let gross_realized = s * (price - self.avg_price);
        let realized_after_fee = gross_realized - fee;
        let proceeds_after_fee = notional - fee;
        // clear position
        self.qty = 0.0;
        self.avg_price = 0.0;
        (realized_after_fee, proceeds_after_fee)
    }
}

pub struct FourierStrat {
    pub capital: f64, // money
    candles: Vec<Candle>,
    position: Position, // bought at, bought how much
    fees: f64,
}

impl FourierStrat {
    pub fn should_long(&self) -> bool {
        let short_ema = self.ema(3);
        let long_ema = self.ema(5);
        let momentum = self.momentum();

        // println!(
        //     "EMA(3): {:.4}, EMA(6): {:.4}, Momentum: {:.4}%",
        //     short_ema,
        //     long_ema,
        //     momentum * 100.0
        // );

        short_ema > long_ema && momentum > 0.0
    }

    pub fn go_long(&mut self) -> Option<f64> {
        let p = self.price();
        if p == 0.0 || self.capital <= 0.0 {
            // println!("Skipping - price: {:.2}, capital: {:.2}", p, self.capital);
            return None;
        }

        let risk_capital = self.capital * 0.05; // Use only 10%

        if risk_capital < 1.0 {
            // println!("Skipping - risk capital too small: {:.6}", risk_capital);
            return None;
        }

        let qty = (risk_capital / p).round();
        println!(
            "{} Trading - capital: {:.2}, risk: {:.2}, price: {:.2}, qty: {:.8}",
            self.candles.last().unwrap().time,
            self.capital,
            risk_capital,
            p,
            qty
        );

        self.buy(qty);
        println!("BUY {} BTC at {}", qty, self.price());
        return Some(qty);
    }

    // return whether liquidate all or not LMAO
    pub fn update_position(&mut self) -> bool {
        // Exit if we have a 2% loss or 5% gain
        let pnl_percent = self.pnl();

        // println!("Position PnL: {:.2}%", pnl_percent);

        if pnl_percent <= -2.0 || pnl_percent >= 0.5 {
            println!("Closing position due to PnL: {:.2}%", pnl_percent);
            self.liquidiate();
            return true;
        }

        return false;
    }

    pub fn open_position_qty(&self) -> f64 {
        return self.position.qty;
    }

    pub fn total_portfolio_value(&self) -> f64 {
        let position_value = if self.has_open_position() {
            let current_price = self.price();
            self.position.qty * current_price
        } else {
            0.0
        };

        // Total = available capital + current position value
        self.capital + position_value
    }

    pub fn build(capital: f64, fees: f64) -> Self {
        FourierStrat {
            capital,
            candles: Vec::new(),
            position: Position {
                qty: 0.0,
                avg_price: 0.0,
            },
            fees,
        }
    }

    pub fn pnl(&self) -> f64 {
        assert!(self.has_open_position());
        return self.position.unrealized_pnl(self.price()) / self.position.total_cost() * 100.0;
    }

    pub fn liquidiate(&mut self) {
        if self.has_open_position() {
            let price = self.price();
            let (realized_pnl, proceeds) = self.position.liquidate_all(price);
            self.capital += proceeds; // Make sure this line exists!
            println!(
                "Liquidated position. PnL: {:.2}, Capital now: {:.2}",
                realized_pnl, self.capital
            );
        }
    }

    pub fn has_open_position(&self) -> bool {
        return self.position.is_open();
    }

    fn price(&self) -> f64 {
        return self.candles.last().unwrap().close;
    }

    fn buy(&mut self, qty: f64) {
        // Add safety checks
        if qty <= 0.0 {
            println!("ERROR: Attempted to buy non-positive quantity: {:.12}", qty);
            return;
        }

        let price = self.price();
        let cost = qty * price;
        let fee = cost * self.fees;
        let total_cost = cost + fee;

        if total_cost > self.capital {
            println!(
                "ERROR: Insufficient capital. Need: {:.6}, Have: {:.6}",
                total_cost, self.capital
            );
            return;
        }

        self.capital -= total_cost;
        self.position.buy(qty, price);
    }

    fn sell(&mut self, qty: f64) {
        let price = self.price();
        let realized_pnl = self.position.sell(qty, price);
        self.capital += realized_pnl; // Add back the proceeds
    }

    pub fn add_candle(&mut self, candle: Candle) {
        self.candles.push(candle);
    }
    pub fn capital(&self) -> f64 {
        return self.capital;
    }

    pub fn candles(&self) -> &Vec<Candle> {
        return &self.candles;
    }
}

struct StrategyDriverMessage {}

struct StrategyDriver {
    strategy: FourierStrat,
    coms: (
        mpsc::Sender<StrategyDriverMessage>, // sends trade data (buy sell)
        mpsc::Receiver<StrategyDriverMessage>, // receives candle data, and state updates (capital
                                             // etc)
    ),
}
