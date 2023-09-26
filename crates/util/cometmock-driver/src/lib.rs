#![deny(clippy::unwrap_used)]
//! The `cometmock-driver` module is designed to control an already
//! running instance of [CometMock](https://github.com/informalsystems/cometmock)
//! from Rust code, for use in testing CometBFT applications.
use anyhow::Result;
use reqwest;
use serde_json::json;
use std::collections::HashMap;
use url::Url;

/// Manager for interacting with an already running instance of CometMock.
pub struct CometMockDriver {
    /// URL for the CometMock API endpoint.
    pub cometmock_url: Url,
    /// Reqwest client for handling HTTP POST requests.
    client: reqwest::Client,
}

impl Default for CometMockDriver {
    fn default() -> Self {
        Self {
            cometmock_url: "tcp://127.0.0.1:22331"
                .parse()
                .expect("can parse localhost url"),
            client: reqwest::Client::new(),
        }
    }
}

impl CometMockDriver {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}

/// Describes an API request made to the CometMock controller API.
pub struct CometMockRequest {
    /// Name of the API method to trigger. Must be supported by CometMock.
    pub method: String,
    /// Map of parameters for the API `method`, e.g. `{"num_blocks": "20"}.
    pub params: HashMap<String, String>,
}

impl CometMockDriver {
    /// Internal function for interacting with the API endpoint. Generalized
    /// to be reusable across multiple methods.
    async fn api_call(&self, r: CometMockRequest) -> Result<()> {
        // Prepare JSON parameters
        let j = json!({
            "jsonrpc": "2.0",
            "method": r.method,
            "params": r.params,
        });

        let mut http_headers = HashMap::new();
        http_headers.insert("Content-Type", "application/json");
        http_headers.insert("Accept", "application/json");

        // Send it
        let r = self
            .client
            .post(self.cometmock_url.clone())
            .json(&j)
            .send()
            .await?;
        r.error_for_status()?;
        Ok(())
    }
    /// Perform time travel, skipping ahead a certain number of blocks.
    /// CometMock will immediately create the intervening blocks.
    pub async fn advance_blocks(&self, num_blocks: u64) -> Result<()> {
        let mut params = HashMap::new();
        params.insert("num_blocks".to_string(), num_blocks.to_string());
        let r = CometMockRequest {
            method: "advance_blocks".to_string(),
            params: params,
        };
        // curl -H 'Content-Type: application/json' -H 'Accept:application/json' --data '{"jsonrpc":"2.0","method":"advance_blocks","params":{"num_blocks": "20"},"id":1}' 127.0.0.1:22331
        self.api_call(r).await
    }
}
