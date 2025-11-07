use crate::roostoo::RoostooClient;
use crate::strategy::Order;
use tokio::sync::mpsc;

pub struct OrderEngine {
    client: RoostooClient,
}

impl OrderEngine {
    pub fn build(api_key: String, api_secret: String) -> Self {
        return Self {
            client: RoostooClient::new(api_key, api_secret),
        };
    }
    pub async fn run(&mut self, mut rx: mpsc::Receiver<Order>) {
        while let Some(order) = rx.recv().await {
            println!("Ordering...");
            let _result = self
                .client
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
}
