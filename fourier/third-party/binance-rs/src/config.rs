#[derive(Clone, Debug)]
pub struct Config {
    pub rest_api_endpoint: String,
    pub ws_endpoint: String,

    pub futures_rest_api_endpoint: String,
    pub futures_ws_endpoint: String,

    pub recv_window: u64,
}

pub const SPOT_MAINNET: &str = "https://api.binance.com";
pub const SPOT_TESTNET: &str = "https://testnet.binance.vision";

pub const SPOT_WS_MAINNET: &str = "wss://stream.binance.com/ws";
pub const SPOT_WS_TESTNET: &str = "wss://testnet.binance.vision/ws";

pub const FUTURES_MAINNET: &str = "https://fapi.binance.com";
pub const FUTURES_TESTNET: &str = "https://testnet.binancefuture.com";

pub const FUTURES_WS_MAINNET: &str = "wss://fstream.binance.com/ws";
pub const FUTURES_WS_TESTNET: &str = "wss://fstream.binancefuture.com/ws";

impl Default for Config {
    fn default() -> Self {
        Self {
            rest_api_endpoint: SPOT_MAINNET.into(),
            ws_endpoint: SPOT_WS_MAINNET.into(),

            futures_rest_api_endpoint: FUTURES_MAINNET.into(),
            futures_ws_endpoint: FUTURES_WS_MAINNET.into(),

            recv_window: 5000,
        }
    }
}

impl Config {
    pub fn testnet() -> Self {
        Self::default()
            .set_rest_api_endpoint(SPOT_TESTNET)
            .set_ws_endpoint(SPOT_WS_TESTNET)
            .set_futures_rest_api_endpoint(FUTURES_TESTNET)
            .set_futures_ws_endpoint(FUTURES_WS_TESTNET)
    }

    pub fn set_rest_api_endpoint<T: Into<String>>(mut self, rest_api_endpoint: T) -> Self {
        self.rest_api_endpoint = rest_api_endpoint.into();
        self
    }

    pub fn set_ws_endpoint<T: Into<String>>(mut self, ws_endpoint: T) -> Self {
        self.ws_endpoint = ws_endpoint.into();
        self
    }
    pub fn set_futures_rest_api_endpoint<T: Into<String>>(
        mut self, futures_rest_api_endpoint: T,
    ) -> Self {
        self.futures_rest_api_endpoint = futures_rest_api_endpoint.into();
        self
    }

    pub fn set_futures_ws_endpoint<T: Into<String>>(mut self, futures_ws_endpoint: T) -> Self {
        self.futures_ws_endpoint = futures_ws_endpoint.into();
        self
    }

    pub fn set_recv_window(mut self, recv_window: u64) -> Self {
        self.recv_window = recv_window;
        self
    }
}
