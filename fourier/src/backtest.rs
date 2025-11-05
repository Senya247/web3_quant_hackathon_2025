use crate::fourier::{Candle, FourierStrat};
use anyhow::Result;

pub struct BackTester {
    strategy: FourierStrat,
}

impl BackTester {
    pub fn create(strategy: FourierStrat) -> Self {
        BackTester { strategy }
    }
    // take &mut self so we can call &mut methods on the strategy
    pub fn begin(&mut self, csv_file: &str) -> Result<f64> {
        // capital at end
        let mut reader = csv::Reader::from_path(csv_file)?;

        let warmup_candles: i64 = 12;
        let mut num_candle: i64 = 0;
        for row in reader.deserialize().skip(10) {
            let candle: Candle = row?;

            // add_candle takes ownership of the Candle (or change to &Candle if preferred)
            self.strategy.add_candle(candle);
            num_candle += 1;

            if num_candle < warmup_candles {
                continue;
            }

            if self.strategy.has_open_position() {
                self.strategy.update_position();
                if self.strategy.has_open_position() {
                    continue;
                }
            }
            if self.strategy.should_long() {
                self.strategy.go_long();
            }
        }

        Ok(self.strategy.capital())
    }
}
