mod create_invoice;
pub mod serve_webhook;

use crate::crypto::md5;
use base64::prelude::{Engine, BASE64_STANDARD};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use url::Url;

#[derive(Clone)]
pub struct Client {
    client: reqwest::Client,
    config: Config,
}

#[derive(Clone, Deserialize)]
pub struct Config {
    pub merchant_id: String,
    pub api_key: String,
    pub callback_url: Url,
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
        let signature = compute_signature(
            &self.config.api_key,
            serde_json::to_vec(payload).unwrap(),
        );
        self.client.post(format!("https://api.cryptomus.com{endpoint}"))
            .header("merchant", &self.config.merchant_id)
            .header("sign", signature)
            .json(payload)
            .send()
            .await?
            .error_for_status()?
            .json::<T>()
            .await
    }
}

fn compute_signature(api_key: &str, payload: impl AsRef<[u8]>) -> String {
    let data = format!("{}{}", BASE64_STANDARD.encode(payload), api_key);
    let signature = md5(data);
    hex::encode(signature)
}
