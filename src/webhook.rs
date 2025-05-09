use crate::crypto::compute_signature;
use axum::http::StatusCode;
use serde::Deserialize;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug)]
pub struct PaymentResult {
    pub order_id: Uuid,
    pub success: bool,
}

pub struct AppState {
    pub api_key: String,
    pub whitelist_ip: IpAddr,
}

#[derive(Deserialize)]
pub struct PaymentUpdate {
    #[serde(flatten)]
    data: serde_json::Value,
    sign: String,
}

#[derive(Deserialize)]
struct PaymentUpdateData {
    r#type: String,
    order_id: Uuid,
    is_final: bool,
    status: String,
}

pub async fn handle_payment_update(
    state: Arc<AppState>,
    addr: SocketAddr,
    payload: PaymentUpdate,
) -> (StatusCode, Option<PaymentResult>) {
    if addr.ip() != state.whitelist_ip {
        return (StatusCode::FORBIDDEN, None);
    }
    let signature = compute_signature(
        &state.api_key,
        serde_json::to_string(&payload.data).unwrap()
            .replace("/", "\\/"),
    );
    if payload.sign != signature {
        return (StatusCode::UNAUTHORIZED, None);
    }
    let payload = match serde_json::from_value::<PaymentUpdateData>(payload.data) {
        Ok(payload) => payload,
        Err(_) => return (StatusCode::BAD_REQUEST, None),
    };
    if payload.r#type != "payment" {
        return (StatusCode::OK, None);
    }
    if !payload.is_final {
        return (StatusCode::OK, None);
    }
    let result = PaymentResult {
        order_id: payload.order_id,
        success: payload.status == "paid" || payload.status == "paid_over",
    };
    (StatusCode::OK, Some(result))
}
