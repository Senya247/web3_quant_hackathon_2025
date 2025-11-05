use crate::fourier::FourierStrat;

impl FourierStrat {
    pub fn sma(&self, period: usize) -> f64 {
        assert!(self.candles().len() >= period);
        let sum: f64 = self
            .candles()
            .iter()
            .rev()
            .take(period)
            .map(|&candle| candle.close)
            .sum();
        return sum / (period as f64);
    }
}
