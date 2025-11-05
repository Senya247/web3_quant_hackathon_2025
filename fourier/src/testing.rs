use binance::api::Binance;
use binance::market::Market;

fn main() {
    let market: Market = Binance::new(None, None);

    match market.get_price("ETH") {
        Ok(answer) => println!("{:?}", answer),
        Err(e) => println!("Error: {:?}", e),
    }
}
