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
    let mut reader = csv::Reader::from_path(csv_file)?;

    let warmup_candles: i64 = 12;
    let mut num_candle: i64 = 0;
    
    for row in reader.deserialize().skip(10) {
        let candle: Candle = row?;
        self.strategy.add_candle(candle);
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

    Ok(self.strategy.capital())
}
}
