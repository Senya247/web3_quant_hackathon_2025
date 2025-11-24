use crate::fourier::Candle;

pub struct Indicators<'a> {
    candles: &'a [Candle],
}

impl<'a> Indicators<'a> {
    pub fn new(candles: &'a [Candle]) -> Self {
        Self { candles }
    }

    pub fn ema(&self, period: usize) -> Option<f64> {
        if period == 0 || self.candles.len() < period {
            return None;
        }

        let alpha = 2.0 / (period as f64 + 1.0);
        let start = self.candles.len() - period;
        let window = &self.candles[start..];
        let mut ema = window[0].close;

        for candle in window.iter().skip(1) {
            ema = alpha * candle.close + (1.0 - alpha) * ema;
        }

        Some(ema)
    }

    pub fn ema_series<I>(&self, data: I, period: usize) -> Option<f64>
    where
        I: IntoIterator<Item = f64>,
    {
        if period == 0 {
            return None;
        }

        let mut values: Vec<f64> = data.into_iter().collect();
        if values.len() < period {
            return None;
        }
        values = values.split_off(values.len() - period);

        let alpha = 2.0 / (period as f64 + 1.0);
        let mut ema = values[0];
        for value in values.iter().skip(1) {
            ema = alpha * value + (1.0 - alpha) * ema;
        }
        Some(ema)
    }

    pub fn stddev_series<I>(&self, data: I, period: usize) -> Option<f64>
    where
        I: IntoIterator<Item = f64>,
    {
        let values: Vec<f64> = data.into_iter().collect();

        if values.len() < period {
            return None;
        }

        let window = &values[values.len() - period..];
        let mean = window.iter().sum::<f64>() / period as f64;
        let variance = window.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / period as f64;

        Some(variance.sqrt())
    }

    pub fn sma(&self, period: usize) -> Option<f64> {
        if self.candles.len() < period {
            return None;
        }

        let sum: f64 = self.candles[self.candles.len() - period..]
            .iter()
            .map(|c| c.close)
            .sum();

        Some(sum / period as f64)
    }

    pub fn rsi(&self, period: usize) -> Option<f64> {
        if period == 0 || self.candles.len() <= period {
            return None;
        }

        let window = &self.candles[self.candles.len() - (period + 1)..];
        let mut gains = 0.0f64;
        let mut losses = 0.0f64;

        for i in 1..window.len() {
            let change = window[i].close - window[i - 1].close;
            if change > 0.0 {
                gains += change;
            } else {
                losses += change.abs();
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
        if self.candles.len() < period + 1 {
            return None;
        }

        let mut sum = 0.0;
        let start_idx = self.candles.len() - period;

        for i in start_idx..self.candles.len() {
            let prev = &self.candles[i - 1];
            let cur = &self.candles[i];

            let tr1 = cur.high - cur.low;
            let tr2 = (cur.high - prev.close).abs();
            let tr3 = (cur.low - prev.close).abs();

            sum += tr1.max(tr2).max(tr3);
        }

        Some(sum / period as f64)
    }
}
