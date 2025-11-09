use std::thread::sleep;
use std::time::Duration;

use crate::fourier::{Candle, Fourier};
use crate::order_engine::OrderEngine;
use crate::strategy::{CandleData, Executioner, Strategy, TraderConfig};
use anyhow::Result;
use plotly::{Plot, Scatter};
use tokio::sync::mpsc;

pub struct BackTester<T> {
    strategy: T,
}

impl<T: Strategy> BackTester<T> {
    pub fn create(strategy: T) -> Self {
        BackTester { strategy }
    }
    // take &mut self so we can call &mut methods on the strategy
    pub async fn begin(&mut self, csv_file: &str) -> Result<f64> {
        let mut reader = csv::Reader::from_path(csv_file)?;

        let warmup_candles: i64 = 15;
        let mut num_candle: i64 = 0;

        let mut index: Vec<i64> = Vec::new();
        let mut btc_price: Vec<f64> = Vec::new();
        let mut capital: Vec<f64> = Vec::new();

        let (candle_tx, candle_rx) = mpsc::channel(32);
        let (oe_tx, oe_rx) = mpsc::channel(32);

        // let _t = tokio::spawn(async move {
        // let mut order_engine = OrderEngine::build("SEX".into(), "SEX".into());
        // order_engine.run(oe_rx).await;
        // });

        let strategy = Fourier {};

        let config = TraderConfig {
            initial_capital: 100000.0,
            strategy: strategy,
            candle_data_rx: candle_rx,
            order_engine_tx: oe_tx,
            api_key: "SEX".to_string(),
            api_secret: "SEX".to_string(),
        };

        let _t = tokio::spawn(async move {
            let mut executioner = Executioner::new(config);
            executioner.add_symbol("DOGE".into(), 0);
            executioner.run(true).await;
        });

        for row in reader.deserialize() {
            let candle: Candle = row?;
            let candle_data = CandleData {
                symbol: "DOGE".to_string(),
                candle: candle,
            };

            let _ = candle_tx.send(candle_data).await;

            sleep(Duration::new(0, 100));
        }

        // Optional: Close any remaining position at the end
        // if self.strategy.has_open_position() {
        //     self.strategy.liquidiate();
        // }

        // let mut plot = Plot::new();
        // plot.add_trace(Scatter::new(index.clone(), btc_price).name("BTC"));
        // plot.add_trace(Scatter::new(index, capital).name("Portfolio"));
        // plot.show();
        // Ok(self.strategy.capital())
        Ok(0.0)
    }
}
