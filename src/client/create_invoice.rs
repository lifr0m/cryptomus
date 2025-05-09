use super::Client;
use rust_decimal::Decimal;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

#[derive(Deserialize)]
struct CreateInvoice {
    result: CreateInvoiceResult,
}

#[derive(Deserialize)]
struct CreateInvoiceResult {
    url: String,
}

impl Client {
    pub async fn create_invoice(
        &self,
        order_id: Uuid,
        usd_amount: Decimal,
        currencies: &[impl AsRef<str>],
        callback_url: &str,
    ) -> reqwest::Result<String> {
        let currencies = currencies.iter()
            .map(|currency| json!({
                "currency": currency.as_ref()
            }))
            .collect::<Vec<_>>();
        let payload = json!({
            "amount": usd_amount,
            "currency": "USD",
            "order_id": order_id,
            "url_callback": callback_url,
            "currencies": currencies,
        });
        Ok(self.request::<CreateInvoice>("/v1/payment", &payload)
            .await?
            .result
            .url)
    }
}
