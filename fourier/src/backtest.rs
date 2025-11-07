use crate::fourier::{Candle, UnusedPieceofShit};
use anyhow::Result;
use plotly::common::Mode;
use plotly::{Plot, Scatter};

pub struct BackTester {
    strategy: UnusedPieceofShit,
}

impl BackTester {
    pub fn create(strategy: UnusedPieceofShit) -> Self {
        BackTester { strategy }
    }
    // take &mut self so we can call &mut methods on the strategy
    pub fn begin(&mut self, csv_file: &str) -> Result<f64> {
        let mut reader = csv::Reader::from_path(csv_file)?;

        let warmup_candles: i64 = 15;
        let mut num_candle: i64 = 0;

        let mut index: Vec<i64> = Vec::new();
        let mut btc_price: Vec<f64> = Vec::new();
        let mut capital: Vec<f64> = Vec::new();

        for row in reader.deserialize() {
            let candle: Candle = row?;
            self.strategy.add_candle(candle);
            index.push(num_candle);
            btc_price.push(candle.close);
            capital.push(self.strategy.total_portfolio_value());
            num_candle += 1;

            if num_candle < warmup_candles {
                continue;
            }

            // Update existing position (might close it)
            if self.strategy.has_open_position() {
                self.strategy.update_position();
            }

            // Check for new long opportunities (can open new or add to existing)
            if self.strategy.should_long() {
                self.strategy.go_long();
            }
        }

        // Optional: Close any remaining position at the end
        if self.strategy.has_open_position() {
            self.strategy.liquidiate();
        }

        let mut plot = Plot::new();
        plot.add_trace(Scatter::new(index.clone(), btc_price).name("BTC"));
        plot.add_trace(Scatter::new(index, capital).name("Portfolio"));
        plot.show();
        Ok(self.strategy.capital())
    }
}
