use binance::api::Binance;
use binance::market::Market;
use binance::model::KlineSummaries;
use dotenv::dotenv;
use fourier::fourier::{Candle, Fourier};
use fourier::order_engine::OrderEngine;
use fourier::strategy::{CandleData, Executioner, Strategy, TraderConfig};
use std::collections::HashMap;
use std::env;
use tokio::sync::mpsc;
use tokio::time::{Duration, interval};

const CRYPTOS: [&str; 15] = [
    "BTC", "ETH", "SOL", "BNB", "DOGE", "ICP", "XRP", "AAVE", "UNI", "XLM", "SUI", "BONK", "FIL",
    "TRX", "WIF",
];

async fn binance_task(tx: mpsc::Sender<CandleData>) {
    let cryptos: HashMap<&str, String> = CRYPTOS
        .iter()
        .map(|symbol| (*symbol, format!("{symbol}USDT")))
        .collect();
    let mut throttle = interval(Duration::from_secs(2));

    loop {
        throttle.tick().await;
        let mut handles = Vec::with_capacity(cryptos.len());

        for (real_name, symbol) in &cryptos {
            let real_name = (*real_name).to_string();
            let symbol = symbol.clone();
            handles.push(tokio::task::spawn_blocking(move || {
                let market: Market = Binance::new(None, None);
                let candle = market.get_klines(&symbol, "1s", 1, None, None);
                (real_name, candle)
            }));
        }

        for handle in handles {
            match handle.await {
                Ok((real_name, Ok(KlineSummaries::AllKlineSummaries(mut candles)))) => {
                    if candles.is_empty() {
                        println!("[WARN][BINANCE] No klines returned for {}", real_name);
                        continue;
                    }
                    let kline = candles.remove(0);
                    match parse_kline(&real_name, kline) {
                        Some(candle_data) => {
                            if tx.send(candle_data).await.is_err() {
                                println!("[INFO][BINANCE] Candle consumer dropped, stopping feed");
                                return;
                            }
                        }
                        None => {
                            println!("[ERROR][BINANCE] Failed to parse kline for {}", real_name);
                        }
                    }
                }
                Ok((real_name, Err(e))) => {
                    println!(
                        "[ERROR][BINANCE] Error fetching price for {}: {}",
                        real_name, e
                    );
                }
                Err(e) => {
                    println!("[ERROR][BINANCE] Task error: {}", e);
                }
            }
        }
    }
}

fn parse_kline(symbol: &str, kline: binance::model::KlineSummary) -> Option<CandleData> {
    let parse_number = |value: &str| value.parse::<f64>().ok();
    Some(CandleData {
        symbol: symbol.to_string(),
        candle: Candle {
            open_time: kline.open_time as u64,
            close_time: kline.close_time as u64,
            open: parse_number(&kline.open)?,
            high: parse_number(&kline.high)?,
            low: parse_number(&kline.low)?,
            close: parse_number(&kline.close)?,
            volume: parse_number(&kline.volume)?,
            trade_count: kline.number_of_trades,
        },
    })
}

// trader task to trade one symbol.receiver for time/prices as they are generated
async fn trader<T: Strategy + Send>(config: TraderConfig<T>) {
    println!("IN TRADER");
    let mut executioner = Executioner::new(config);
    for symbol in CRYPTOS {
        executioner.add_symbol(symbol.to_string(), default_precision(symbol));
    }

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

    let trader_handle = tokio::spawn(async move {
        trader(config).await;
    });

    let order_engine_handle = tokio::spawn(async move {
        let mut engine = OrderEngine::build(api_key, api_secret);
        engine.run(oe_rx).await;
    });

    while let Some(candledata) = bt_rx.recv().await {
        if candle_tx.send(candledata).await.is_err() {
            break;
        }
    }

    trader_handle.await.ok();
    order_engine_handle.await.ok();
}
//

const INIT_CAPITAL: f64 = 50_005.91;

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

    let (binance_res, trader_res) = tokio::join!(binance_task, trader_task);
    if let Err(e) = binance_res {
        eprintln!("[FATAL] Binance task crashed: {}", e);
    }
    if let Err(e) = trader_res {
        eprintln!("[FATAL] Trader task crashed: {}", e);
    }
}

fn default_precision(symbol: &str) -> u64 {
    match symbol {
        "BTC" => 5,
        "ETH" => 4,
        "SOL" => 3,
        "BNB" => 3,
        "DOGE" => 0,
        "ICP" => 2,
        "XRP" => 1,
        "AAVE" => 3,
        "UNI" => 2,
        "XLM" => 0,
        "SUI" => 1,
        "BONK" => 0,
        "FIL" => 2,
        "TRX" => 1,
        "WIF" => 2,
        _ => 2,
    }
}
