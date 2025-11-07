use crate::roostoo::{OrderSide, OrderType};
use crate::{fourier::Candle, roostoo::RoostooClient};
use plotly::common::Position;
use std::collections::HashMap;
use std::hash::Hash;
use tokio::sync::mpsc;

pub struct Indicators {}

impl Indicators {}

// THis is context for a single trader
pub struct Context {
    pub symbol: String,
    pub candles: Vec<Candle>,
    pub position: Position,
}

impl Context {
    fn update() {}
}

pub struct Order {
    pub pair: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub quantity: f64,
    pub price: Option<f64>,
}

pub trait Strategy {
    fn should_long(&self, ctx: &Context, shared_state: &SharedState) -> bool;
    fn go_long(&self, ctx: &Context, shared_state: &SharedState) -> Option<Order>;
    fn update_position(&self, ctx: &Context, shared_state: &SharedState);
}

pub struct CandleData {
    pub symbol: String,
    pub candle: Candle,
}

// This is for hared state of ALL crypto traders
pub struct SharedState {
    pub capital: f64,
}

pub struct Executioner<T: Strategy + Send> {
    cryptos: HashMap<String, Context>, // crypt -> context
    shared_state: SharedState,
    strategy: T,
    order_engine: mpsc::Sender<Order>,
    candle_input: mpsc::Receiver<CandleData>,
}

// a.rs
pub struct TraderConfig<T: Strategy + Send> {
    pub initial_capital: f64,
    pub strategy: T,
    pub candle_data_rx: mpsc::Receiver<CandleData>,
    pub order_engine_tx: mpsc::Sender<Order>,
    pub api_key: String,
    pub api_secret: String,
}

impl<T: Strategy + Send> Executioner<T> {
    pub fn new(config: TraderConfig<T>) -> Self {
        return Self {
            cryptos: HashMap::new(),
            shared_state: SharedState {
                capital: config.initial_capital,
            },
            strategy: config.strategy,
            order_engine: config.order_engine_tx,
            candle_input: config.candle_data_rx,
        };
    }

    pub async fn run(&mut self) {
        while let Some(candle_message) = self.candle_input.recv().await {
            let ctx = match self.cryptos.get(&candle_message.symbol) {
                None => continue,
                Some(c) => c,
            };

            self.strategy.update_position(ctx, &self.shared_state);
            if self.strategy.should_long(ctx, &self.shared_state) {
                if let Some(order) = self.strategy.go_long(ctx, &self.shared_state) {
                    self.order_engine.send(order).await; // #TODO update local cache of capital
                }
            }
        }
    }
}
