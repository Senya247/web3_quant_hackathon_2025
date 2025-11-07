use fourier::strategy::{CandleData, Executioner, Strategy, TraderConfig};
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;

use fourier::order_engine::OrderEngine;

use binance::api::Binance;
use binance::market::Market;
use dotenv::dotenv;
use fourier::fourier::{Candle, Fourier, UnusedPieceofShit};
use fourier::roostoo::RoostooClient;
use fourier::roostoo::{OrderSide, OrderType};
use tokio::sync::mpsc;
use tokio::time::interval;

const CRYPTOS: [&str; 1] = ["DOGE"];

struct BinancePacket {
    symbol: String,
    time: u64,
    price: f64,
}

async fn binance_task(tx: mpsc::Sender<BinancePacket>) -> () {
    let mut cryptos = HashMap::new();
    for crypto in CRYPTOS {
        cryptos.insert(crypto, [crypto, "USDT"].concat());
    }

    loop {
        let mut handles = vec![];

        for (_c, symbol) in &cryptos {
            let symbol = symbol.clone();
            let tx = tx.clone();

            let handle = tokio::task::spawn_blocking(move || {
                let market: Market = Binance::new(None, None);
                let price = market.get_price(&symbol);
                (symbol, price)
            });
            handles.push(handle);
        }

        // Wait for all requests to complete
        for handle in handles {
            match handle.await {
                Ok((symbol, Ok(price))) => {
                    tx.send(BinancePacket {
                        time: SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                        symbol: symbol.clone(),
                        price: price.price,
                    })
                    .await
                    .unwrap();
                }
                Ok((symbol, Err(e))) => {
                    println!("Error fetching price for {}: {}", symbol, e);
                }
                Err(e) => {
                    println!("Task error: {}", e);
                }
            }
        }

        // Optional: add a small delay between iterations to avoid rate limiting
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}

struct Order {
    pair: String,
    side: OrderSide,
    order_type: OrderType,
    quantity: f64,
    price: Option<f64>,
}

// trader task to trade one symbol.receiver for time/prices as they are generated
async fn trader<T: Strategy+Send>(config: TraderConfig<T>) {
    let mut executioner = Executioner::new(config);
    executioner.run();
}

async fn trading_task<T: Strategy+Send + 'static>(
    mut bt_rx: mpsc::Receiver<BinancePacket>,
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
        engine.run(oe_rx);
    });

    while let Some(bin_packet) = bt_rx.recv().await {
        let candledata = CandleData {
            symbol: bin_packet.symbol,
            candle: Candle {
                time: bin_packet.time,
                open: 0.0,
                high: 0.0,
                low: 0.0,
                close: bin_packet.price,
                volume: 0.0,
                trade_count: 0,
            },
        };

        let _ = candle_tx.send(candledata).await;
    }
}
//

const INIT_CAPITAL: f64 = 5000.0;

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
