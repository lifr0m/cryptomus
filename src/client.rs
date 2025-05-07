mod create_invoice;
mod serve_webhook;

use crate::crypto::md5;
use base64::prelude::{Engine, BASE64_STANDARD};
use serde::de::DeserializeOwned;

pub struct Client {
    client: reqwest::Client,
    config: Config,
}

pub struct Config {
    pub merchant_id: String,
    pub api_key: String,
    pub callback_url: String,
}

impl Client {
    pub fn new(config: Config) -> Self {
        let client = reqwest::Client::new();

        Self { client, config }
    }

    async fn request<T>(&self, endpoint: &str, payload: &serde_json::Value) -> reqwest::Result<T>
    where
        T: DeserializeOwned,
    {
        self.client.post(format!("https://api.cryptomus.com{endpoint}"))
            .header("merchant", &self.config.merchant_id)
            .header("sign", compute_signature(&self.config.api_key, payload))
            .json(payload)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }
}

fn compute_signature(api_key: &str, payload: &serde_json::Value) -> String {
    let payload = serde_json::to_vec(payload).unwrap();
    let prepared = format!("{}{}", BASE64_STANDARD.encode(payload), api_key);
    let signature = md5(prepared);
    hex::encode(signature)
}
