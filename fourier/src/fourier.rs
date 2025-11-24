use crate::{
    indicators::Indicators,
    roostoo::{OrderSide, OrderType},
    strategy::{ExecContext, Order, SharedState, Strategy},
};
use anyhow::{Context, Result};
use async_trait::async_trait;

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::sync::Mutex;

#[derive(Debug, Deserialize, Copy, Clone, Default)]
pub struct Candle {
    #[serde(rename = "datetime")]
    pub open_time: u64,
    pub close_time: u64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub trade_count: i64,
}

/// Helper to get current unix epoch seconds
fn now_unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Simple long-only position representation.
/// Uses `i64` unix epoch seconds for times and `f64` for prices/quantities for simplicity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub symbol: String,
    pub quantity: f64,           // base units, always >= 0.0
    pub entry_price: f64,        // VWAP entry price in quote currency
    pub entry_time: Option<u64>, // unix epoch seconds when position was opened (None if closed)
    pub realized_pnl: f64,       // cumulative realized PnL in quote currency
    pub unrealized_pnl: f64,     // last computed unrealized PnL in quote currency
    pub avg_fee_per_unit: f64,   // average fee in quote currency per base unit
}

impl Position {
    /// Create an empty (closed) position for a symbol.
    pub fn empty<S: Into<String>>(symbol: S) -> Self {
        Position {
            symbol: symbol.into(),
            quantity: 0.0,
            entry_price: 0.0,
            entry_time: None,
            realized_pnl: 0.0,
            unrealized_pnl: 0.0,
            avg_fee_per_unit: 0.0,
        }
    }
    pub fn save_to_yaml<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();
        let tmp = path.with_extension("yaml.tmp");

        let s = serde_yaml::to_string(self).context("serialize position to YAML")?;
        fs::write(&tmp, s).with_context(|| format!("write temp file {:?}", tmp))?;
        fs::rename(&tmp, &path).with_context(|| format!("rename {:?} -> {:?}", tmp, path))?;
        Ok(())
    }

    /// Load position from YAML file at `path`.
    pub fn load_from_yaml<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let data = fs::read_to_string(path).with_context(|| format!("read file {:?}", path))?;
        let p: Position = serde_yaml::from_str(&data).context("parse YAML into Position")?;
        Ok(p)
    }
    /// Is there an open position
    pub fn is_open(&self) -> bool {
        self.quantity > 0.0
    }

    /// Notional (quote) value at given mark price
    pub fn notional(&self, mark_price: f64) -> f64 {
        self.quantity * mark_price
    }

    /// Update and return unrealized PnL using provided mark price.
    /// unrealized_pnl = (mark - entry) * qty - fee_cost
    pub fn update_unrealized(&mut self, mark_price: f64) -> f64 {
        if !self.is_open() {
            self.unrealized_pnl = 0.0;
            return 0.0;
        }
        let gross = (mark_price - self.entry_price) * self.quantity;
        let fee_cost = self.avg_fee_per_unit * self.quantity;
        self.unrealized_pnl = gross - fee_cost;
        self.unrealized_pnl
    }
    pub fn unrealized_pct(&self, mark_price: f64) -> Option<f64> {
        if !self.is_open() || self.entry_price == 0.0 {
            return None;
        }
        let pct = (mark_price - self.entry_price) / self.entry_price * 100.0;
        Some(pct)
    }

    /// Open a new long position when currently closed.
    /// `fee` is total fee paid in quote currency for this fill.
    pub fn open_new(
        &mut self,
        qty: f64,
        price: f64,
        fee: f64,
        time_unix_secs: Option<u64>,
    ) -> Result<(), &'static str> {
        if qty <= 0.0 {
            return Err("qty must be positive");
        }
        if self.is_open() {
            return Err("position already open");
        }

        self.quantity = qty;
        self.entry_price = price;
        self.entry_time = Some(time_unix_secs.unwrap_or_else(now_unix_secs));
        self.avg_fee_per_unit = if qty != 0.0 { fee / qty } else { 0.0 };
        self.unrealized_pnl = 0.0;
        Ok(())
    }

    /// Add to an existing long position (same-side add).
    /// If closed, behaves like `open_new`.
    pub fn add_fill(
        &mut self,
        qty: f64,
        price: f64,
        fee: f64,
        time_unix_secs: Option<u64>,
    ) -> Result<(), &'static str> {
        if qty <= 0.0 {
            return Err("qty must be positive");
        }

        if !self.is_open() {
            return self.open_new(qty, price, fee, time_unix_secs);
        }

        // update VWAP entry price and average fee per unit
        let total_qty = self.quantity + qty;
        let new_vwap = (self.entry_price * self.quantity + price * qty) / total_qty;
        let new_avg_fee = (self.avg_fee_per_unit * self.quantity + fee) / total_qty;

        self.entry_price = new_vwap;
        self.avg_fee_per_unit = new_avg_fee;
        self.quantity = total_qty;
        if self.entry_time.is_none() {
            self.entry_time = Some(time_unix_secs.unwrap_or_else(now_unix_secs));
        }
        Ok(())
    }

    /// Reduce (sell) up to `qty` units at `price`. Returns realized pnl for the closed portion.
    /// If qty >= current quantity the position is fully closed.
    /// `fee` is the explicit fee paid for this reduction (quote currency).
    pub fn reduce(&mut self, qty: f64, price: f64, fee: f64) -> Result<f64, &'static str> {
        if qty <= 0.0 {
            return Err("qty must be positive");
        }
        if !self.is_open() {
            return Err("no open position to reduce");
        }

        let closed_qty = qty.min(self.quantity);
        let gross = (price - self.entry_price) * closed_qty;
        // approximate realized fee: proportion of avg_fee_per_unit plus explicit fee
        let realized_fee = self.avg_fee_per_unit * closed_qty + fee;
        let realized = gross - realized_fee;
        self.realized_pnl += realized;

        self.quantity -= closed_qty;

        if self.quantity == 0.0 {
            // reset
            self.entry_price = 0.0;
            self.entry_time = None;
            self.avg_fee_per_unit = 0.0;
            self.unrealized_pnl = 0.0;
        }

        Ok(realized)
    }

    /// Close entire position at given price, returning realized pnl.
    /// `fee` is explicit fee for the closing trade.
    pub fn close_all(&mut self, price: f64, fee: f64) -> Result<f64, &'static str> {
        if !self.is_open() {
            return Err("no open position");
        }
        let qty = self.quantity;
        let gross = (price - self.entry_price) * qty;
        let total_fee = self.avg_fee_per_unit * qty + fee;
        let realized = gross - total_fee;
        self.realized_pnl += realized;

        // clear
        self.quantity = 0.0;
        self.entry_price = 0.0;
        self.entry_time = None;
        self.avg_fee_per_unit = 0.0;
        self.unrealized_pnl = 0.0;

        Ok(realized)
    }
}

pub struct Fourier {}

#[async_trait]
impl Strategy for Fourier {
    async fn should_long(
        &self,
        ctx: &mut ExecContext,
        shared_state: Arc<Mutex<SharedState>>,
    ) -> bool {
        if ctx.position.is_open() {
            return false;
        }

        if ctx.candles.len() < 32 {
            return false;
        }

        let indicators = Indicators::new(&ctx.candles);
        let short = indicators.ema(12);
        let long = indicators.ema(26);
        let rsi = indicators.rsi(14);
        let has_capital = {
            let guard = shared_state.lock().await;
            guard.capital > 0.0
        };
        if !has_capital {
            return false;
        }

        match (short, long, rsi) {
            (Some(short), Some(long), Some(rsi)) => short > long && rsi < 60.0,
            _ => false,
        }
    }

    async fn go_long(
        &self,
        ctx: &ExecContext,
        shared_state: Arc<Mutex<SharedState>>,
    ) -> Option<Order> {
        let indicators = Indicators::new(&ctx.candles);
        let atr = indicators.atr(14).unwrap_or(ctx.last_close * 0.01);

        let risk_capital = {
            let guard = shared_state.lock().await;
            (guard.capital * 0.02).max(0.0)
        };
        if risk_capital == 0.0 || ctx.last_close == 0.0 {
            return None;
        }

        let per_unit_risk = atr.max(1e-6);
        let position_size = (risk_capital / per_unit_risk).min(guarded_max_size(ctx.last_close));
        if position_size <= 0.0 {
            return None;
        }

        let order: Order = Order {
            pair: [ctx.symbol.clone(), "/USD".to_string()].concat(),
            side: OrderSide::Buy,
            order_type: OrderType::Market,
            quantity: position_size,
            price: None,
        };

        return Some(order);
    }

    // #TODO more options other than just liquidate all
    async fn update_position(
        &self,
        ctx: &ExecContext,
        _shared_state: Arc<Mutex<SharedState>>,
    ) -> bool {
        if !ctx.position.is_open() {
            return false;
        }

        let pct = ctx.position.unrealized_pct(ctx.last_close);
        let target = match pct {
            Some(p) => p,
            None => return false,
        };

        let stop_loss = -2.0;
        let take_profit = 4.0;

        if target <= stop_loss || target >= take_profit {
            return true;
        }

        false
    }
}

fn guarded_max_size(price: f64) -> f64 {
    if price <= 0.0 {
        return 0.0;
    }
    10_000.0 / price
}
