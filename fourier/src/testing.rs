use binance::{api::Binance, market::Market};
use dotenv::dotenv;
use fourier::{
    backtest::BackTester,
    fourier::Fourier,
    roostoo::{OrderSide, OrderType, RoostooClient},
};
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

// #[tokio::main]
fn main() {
    dotenv().ok();

    let rs_api_key = env::var("ROOSTOO_API_KEY").unwrap();
    let rs_api_secret = env::var("ROOSTOO_API_SECRET").unwrap();

    // let strategy = Fourier {};
    // let mut backtest = BackTester::create(strategy);
    // let _ = backtest.begin("/home/taru/Programming/comp/web3_quant_hackathon_2025/historical/BTCUSDT-1s-candles-2025-10.csv").await;

    let market: Market = Binance::new(None, None);
    println!(
        "Current time: {}",
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
    );
    println!("{:?}", market.get_klines("DOGEUSDT", "1s", 1, None, None));
    // let client = RoostooClient::new(rs_api_key, rs_api_secret);
    // println!("{:?}", client.get_balance().await.unwrap());
    //
    // let result = client
    //     .place_order(
    //         "DOGE/USD",
    //         OrderSide::Sell,
    //         OrderType::Market,
    //         30634.0,
    //         None,
    //     )
    //     .await
    //     .unwrap();
    //
    // println!("{:?}", result);
    // println!("{:?}", client.get_balance().await.unwrap());
}
