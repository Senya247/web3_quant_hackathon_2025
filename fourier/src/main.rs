use std::env;

use dotenv::dotenv;
use fourier::roostoo::RoostooClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let rs_api_key = env::var("ROOSTOO_API_KEY").expect("Roostoo API key not in env");
    let rs_api_secret = env::var("ROOSTOO_API_SECRET").expect("Roostoo API secret not in env");
    let client = RoostooClient::new(rs_api_key, rs_api_secret);

    // Check server time
    let server_time = client.check_server_time().await?;
    println!("Server time: {}", server_time.server_time);

    // Get exchange info
    let exchange_info = client.get_exchange_info().await?;
    println!("Exchange is running: {}", exchange_info.is_running);

    // Get ticker for all pairs
    let ticker = client.get_ticker(None).await?;
    println!("Got ticker data for {} pairs", ticker.data.len());

    // Get ticker for specific pair
    let btc_ticker = client.get_ticker(Some("BTC/USD")).await?;
    if let Some(btc_data) = btc_ticker.data.get("BTC/USD") {
        println!("BTC last price: {}", btc_data.last_price);
    }

    // Get balance
    let balance = client.get_balance().await?;
    println!(
        "Wallet balances: {:?}, {:?}",
        balance.spot_wallet, balance.margin_wallet
    );

    // Get pending order count
    let pending_count = client.get_pending_count().await?;
    println!("Total pending orders: {}", pending_count.total_pending);

    // Query orders
    let orders = client
        .query_order(None, Some("BTC/USD"), Some(true))
        .await?;
    println!("Found {} matching orders", orders.order_matched.len());

    Ok(())
}
