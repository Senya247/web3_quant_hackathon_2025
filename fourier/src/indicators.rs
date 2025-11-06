use crate::fourier::FourierStrat;

impl FourierStrat {
    // Simple Moving Average
    pub fn sma(&self, period: usize) -> f64 {
        if self.candles().len() < period {
            return 0.0;
        }
        let sum: f64 = self
            .candles()
            .iter()
            .rev()
            .take(period)
            .map(|c| c.close)
            .sum();
        sum / (period as f64)
    }

    // Exponential Moving Average
    pub fn ema(&self, period: usize) -> f64 {
        let candles = self.candles();

        if candles.len() < period {
            return 0.0;
        }
        let alpha = 2.0 / (period as f64 + 1.0);
        let mut ema = candles[candles.len() - period].close;

        for i in (candles.len() - period + 1)..candles.len() {
            ema = alpha * candles[i].close + (1.0 - alpha) * ema;
        }
        ema
    }

    // Price momentum (current close vs previous close)
    pub fn momentum(&self) -> f64 {
        let candles = self.candles();
        if candles.len() < 2 {
            return 0.0;
        }
        let current = candles.last().unwrap().close;
        let prev = candles[candles.len() - 2].close;
        (current - prev) / prev
    }
    // pub fn adx(&self, period: usize) -> f64 {
    //     if self.candles().len() < period * 2 + 1 {
    //         return 50.0; // Default neutral value when not enough data
    //     }
    //
    //     let mut plus_dm_values = Vec::new();
    //     let mut minus_dm_values = Vec::new();
    //     let mut tr_values = Vec::new();
    //
    //     // Calculate +DM, -DM, and TR for each period
    //     for i in 1..self.candles().len() {
    //         let current = &self.candles()[i];
    //         let previous = &self.candles()[i - 1];
    //
    //         // True Range
    //         let tr1 = current.high - current.low;
    //         let tr2 = (current.high - previous.close).abs();
    //         let tr3 = (current.low - previous.close).abs();
    //         let tr = tr1.max(tr2).max(tr3);
    //         tr_values.push(tr);
    //
    //         // Directional Movement
    //         let up_move = current.high - previous.high;
    //         let down_move = previous.low - current.low;
    //
    //         let plus_dm = if up_move > down_move && up_move > 0.0 {
    //             up_move
    //         } else {
    //             0.0
    //         };
    //         let minus_dm = if down_move > up_move && down_move > 0.0 {
    //             down_move
    //         } else {
    //             0.0
    //         };
    //
    //         plus_dm_values.push(plus_dm);
    //         minus_dm_values.push(minus_dm);
    //     }
    //
    //     // Calculate smoothed values
    //     let mut smoothed_plus_dm = Self::wilders_smoothing(&plus_dm_values, period);
    //     let mut smoothed_minus_dm = Self::wilders_smoothing(&minus_dm_values, period);
    //     let smoothed_tr = Self::wilders_smoothing(&tr_values, period);
    //
    //     // Calculate Directional Indicators
    //     let mut plus_di: Vec<f64> = smoothed_plus_dm
    //         .iter()
    //         .zip(&smoothed_tr)
    //         .map(|(dm, tr)| if *tr > 0.0 { 100.0 * dm / tr } else { 0.0 })
    //         .collect();
    //
    //     let mut minus_di: Vec<f64> = smoothed_minus_dm
    //         .iter()
    //         .zip(&smoothed_tr)
    //         .map(|(dm, tr)| if *tr > 0.0 { 100.0 * dm / tr } else { 0.0 })
    //         .collect();
    //
    //     // Calculate DX and ADX
    //     let dx_values: Vec<f64> = plus_di
    //         .iter()
    //         .zip(&minus_di)
    //         .map(|(pdi, mdi)| {
    //             let di_sum = pdi + mdi;
    //             let di_diff = (pdi - mdi).abs();
    //             if di_sum > 0.0 {
    //                 100.0 * di_diff / di_sum
    //             } else {
    //                 0.0
    //             }
    //         })
    //         .collect();
    //
    //     // Final ADX is the smoothed DX
    //     let adx_values = Self::wilders_smoothing(&dx_values, period);
    //
    //     *adx_values.last().unwrap_or(&50.0)
    // }
    //
    // // Wilder's smoothing (EMA-like but different smoothing factor)
    // fn wilders_smoothing(data: &[f64], period: usize) -> Vec<f64> {
    //     if data.len() < period {
    //         return vec![0.0; data.len()];
    //     }
    //
    //     let mut smoothed = Vec::new();
    //
    //     // First value is simple average
    //     let first_avg: f64 = data[0..period].iter().sum::<f64>() / period as f64;
    //     smoothed.push(first_avg);
    //
    //     // Subsequent values use Wilder's smoothing: (previous * (n-1) + current) / n
    //     for i in period..data.len() {
    //         let prev = smoothed.last().unwrap();
    //         let current = data[i];
    //         let new_val = (prev * (period - 1) as f64 + current) / period as f64;
    //         smoothed.push(new_val);
    //     }
    //
    //     smoothed
    // }
    //
    // // Helper method to get +DI
    // pub fn plus_di(&self, period: usize) -> f64 {
    //     self.calculate_di_components(period).0
    // }
    //
    // // Helper method to get -DI
    // pub fn minus_di(&self, period: usize) -> f64 {
    //     self.calculate_di_components(period).1
    // }
    //
    // // Internal method to calculate DI components
    // fn calculate_di_components(&self, period: usize) -> (f64, f64) {
    //     if self.candles().len() < period + 1 {
    //         return (0.0, 0.0);
    //     }
    //
    //     let mut plus_dm_values = Vec::new();
    //     let mut minus_dm_values = Vec::new();
    //     let mut tr_values = Vec::new();
    //
    //     for i in 1..self.candles().len() {
    //         let current = &self.candles()[i];
    //         let previous = &self.candles()[i - 1];
    //
    //         // True Range
    //         let tr1 = current.high - current.low;
    //         let tr2 = (current.high - previous.close).abs();
    //         let tr3 = (current.low - previous.close).abs();
    //         let tr = tr1.max(tr2).max(tr3);
    //         tr_values.push(tr);
    //
    //         // Directional Movement
    //         let up_move = current.high - previous.high;
    //         let down_move = previous.low - current.low;
    //
    //         let plus_dm = if up_move > down_move && up_move > 0.0 {
    //             up_move
    //         } else {
    //             0.0
    //         };
    //         let minus_dm = if down_move > up_move && down_move > 0.0 {
    //             down_move
    //         } else {
    //             0.0
    //         };
    //
    //         plus_dm_values.push(plus_dm);
    //         minus_dm_values.push(minus_dm);
    //     }
    //
    //     let smoothed_plus_dm = Self::wilders_smoothing(&plus_dm_values, period);
    //     let smoothed_minus_dm = Self::wilders_smoothing(&minus_dm_values, period);
    //     let smoothed_tr = Self::wilders_smoothing(&tr_values, period);
    //
    //     let plus_di = if let (Some(dm), Some(tr)) = (smoothed_plus_dm.last(), smoothed_tr.last()) {
    //         if *tr > 0.0 { 100.0 * dm / tr } else { 0.0 }
    //     } else {
    //         // plot.add_trace(Scatter::new(index, btc_price));
    //         //     0.0
    //     };
    //
    //     let minus_di = if let (Some(dm), Some(tr)) = (smoothed_minus_dm.last(), smoothed_tr.last())
    //     {
    //         if *tr > 0.0 { 100.0 * dm / tr } else { 0.0 }
    //     } else {
    //         0.0
    //     };
    //
    //     (plus_di, minus_di)
    // }
}
