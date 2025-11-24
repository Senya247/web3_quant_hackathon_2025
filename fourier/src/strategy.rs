use crate::fourier::{Candle, Position};
use crate::order_engine::OrderWithResponse;
use crate::roostoo::{OrderDetail, OrderSide, OrderType, RoostooClient};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::{mpsc, oneshot};

const MAX_CANDLE_HISTORY: usize = 2048;

pub struct Indicators {}

impl Indicators {}

// THis is context for a single trader
#[derive(Debug)]
pub struct ExecContext {
    pub symbol: String,
    pub candles: Vec<Candle>,
    pub position: Position,

    pub last_close: f64,
    pub last_signal: f64,
    pub precision: u64,
}

impl ExecContext {
    fn update(&mut self, candle: Candle) {
        self.last_close = candle.close;
        self.candles.push(candle);
        if self.candles.len() > MAX_CANDLE_HISTORY {
            let drop_len = self.candles.len() - MAX_CANDLE_HISTORY;
            self.candles.drain(0..drop_len);
        }
        let _ = self.position.update_unrealized(self.last_close);
    }
}

pub struct Order {
    pub pair: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub quantity: f64,
    pub price: Option<f64>,
    // pub response: Option<mpsc::Receiver<PlaceOrderResponse>>
}

#[async_trait]
pub trait Strategy {
    async fn should_long(
        &self,
        ctx: &mut ExecContext,
        shared_state: Arc<Mutex<SharedState>>,
    ) -> bool;
    async fn go_long(
        &self,
        ctx: &ExecContext,
        shared_state: Arc<Mutex<SharedState>>,
    ) -> Option<Order>;
    async fn update_position(
        &self,
        ctx: &ExecContext,
        shared_state: Arc<Mutex<SharedState>>,
    ) -> bool;
}

pub struct CandleData {
    pub symbol: String,
    pub candle: Candle,
}

// This is for hared state of ALL crypto traders
pub struct SharedState {
    pub capital: f64,
    pub streak: u64,
}

pub struct Executioner<T: Strategy + Send> {
    cryptos: HashMap<String, ExecContext>, // crypt -> context
    shared_state: Arc<Mutex<SharedState>>,
    strategy: T,
    order_engine: mpsc::Sender<OrderWithResponse>,
    candle_input: mpsc::Receiver<CandleData>,
    client: RoostooClient,
    bootstrap_positions: HashMap<String, f64>,
}

// a.rs
pub struct TraderConfig<T: Strategy + Send> {
    pub initial_capital: f64,
    pub strategy: T,
    pub candle_data_rx: mpsc::Receiver<CandleData>,
    pub order_engine_tx: mpsc::Sender<OrderWithResponse>,
    pub api_key: String,
    pub api_secret: String,
    pub initial_positions: HashMap<String, f64>,
}

impl<T: Strategy + Send> Executioner<T> {
    pub fn new(config: TraderConfig<T>) -> Self {
        // TODO: read positions from cache
        return Self {
            cryptos: HashMap::new(),
            shared_state: Arc::new(Mutex::new(SharedState {
                capital: config.initial_capital,
                streak: 0,
            })),
            strategy: config.strategy,
            order_engine: config.order_engine_tx,
            candle_input: config.candle_data_rx,
            client: RoostooClient::new(config.api_key, config.api_secret),
            bootstrap_positions: config.initial_positions,
        };
    }

    pub fn add_symbol(&mut self, symbol: String, precision: u64) {
        let v: Vec<Candle> = Vec::new();
        let exectx = ExecContext {
            symbol: symbol.clone(),
            candles: v,
            position: Position::empty(symbol.clone()),
            last_close: 0.0,
            last_signal: 0.0,
            precision: precision,
        };

        self.cryptos.insert(symbol.clone(), exectx);
    }

    pub async fn run(&mut self, backtesting: bool) {
        let mut index: usize = 0;
        while let Some(candle_message) = self.candle_input.recv().await {
            let l = self.cryptos.len();
            let mut ctx = match self.cryptos.remove(&candle_message.symbol) {
                None => continue,
                Some(c) => c,
            };
            ctx.update(candle_message.candle);
            if let Some(qty) = self.bootstrap_positions.remove(&ctx.symbol) {
                if qty > 0.0 && !ctx.position.is_open() && ctx.last_close > 0.0 {
                    if let Err(err) = ctx.position.add_fill(qty, ctx.last_close, 0.0, None) {
                        println!(
                            "[ERROR][BOOTSTRAP] Failed to seed {} with {} units: {}",
                            ctx.symbol, qty, err
                        );
                    } else {
                        println!(
                            "[INFO][BOOTSTRAP] Restored {} with existing position of {} units",
                            ctx.symbol, qty
                        );
                    }
                }
            }
            index += 1;

            let capital: f64;
            {
                let guard = self.shared_state.lock().await;
                capital = guard.capital;
            }
            println!(
                "[{}] Capital: {} Holding: {}",
                ctx.symbol,
                capital,
                ctx.position.quantity * ctx.last_close,
            );

            // just liquidated position for this ctx
            if ctx.position.is_open()
                && self
                    .strategy
                    .update_position(&ctx, self.shared_state.clone())
                    .await
            {
                if backtesting {
                    let price = ctx.last_close;
                    let qty = ctx.position.quantity;
                    let fees = 0.001 * price * qty;
                    if let Err(err) = ctx.position.close_all(price, fees) {
                        println!("[ERROR][BACKTEST] Failed to close position: {}", err);
                    }
                    {
                        let mut guard = self.shared_state.lock().await;
                        guard.capital += qty * price - fees;
                    }
                } else {
                    let order = Order {
                        pair: [ctx.symbol.clone(), "/USD".to_string()].concat(),
                        side: OrderSide::Sell,
                        order_type: OrderType::Market,
                        quantity: ctx.position.quantity,
                        price: None,
                    };
                    let (tx, rx) = oneshot::channel();

                    let orderwithresponse = OrderWithResponse {
                        order: order,
                        precision: ctx.precision,
                        response: tx,
                    };
                    if let Err(e) = self.order_engine.send(orderwithresponse).await {
                        println!("[ERROR][ORDERENGINE] Failed to dispatch close order: {}", e);
                    } else {
                        match rx.await {
                            Ok(order_detail) => {
                                if let Some((qty, price, fee)) = self.sync(Some(order_detail)).await
                                {
                                    if let Err(err) = ctx.position.reduce(qty, price, fee) {
                                        println!("[ERROR][POSITION] Reduce failed: {}", err);
                                    }
                                } else {
                                    println!(
                                        "[ERROR][UPDATEPOSITION] Sync returned no fill details"
                                    );
                                }
                            }
                            Err(e) => {
                                println!("[ERROR][ORDERENGINE] Could not receive fill: {}", e);
                            }
                        }
                    }
                }
            }

            if self
                .strategy
                .should_long(&mut ctx, self.shared_state.clone())
                .await
            {
                if let Some(order) = self.strategy.go_long(&ctx, self.shared_state.clone()).await {
                    if backtesting {
                        let qty = order.quantity;
                        let price = ctx.last_close;
                        let fee = qty * price * 0.001;
                        if let Err(err) = ctx.position.add_fill(qty, price, fee, None) {
                            println!("[ERROR][BACKTEST] Unable to add fill: {}", err);
                        }
                        {
                            let mut guard = self.shared_state.lock().await;
                            guard.capital -= qty * price + fee;
                        }
                    } else {
                        let (tx, rx) = oneshot::channel();
                        let orderwithresponse = OrderWithResponse {
                            order: order,
                            precision: ctx.precision,
                            response: tx,
                        };
                        if let Err(e) = self.order_engine.send(orderwithresponse).await {
                            println!("[ERROR][ORDERENGINE] Failed to dispatch open order: {}", e);
                        } else {
                            // update local stuff
                            match rx.await {
                                // hopefully instant?
                                Ok(order_detail) => match self.sync(Some(order_detail)).await {
                                    Some((qty, price, fee)) => {
                                        if let Err(err) =
                                            ctx.position.add_fill(qty, price, fee, None)
                                        {
                                            println!(
                                                "[ERROR][POSITION] Failed to register fill: {}",
                                                err
                                            );
                                        }
                                    }
                                    None => {
                                        println!(
                                            "[ERROR][UPDATEPOSITION] Sync returned no fill data"
                                        );
                                    }
                                },
                                Err(e) => {
                                    println!(
                                        "Could not receive data from OrderEngine oneshot: {}",
                                        e
                                    );
                                }
                            }
                        }
                    }
                }
            }

            self.cryptos.insert(candle_message.symbol, ctx);

            // periodic wallet sync cause floating point is gay
            if l > 0 && index % (l * 15) == 0 && !backtesting {
                self.sync(None).await;
            }
        }
    }

    // if argument to details None, sync capital. if given order, return qty,price
    // ONLY UPDATES CAPTAL, NOT POSITION
    async fn sync(&self, details: Option<OrderDetail>) -> Option<(f64, f64, f64)> {
        // let guard = self.shared_state.lock().await;
        match details {
            None => {
                match self.client.get_balance().await {
                    Err(e) => {
                        println!("[ERROR][Sync] Could not fetch balance: {}", e);
                    }
                    Ok(balance_info) => {
                        let Some(balance) = balance_info.spot_wallet.get("USD") else {
                            println!("[ERROR][Sync] USD balance missing in wallet snapshot");
                            return None;
                        };
                        let capital_copy = balance.free;
                        {
                            let mut guard = self.shared_state.lock().await;
                            guard.capital = capital_copy;
                        }

                        println!("[SUCCESS][Sync] Successfull. Capital: {}", capital_copy);
                    }
                };
                return None;
            }
            Some(details) => {
                let sign: f64 = match details.side.as_str() {
                    "BUY" => -1.0,
                    "SELL" => 1.0,
                    other => {
                        println!(
                            "[ERROR][SYNC] Got order type that's neither BUY or SELL: {}",
                            other
                        );
                        0.0
                    }
                };
                if sign == 0.0 {
                    return None;
                }

                let capital_copy: f64;
                let qty = details.filled_quantity;
                let price = details.filled_aver_price;
                let fee = details.commission_charge_value;
                {
                    let mut guard = self.shared_state.lock().await;
                    guard.capital += sign * qty * price - fee;
                    capital_copy = guard.capital;
                }
                println!(
                    "[SUCCESS][SYNC] Order {}: sym: {} qty: {} price: {}, capital: {}",
                    if sign > 0.0 { "SELL" } else { "BUY" },
                    details.pair,
                    qty,
                    price,
                    capital_copy
                );

                return Some((qty, price, fee));
            }
        }
    }
}
