use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderValue};
use sha2::Sha256;
use std::collections::HashMap;
use thiserror::Error;

type Result<T> = std::result::Result<T, RoostooError>;

#[derive(Error, Debug)]
pub enum RoostooError {
    #[error("HTTP request error: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("API error: {0}")]
    ApiError(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Authentication error: {0}")]
    AuthError(String),

    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    #[error("Invalid JSON: {0}")]
    JsonParseError(String)
}

#[derive(Debug, Clone)]
pub struct RoostooClient {
    base_url: String,
    api_key: String,
    secret_key: String,
    client: reqwest::Client,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerTime {
    #[serde(rename = "ServerTime")]
    pub server_time: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeInfo {
    #[serde(rename = "IsRunning")]
    pub is_running: bool,
    #[serde(rename = "InitialWallet")]
    pub initial_wallet: HashMap<String, f64>,
    #[serde(rename = "TradePairs")]
    pub trade_pairs: HashMap<String, TradePair>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradePair {
    #[serde(rename = "Coin")]
    pub coin: String,
    #[serde(rename = "CoinFullName")]
    pub coin_full_name: String,
    #[serde(rename = "Unit")]
    pub unit: String,
    #[serde(rename = "UnitFullName")]
    pub unit_full_name: String,
    #[serde(rename = "CanTrade")]
    pub can_trade: bool,
    #[serde(rename = "PricePrecision")]
    pub price_precision: u32,
    #[serde(rename = "AmountPrecision")]
    pub amount_precision: u32,
    #[serde(rename = "MiniOrder")]
    pub mini_order: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickerData {
    #[serde(rename = "MaxBid")]
    pub max_bid: f64,
    #[serde(rename = "MinAsk")]
    pub min_ask: f64,
    #[serde(rename = "LastPrice")]
    pub last_price: f64,
    #[serde(rename = "Change")]
    pub change: f64,
    #[serde(rename = "CoinTradeValue")]
    pub coin_trade_value: f64,
    #[serde(rename = "UnitTradeValue")]
    pub unit_trade_value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickerResponse {
    #[serde(rename = "Success")]
    pub success: bool,
    #[serde(rename = "ErrMsg")]
    pub err_msg: String,
    #[serde(rename = "ServerTime")]
    pub server_time: u64,
    #[serde(rename = "Data")]
    pub data: HashMap<String, TickerData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceInfo {
    #[serde(rename = "Free")]
    pub free: f64,
    #[serde(rename = "Lock")]
    pub lock: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceResponse {
    #[serde(rename = "Success")]
    pub success: bool,
    #[serde(rename = "ErrMsg")]
    pub err_msg: String,
    #[serde(rename = "SpotWallet")]
    pub spot_wallet: HashMap<String, BalanceInfo>,
    #[serde(rename = "MarginWallet")]
    pub margin_wallet: HashMap<String, BalanceInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingCountResponse {
    #[serde(rename = "Success")]
    pub success: bool,
    #[serde(rename = "ErrMsg")]
    pub err_msg: String,
    #[serde(rename = "TotalPending")]
    pub total_pending: u32,
    #[serde(rename = "OrderPairs")]
    pub order_pairs: HashMap<String, u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderDetail {
    #[serde(rename = "Pair")]
    pub pair: String,
    #[serde(rename = "OrderID")]
    pub order_id: u64,
    #[serde(rename = "Status")]
    pub status: String,
    #[serde(rename = "Role")]
    pub role: String,
    #[serde(rename = "ServerTimeUsage")]
    pub server_time_usage: f64,
    #[serde(rename = "CreateTimestamp")]
    pub create_timestamp: u64,
    #[serde(rename = "FinishTimestamp")]
    pub finish_timestamp: u64,
    #[serde(rename = "Side")]
    pub side: String,
    #[serde(rename = "Type")]
    pub order_type: String,
    #[serde(rename = "StopType")]
    pub stop_type: String,
    #[serde(rename = "Price")]
    pub price: f64,
    #[serde(rename = "Quantity")]
    pub quantity: f64,
    #[serde(rename = "FilledQuantity")]
    pub filled_quantity: f64,
    #[serde(rename = "FilledAverPrice")]
    pub filled_aver_price: f64,
    #[serde(rename = "CoinChange")]
    pub coin_change: f64,
    #[serde(rename = "UnitChange")]
    pub unit_change: f64,
    #[serde(rename = "CommissionCoin")]
    pub commission_coin: String,
    #[serde(rename = "CommissionChargeValue")]
    pub commission_charge_value: f64,
    #[serde(rename = "CommissionPercent")]
    pub commission_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaceOrderResponse {
    #[serde(rename = "Success")]
    pub success: bool,
    #[serde(rename = "ErrMsg")]
    pub err_msg: String,
    #[serde(rename = "OrderDetail")]
    pub order_detail: Option<OrderDetail>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryOrderResponse {
    #[serde(rename = "Success")]
    pub success: bool,
    #[serde(rename = "ErrMsg")]
    pub err_msg: String,
    #[serde(rename = "OrderMatched")]
    pub order_matched: Vec<OrderDetail>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelOrderResponse {
    #[serde(rename = "Success")]
    pub success: bool,
    #[serde(rename = "ErrMsg")]
    pub err_msg: String,
    #[serde(rename = "CanceledList")]
    pub canceled_list: Vec<u64>,
}

#[derive(Debug, Clone)]
pub enum OrderType {
    Limit,
    Market,
}

impl std::fmt::Display for OrderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderType::Limit => write!(f, "LIMIT"),
            OrderType::Market => write!(f, "MARKET"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum OrderSide {
    Buy,
    Sell,
}

impl std::fmt::Display for OrderSide {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderSide::Buy => write!(f, "BUY"),
            OrderSide::Sell => write!(f, "SELL"),
        }
    }
}

impl RoostooClient {
    pub fn new(api_key: String, secret_key: String) -> Self {
        Self {
            base_url: "https://mock-api.roostoo.com".to_string(),
            api_key,
            secret_key,
            client: reqwest::Client::new(),
        }
    }

    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url = base_url;
        self
    }

    fn get_timestamp(&self) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }

    fn create_signature(&self, params: &str) -> String {
        let mut mac = Hmac::<Sha256>::new_from_slice(self.secret_key.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(params.as_bytes());
        let result = mac.finalize();
        let code_bytes = result.into_bytes();
        hex::encode(code_bytes)
    }

    fn create_signed_headers(&self, params: &str) -> HeaderMap {
        let signature = self.create_signature(params);
        let mut headers = HeaderMap::new();
        headers.insert("RST-API-KEY", HeaderValue::from_str(&self.api_key).unwrap());
        headers.insert("MSG-SIGNATURE", HeaderValue::from_str(&signature).unwrap());
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_static("application/x-www-form-urlencoded"),
        );
        headers
    }

    /// Check server time
    /// GET /v3/serverTime
    pub async fn check_server_time(&self) -> Result<ServerTime> {
        let url = format!("{}/v3/serverTime", self.base_url);
        let response = self.client.get(&url).send().await?;

        if response.status().is_success() {
            let server_time: ServerTime = response.json().await?;
            Ok(server_time)
        } else {
            Err(RoostooError::ApiError(
                "Failed to get server time".to_string(),
            ))
        }
    }

    /// Exchange information
    /// GET /v3/exchangeInfo
    pub async fn get_exchange_info(&self) -> Result<ExchangeInfo> {
        let url = format!("{}/v3/exchangeInfo", self.base_url);
        let response = self.client.get(&url).send().await?;

        if response.status().is_success() {
            let exchange_info: ExchangeInfo = response.json().await?;
            Ok(exchange_info)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            eprintln!("Failed to get exchange info. Response: {}", error_text);
            Err(RoostooError::ApiError(
                "Failed to get exchange info".to_string(),
            ))
        }
    }

    /// Get Market Ticker
    /// GET /v3/ticker
    pub async fn get_ticker(&self, pair: Option<&str>) -> Result<TickerResponse> {
        let url = format!("{}/v3/ticker", self.base_url);
        let timestamp = self.get_timestamp();

        let mut params = HashMap::new();
        params.insert("timestamp", timestamp.to_string());
        // if let Some(p) = pair {
        //     params.insert("pair", p.to_string());
        // }
        params.insert("pair", pair.unwrap().to_string());

        let response = self.client.get(&url).query(&params).send().await?;

        if response.status().is_success() {
            let ticker_response: TickerResponse = response.json().await?;
            if ticker_response.success {
                Ok(ticker_response)
            } else {
                Err(RoostooError::ApiError(ticker_response.err_msg))
            }
        } else {
            Err(RoostooError::ApiError("Failed to get ticker".to_string()))
        }
    }

    /// Balance information
    /// GET /v3/balance
    /// Balance information
    /// GET /v3/balance
    pub async fn get_balance(&self) -> Result<BalanceResponse> {
        let url = format!("{}/v3/balance", self.base_url);
        let timestamp = self.get_timestamp();

        let params = format!("timestamp={}", timestamp);
        let headers = self.create_signed_headers(&params);

        let response = self
            .client
            .get(&url)
            .query(&[("timestamp", timestamp.to_string())])
            .headers(headers)
            .send()
            .await?;

        if response.status().is_success() {
            let response_text = response.text().await?;
            // println!("Balance API response: {}", response_text); // Debug output

            let balance_response: BalanceResponse =
                serde_json::from_str(&response_text).map_err(|e| {
                    RoostooError::SerializationError(e) // Pass the error directly, not a formatted string
                })?;

            if balance_response.success {
                Ok(balance_response)
            } else {
                Err(RoostooError::ApiError(balance_response.err_msg))
            }
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(RoostooError::ApiError(format!(
                "HTTP error: {}",
                error_text
            )))
        }
    }
    /// Pending Order Count
    /// GET /v3/pending_count
    pub async fn get_pending_count(&self) -> Result<PendingCountResponse> {
        let url = format!("{}/v3/pending_count", self.base_url);
        let timestamp = self.get_timestamp();

        let params = format!("timestamp={}", timestamp);
        let headers = self.create_signed_headers(&params);

        let response = self
            .client
            .get(&url)
            .query(&[("timestamp", timestamp.to_string())])
            .headers(headers)
            .send()
            .await?;

        let pending_response: PendingCountResponse = response.json().await?;
        return Ok(pending_response);
    }

    /// New order (Trade)
    /// POST /v3/place_order
    pub async fn place_order(
        &self,
        pair: &str,
        side: OrderSide,
        order_type: OrderType,
        quantity: f64,
        price: Option<f64>,
    ) -> Result<PlaceOrderResponse> {
        let url = format!("{}/v3/place_order", self.base_url);
        let timestamp = self.get_timestamp();

        // Validate parameters
        if let OrderType::Limit = order_type {
            if price.is_none() {
                return Err(RoostooError::InvalidParameter(
                    "LIMIT orders require a price".to_string(),
                ));
            }
        }

        let mut params = HashMap::new();
        params.insert("pair", pair.to_string());
        params.insert("side", side.to_string());
        params.insert("type", order_type.to_string());
        params.insert("quantity", quantity.to_string());
        params.insert("timestamp", timestamp.to_string());

        if let Some(p) = price {
            params.insert("price", p.to_string());
        }

        // Sort parameters by key and create the parameter string
        let mut sorted_params: Vec<_> = params.iter().collect();
        sorted_params.sort_by_key(|(k, _)| *k);

        let param_string = sorted_params
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&");

        let headers = self.create_signed_headers(&param_string);
        let response = self
            .client
            .post(&url)
            .headers(headers)
            .body(param_string.clone())
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(RoostooError::ApiError("HTTP request failed".to_string()));
        }

        let raw_text = response.text().await?;
        let json_value: Value = serde_json::from_str(&raw_text)
            .map_err(|e| RoostooError::JsonParseError(format!("Invalid JSON: {}", e)))?;

        // Extract basic fields
        let success = json_value["Success"]
            .as_bool()
            .ok_or_else(|| RoostooError::JsonParseError("Missing 'Success' field".to_string()))?;

        let err_msg = json_value["ErrMsg"].as_str().unwrap_or("").to_string();

        if !success {
            return Ok(PlaceOrderResponse {
                success: false,
                err_msg,
                order_detail: None,
            });
        }

        // Try to parse OrderDetail if present
        let order_detail = if let Some(order_detail_value) = json_value.get("OrderDetail") {
            match serde_json::from_value::<OrderDetail>(order_detail_value.clone()) {
                Ok(detail) => Some(detail),
                Err(e) => {
                    // Log the error but don't fail the entire request
                    eprintln!("Failed to parse OrderDetail: {}", e);
                    None
                }
            }
        } else {
            None
        };

        Ok(PlaceOrderResponse {
            success: true,
            err_msg,
            order_detail,
        })
    }

    /// Query order
    /// POST /v3/query_order
    pub async fn query_order(
        &self,
        order_id: Option<u64>,
        pair: Option<&str>,
        pending_only: Option<bool>,
    ) -> Result<QueryOrderResponse> {
        let url = format!("{}/v3/query_order", self.base_url);
        let timestamp = self.get_timestamp();

        let mut params = HashMap::new();
        params.insert("timestamp", timestamp.to_string());

        if let Some(id) = order_id {
            params.insert("order_id", id.to_string());
        } else if let Some(p) = pair {
            params.insert("pair", p.to_string());
            if let Some(po) = pending_only {
                params.insert(
                    "pending_only",
                    if po { "TRUE" } else { "FALSE" }.to_string(),
                );
            }
        }

        // Sort parameters by key and create the parameter string
        let mut sorted_params: Vec<_> = params.iter().collect();
        sorted_params.sort_by_key(|(k, _)| *k);

        let param_string = sorted_params
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&");

        let headers = self.create_signed_headers(&param_string);

        let response = self
            .client
            .post(&url)
            .headers(headers)
            .body(param_string.clone())
            .send()
            .await?;

        if response.status().is_success() {
            let query_response: QueryOrderResponse = response.json().await?;
            if query_response.success {
                Ok(query_response)
            } else {
                Err(RoostooError::ApiError(query_response.err_msg))
            }
        } else {
            Err(RoostooError::ApiError("Failed to query order".to_string()))
        }
    }

    /// Cancel order
    /// POST /v3/cancel_order
    pub async fn cancel_order(
        &self,
        order_id: Option<u64>,
        pair: Option<&str>,
    ) -> Result<CancelOrderResponse> {
        let url = format!("{}/v3/cancel_order", self.base_url);
        let timestamp = self.get_timestamp();

        let mut params = HashMap::new();
        params.insert("timestamp", timestamp.to_string());

        if let Some(id) = order_id {
            params.insert("order_id", id.to_string());
        } else if let Some(p) = pair {
            params.insert("pair", p.to_string());
        }

        // Sort parameters by key and create the parameter string
        let mut sorted_params: Vec<_> = params.iter().collect();
        sorted_params.sort_by_key(|(k, _)| *k);

        let param_string = sorted_params
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&");

        let headers = self.create_signed_headers(&param_string);

        let response = self
            .client
            .post(&url)
            .headers(headers)
            .body(param_string.clone())
            .send()
            .await?;

        if response.status().is_success() {
            let cancel_response: CancelOrderResponse = response.json().await?;
            if cancel_response.success {
                Ok(cancel_response)
            } else {
                Err(RoostooError::ApiError(cancel_response.err_msg))
            }
        } else {
            Err(RoostooError::ApiError("Failed to cancel order".to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order_type_display() {
        assert_eq!(OrderType::Limit.to_string(), "LIMIT");
        assert_eq!(OrderType::Market.to_string(), "MARKET");
    }

    #[test]
    fn test_order_side_display() {
        assert_eq!(OrderSide::Buy.to_string(), "BUY");
        assert_eq!(OrderSide::Sell.to_string(), "SELL");
    }

    #[tokio::test]
    async fn test_client_creation() {
        let client = RoostooClient::new("test_api_key".to_string(), "test_secret_key".to_string());
        assert_eq!(client.api_key, "test_api_key");
        assert_eq!(client.secret_key, "test_secret_key");
    }
}
