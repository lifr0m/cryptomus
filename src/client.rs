mod create_invoice;

use crate::crypto::compute_signature;
use serde::de::DeserializeOwned;

#[derive(Clone)]
pub struct Client {
    client: reqwest::Client,
    merchant_id: String,
    api_key: String,
}

impl Client {
    pub fn new(merchant_id: String, api_key: String) -> Self {
        let client = reqwest::Client::new();

        Self { client, merchant_id, api_key }
    }

    async fn request<T>(&self, endpoint: &str, payload: &serde_json::Value) -> reqwest::Result<T>
    where
        T: DeserializeOwned,
    {
        let signature = compute_signature(
            &self.api_key,
            serde_json::to_vec(payload).unwrap(),
        );
        self.client.post(format!("https://api.cryptomus.com{endpoint}"))
            .header("merchant", &self.merchant_id)
            .header("sign", signature)
            .json(payload)
            .send()
            .await?
            .error_for_status()?
            .json::<T>()
            .await
    }
}
