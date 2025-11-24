use crate::fourier::Candle;
use crate::strategy::{CandleData, Executioner, Strategy, TraderConfig};
use anyhow::Result;
use tokio::sync::mpsc;
use tokio::time::{Duration, sleep};

pub struct BackTester<T> {
    strategy: T,
}

impl<T: Strategy + Send + 'static> BackTester<T> {
    pub fn create(strategy: T) -> Self {
        BackTester { strategy }
    }

    pub async fn begin(self, csv_file: &str, symbol: &str, initial_capital: f64) -> Result<f64> {
        let reader = csv::Reader::from_path(csv_file)?;

        let (candle_tx, candle_rx) = mpsc::channel(32);
        let (oe_tx, _oe_rx) = mpsc::channel(1);

        let config = TraderConfig {
            initial_capital,
            strategy: self.strategy,
            candle_data_rx: candle_rx,
            order_engine_tx: oe_tx,
            api_key: "BACKTEST".to_string(),
            api_secret: "BACKTEST".to_string(),
        };

        let mut executioner = Executioner::new(config);
        executioner.add_symbol(symbol.to_string(), 3);

        let symbol_name = symbol.to_string();
        let producer = tokio::spawn(async move {
            let tx = candle_tx;
            let mut reader = reader;
            for row in reader.deserialize::<Candle>() {
                match row {
                    Ok(candle) => {
                        let candle_data = CandleData {
                            symbol: symbol_name.clone(),
                            candle,
                        };

                        if tx.send(candle_data).await.is_err() {
                            break;
                        }
                        sleep(Duration::from_millis(1)).await;
                    }
                    Err(e) => {
                        println!("[ERROR][BACKTEST] Failed to parse candle: {}", e);
                    }
                }
            }
        });

        executioner.run(true).await;
        let _ = producer.await;
        Ok(initial_capital)
    }
}
