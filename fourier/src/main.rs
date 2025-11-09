use binance::api::Binance;
use binance::market::Market;
use binance::model::KlineSummaries;
use dotenv::dotenv;
use fourier::fourier::{Candle, Fourier};
use fourier::order_engine::OrderEngine;
use fourier::strategy::{CandleData, Executioner, Strategy, TraderConfig};
use std::collections::HashMap;
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc;

const CRYPTOS: [&str; 1] = ["DOGE"];

async fn binance_task(tx: mpsc::Sender<CandleData>) -> () {
    let mut cryptos = HashMap::new();
    for crypto in CRYPTOS {
        cryptos.insert(crypto, [crypto, "USDT"].concat());
    }

    loop {
        let mut handles = vec![];

        for (real_name, symbol) in &cryptos {
            // make owned copies so spawn_blocking can move them into the closure
            let real_name = real_name.to_string();
            let symbol = symbol.clone();

            let handle = tokio::task::spawn_blocking(move || {
                let market: Market = Binance::new(None, None);
                let candle = market.get_klines(&symbol, "1s", 1, None, None);
                (real_name, candle)
            });
            handles.push(handle);
        }

        // Wait for all requests to complete
        for handle in handles {
            match handle.await {
                Ok((real_name, Ok(_candle))) => {
                    let kline = match _candle {
                        KlineSummaries::AllKlineSummaries(mut v) => v.remove(0),
                        _ => {
                            panic!("WHA THEE FUCKK")
                        }
                    };

                    let open = kline.open.parse();
                    let high = kline.high.parse();
                    let low = kline.low.parse();
                    let close = kline.close.parse();
                    let volume = kline.volume.parse();

                    match (open, high, low, close, volume) {
                        (Ok(open), Ok(high), Ok(low), Ok(close), Ok(volume)) => {
                            tx.send(CandleData {
                                symbol: real_name,
                                candle: Candle {
                                    open_time: kline.open_time as u64,
                                    close_time: kline.close_time as u64,
                                    open: open,
                                    high: high,
                                    low: low,
                                    close: close,
                                    volume: volume,
                                    trade_count: kline.number_of_trades,
                                },
                            })
                            .await
                            .unwrap();
                        }
                        (_, _, _, _, _) => {
                            println!("[ERROR][BINANCE] Unable to parse data");
                        }
                    };
                }
                Ok((real_name, Err(e))) => {
                    println!("Error fetching price for {}: {}", real_name, e);
                }
                Err(e) => {
                    println!("Task error: {}", e);
                }
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}

// trader task to trade one symbol.receiver for time/prices as they are generated
async fn trader<T: Strategy + Send>(config: TraderConfig<T>) {
    println!("IN TRADER");
    let mut executioner = Executioner::new(config);
    executioner.add_symbol("DOGE".to_string(), 0);
    executioner.run(false).await;
}

async fn trading_task<T: Strategy + Send + 'static + std::marker::Sync>(
    mut bt_rx: mpsc::Receiver<CandleData>,
    initial_capital: f64,
    api_key: String,
    api_secret: String,
    strategy: T,
) -> () {
    let (candle_tx, candle_rx) = mpsc::channel(32);
    let (oe_tx, oe_rx) = mpsc::channel(32);
    let config = TraderConfig {
        initial_capital: initial_capital,
        strategy: strategy,
        candle_data_rx: candle_rx,
        order_engine_tx: oe_tx,
        api_key: api_key.clone(),
        api_secret: api_secret.clone(),
    };

    let _trader = tokio::spawn(async move {
        trader(config).await;
    });

    let _order_engine = tokio::spawn(async move {
        let mut engine = OrderEngine::build(api_key, api_secret);
        engine.run(oe_rx).await;
    });

    while let Some(candledata) = bt_rx.recv().await {
        let _ = candle_tx.send(candledata).await;
    }
}
//

const INIT_CAPITAL: f64 = 49728.16;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let rs_api_key = env::var("ROOSTOO_API_KEY").unwrap();
    let rs_api_secret = env::var("ROOSTOO_API_SECRET").unwrap();

    let (bt_tx, bt_rx) = mpsc::channel(32);
    let binance_task = tokio::spawn(async move {
        binance_task(bt_tx).await;
    });

    let god_strategy = Fourier {};
    let trader_task = tokio::spawn(async move {
        trading_task(bt_rx, INIT_CAPITAL, rs_api_key, rs_api_secret, god_strategy).await;
    });

    tokio::join!(binance_task, trader_task);
}
