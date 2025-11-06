use dotenv::dotenv;
use fourier::roostoo::{OrderSide, OrderType, RoostooClient};
use std::env;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let rs_api_key = env::var("ROOSTOO_API_KEY").unwrap();
    let rs_api_secret = env::var("ROOSTOO_API_SECRET").unwrap();

    let client = RoostooClient::new(rs_api_key, rs_api_secret);
    println!("{:?}", client.get_balance().await.unwrap());

    let result = client
        .place_order(
            "DOGE/USD",
            OrderSide::Sell,
            OrderType::Market,
            30634.0,
            None,
        )
        .await
        .unwrap();

    println!("{:?}", result);
    println!("{:?}", client.get_balance().await.unwrap());
}
