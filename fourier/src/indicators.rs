use crate::fourier::FourierStrat;

impl FourierStrat {
    // Simple Moving Average
    pub fn sma(&self, period: usize) -> f64 {
        if self.candles().len() < period {
            return 0.0;
        }
        let sum: f64 = self.candles().iter().rev().take(period).map(|c| c.close).sum();
        sum / (period as f64)
    }

    // Exponential Moving Average  
    pub fn ema(&self, period: usize) -> f64 {
        if self.candles().len() < period {
            return 0.0;
        }
        let alpha = 2.0 / (period as f64 + 1.0);
        let mut ema = self.candles()[self.candles().len() - period].close;
        
        for i in (self.candles().len() - period + 1)..self.candles().len() {
            ema = alpha * self.candles()[i].close + (1.0 - alpha) * ema;
        }
        ema
    }

    // Price momentum (current close vs previous close)
    pub fn momentum(&self) -> f64 {
        if self.candles().len() < 2 {
            return 0.0;
        }
        let current = self.candles().last().unwrap().close;
        let prev = self.candles()[self.candles().len() - 2].close;
        (current - prev) / prev
    }
}
