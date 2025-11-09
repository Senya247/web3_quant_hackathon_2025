use std::any::Any;

use crate::roostoo::{OrderDetail, RoostooClient};
use crate::strategy::Order;
use tokio::sync::{mpsc, oneshot};

pub struct OrderWithResponse {
    pub order: Order,
    pub precision: u64,
    pub response: oneshot::Sender<OrderDetail>,
}
pub struct OrderEngine {
    client: RoostooClient,
}

impl OrderEngine {
    pub fn build(api_key: String, api_secret: String) -> Self {
        return Self {
            client: RoostooClient::new(api_key, api_secret),
        };
    }
    pub async fn run(&mut self, mut rx: mpsc::Receiver<OrderWithResponse>) {
        while let Some(order) = rx.recv().await {
            let factor = 10f64.powi(order.precision as i32);
            let rounded = (order.order.quantity * factor).round() / factor;
            // println!(
            //     "[INFO][ORDERENGINE] executioner sent {} {}",
            //     order.order.pair.clone(),
            //     rounded,
            // );

            let _result = self
                .client
                .place_order(
                    &order.order.pair,
                    order.order.side,
                    order.order.order_type.clone(),
                    rounded,
                    order.order.price,
                )
                .await;

            match _result {
                Ok(result) => {
                    if result.success {
                        if let Some(order_detail) = result.order_detail {
                            // println!("Order filled: {}", order_detail.order_id);
                            println!(
                                "[SUCCESS][ORDERENGINE] {} {} {} @{} @fee {}",
                                order_detail.pair.clone(),
                                order_detail.side.to_uppercase(),
                                order_detail.filled_quantity,
                                order_detail.filled_aver_price,
                                order_detail.commission_charge_value,
                            );
                            let _ = order.response.send(order_detail);
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
