use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;

use binance::api::Binance;
use binance::market::Market;
use dotenv::dotenv;
use fourier::fourier::{Candle, FourierStrat};
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

async fn order_engine(mut rx: mpsc::Receiver<Order>, api_key: String, api_secret: String) {
    let client = RoostooClient::new(api_key, api_secret);

    while let Some(order) = rx.recv().await {
        println!("Ordering...");
        let _result = client
            .place_order(
                &order.pair,
                order.side,
                order.order_type,
                order.quantity,
                order.price,
            )
            .await;

        match _result {
            Ok(result) => {
                if result.success {
                    if let Some(order_detail) = result.order_detail {
                        println!("Order filled: {}", order_detail.order_id);
                    } else {
                        println!("Order succeeded but no details returned");
                    }
                } else {
                    println!("Order failed: {}", result.err_msg);
                }
            }
            Err(e) => println!("Genuinly what the fuck happened: {}", e),
        }
    }
}

// async fn market_sync_task(
//     strategy: Arc<Mutex<FourierStrat>>,
//     api_key: String,
//     api_secret: String,
//     symbol: String,
// ) {
//     println!("begin market sync");
//     let client = RoostooClient::new(api_key, api_secret);
//
//     let mut interval = tokio::time::interval(Duration::from_secs(15));
//     loop {
//         interval.tick().await;
//         match client.get_balance().await {
//             Ok(balance_info) => {
//                 let wallet_info = balance_info.spot_wallet;
//
//                 match wallet_info.get(&symbol) {
//                     Some(bi) => {
//                         let mut st = strategy.lock().await;
//                         st.capital = bi.free;
//                         println!("Synced capital");
//                     }
//                     None => {}
//                 };
//             }
//             Err(e) => println!("Couldn't get balance info: {}", e),
//         }
//     }
// }
//
struct TraderConfig {
    symbol: String,
    rx: mpsc::Receiver<(u64, f64)>, // receive time/price
    initial_capital: f64,
    api_key: String,
    api_secret: String,
}

// trader task to trade one symbol.receiver for time/prices as they are generated
async fn trader(config: TraderConfig) {
    let mut rx = config.rx;
    let (_first_time, first_price) = rx.recv().await.unwrap();
    let mut last_price = first_price;

    let warmup_candles = 5;
    let mut candle_index = 0;

    let strategy = Arc::new(Mutex::new(FourierStrat::build(
        config.initial_capital,
        0.01,
    )));

    // order engine, the thing that makes web requests
    let (oe_tx, oe_rx) = mpsc::channel::<Order>(32);
    let api_key = Arc::new(config.api_key.clone());
    let api_secret = Arc::new(config.api_secret.clone());

    let api_key_oe = Arc::clone(&api_key).to_string();
    let api_secret_oe = Arc::clone(&api_secret).to_string();
    tokio::spawn(async move {
        order_engine(oe_rx, api_key_oe, api_secret_oe).await;
    });

    // tokio::spawn(async move {
    //     market_sync_task(
    //         st,
    //         Arc::clone(&api_key).to_string(),
    //         Arc::clone(&api_secret).to_string(),
    //         symbol,
    //     )
    // });

    // In your trader function:
    while let Some((time, cur_price)) = rx.recv().await {
        let mut st = strategy.lock().await; // Note: .await instead of .unwrap()
        let candle = Candle {
            time: time,
            open: last_price,
            high: 0.0,
            low: 0.0,
            close: cur_price,
            volume: 0.0,
            trade_count: 0,
        };
        st.add_candle(candle);
        candle_index += 1;

        if candle_index < warmup_candles {
            continue;
        }

        // Extract the data you need BEFORE the await
        let should_close = st.has_open_position() && st.update_position();
        let close_quantity = if should_close {
            st.open_position_qty()
        } else {
            0.0
        };

        let should_long = st.should_long();
        let long_quantity = if should_long { st.go_long() } else { None };

        // Drop the lock BEFORE awaiting
        drop(st);

        // Now you can await without holding the lock
        if should_close && close_quantity > 0.0 {
            let order = Order {
                pair: [config.symbol.clone(), "/USD".to_string()].concat(),
                side: OrderSide::Sell,
                order_type: OrderType::Market,
                quantity: close_quantity,
                price: None,
            };
            oe_tx.send(order).await.unwrap();
        }

        if let Some(qty) = long_quantity {
            let order = Order {
                pair: [config.symbol.clone(), "/USD".to_string()].concat(),
                side: OrderSide::Buy,
                order_type: OrderType::Market,
                quantity: qty,
                price: None,
            };
            oe_tx.send(order).await.unwrap();
        }

        last_price = cur_price;
    }
}

async fn trading_task(
    mut bt_rx: mpsc::Receiver<BinancePacket>,
    initial_capital: f64,
    api_key: String,
    api_secret: String,
) -> () {
    let (tx, rx) = mpsc::channel(32);
    let config = TraderConfig {
        symbol: "DOGE".to_string(),
        rx: rx,
        initial_capital: initial_capital,
        api_key: api_key,
        api_secret: api_secret,
    };

    let _task = tokio::spawn(async move {
        trader(config).await;
    });

    while let Some(bin_packet) = bt_rx.recv().await {
        let (t, p) = (bin_packet.time, bin_packet.price);
        let _ = tx.send((t, p)).await;
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

    let trader_task = tokio::spawn(async move {
        trading_task(bt_rx, INIT_CAPITAL, rs_api_key, rs_api_secret).await;
    });

    tokio::join!(binance_task, trader_task);
}
