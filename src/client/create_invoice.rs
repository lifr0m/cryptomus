use super::Client;
use rust_decimal::Decimal;
use serde::Deserialize;
use serde_json::json;

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
        order_id: &str,
        amount: Decimal,
        currencies: &[String],
    ) -> reqwest::Result<String> {
        let currencies = currencies.iter()
            .map(|currency| json!({
                "currency": currency
            }))
            .collect::<Vec<_>>();
        let payload = json!({
            "amount": amount,
            "currency": "USD",
            "order_id": order_id,
            "url_callback": self.config.callback_url,
            "currencies": currencies,
        });
        Ok(self.request::<CreateInvoice>("/v1/payment", &payload)
            .await?
            .result
            .url)
    }
}
