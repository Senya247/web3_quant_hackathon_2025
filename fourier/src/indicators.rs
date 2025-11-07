use crate::fourier::{Candle, UnusedPieceofShit};

pub struct Indicators<'a> {
    candles: &'a [Candle],
}

impl<'a> Indicators<'a> {
    pub fn new(candles: &'a [Candle]) -> Self {
        Self { candles }
    }

    pub fn ema(&self, period: usize) -> Option<f64> {
        if self.candles.is_empty() || period == 0 {
            return None;
        }

        let alpha = 2.0 / (period as f64 + 1.0);
        let mut ema = self.candles[0].close;

        for candle in self.candles.iter().skip(1) {
            ema = alpha * candle.close + (1.0 - alpha) * ema;
        }

        Some(ema)
    }

    pub fn sma(&self, period: usize) -> Option<f64> {
        if self.candles.len() < period {
            return None;
        }

        let sum: f64 = self
            .candles
            .iter()
            .rev()
            .take(period)
            .map(|c| c.close)
            .sum();

        Some(sum / period as f64)
    }

    pub fn rsi(&self, period: usize) -> Option<f64> {
        if self.candles.len() <= period {
            return None;
        }

        let mut gains = 0.0;
        let mut losses = 0.0;

        for i in 1..=period {
            let change = self.candles[i].close - self.candles[i - 1].close;
            if change > 0.0 {
                gains += change;
            } else {
                losses -= change;
            }
        }

        let avg_gain = gains / period as f64;
        let avg_loss = losses / period as f64;

        if avg_loss.abs() < f64::EPSILON {
            return Some(100.0);
        }

        let rs = avg_gain / avg_loss;
        Some(100.0 - (100.0 / (1.0 + rs)))
    }

    // Add other indicators as needed...
    pub fn atr(&self, period: usize) -> Option<f64> {
        if self.candles.len() < period {
            return None;
        }

        let mut sum = 0.0;
        for i in 1..period {
            let high = self.candles[i].high;
            let low = self.candles[i].low;
            let prev_close = self.candles[i - 1].close;

            let tr1 = high - low;
            let tr2 = (high - prev_close).abs();
            let tr3 = (low - prev_close).abs();

            sum += tr1.max(tr2).max(tr3);
        }

        Some(sum / period as f64)
    }
}
