use dotenv::dotenv;
use plotly::{Bar, Plot};
use serde::Deserialize;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Deserialize, Copy, Clone, Default)]
pub struct CandleMod {
    #[serde(rename = "datetime")]
    pub open_time: u64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub trade_count: i64,
}

// #[tokio::main]
fn main() {
    dotenv().ok();

    // let strategy = Fourier {};
    // let mut backtest = BackTester::create(strategy);
    // let _ = backtest.begin("/home/taru/Programming/comp/web3_quant_hackathon_2025/historical/BTCUSDT-1s-candles-2025-10.csv").await;

    // let market: Market = Binance::new(None, None);
    println!(
        "Current time: {}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    );

    let mut reader =
        csv::Reader::from_path("../../historical/BTCUSDT-1s-candles-2025-10.csv").unwrap();

    let min_delta: i64 = -20000;
    let max_delta: i64 = 33200; // exclusive upper bound
    let len = (max_delta - min_delta) as usize;

    // map from delta to count; we only store counts, delta can be derived: delta = min_delta + idx
    let mut data: Vec<i64> = vec![0; len];

    let mut prev: Option<f64> = None;

    for result in reader.deserialize().skip(2400000) {
        let candle: CandleMod = result.unwrap();
        let cur = candle.close;

        if let Some(p) = prev {
            let delta = (cur * 10.0 - p * 10.0).round() as i64;

            // check if delta is inside the bucket range
            if delta >= min_delta && delta < max_delta {
                let index = (delta - min_delta) as usize; // now in 0..len-1
                data[index] += 1;
            } else {
                // out of tracked range: either ignore or handle specially
                // e.g. clamp: let clamped = delta.clamp(min_delta, max_delta-1);
                // let index = (clamped - min_delta) as usize;
                // data[index] += 1;
            }
        }

        prev = Some(cur);
    }

    // If you need (delta, count) pairs:
    let pairs: Vec<(i64, i64)> = data
        .into_iter()
        .enumerate()
        .map(|(idx, cnt)| (min_delta + idx as i64, cnt))
        .collect();

    let x = pairs.iter().map(|pair| pair.0).collect();
    let y = pairs
        .iter()
        .map(|pair| {
            let x = pair.1 as f64;
            return x.ln();
        })
        .collect();

    let mut plot = Plot::new();
    let trace = Bar::new(x, y);
    plot.add_trace(trace);
    plot.show();

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
