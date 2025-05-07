use super::{compute_signature, Client};
use axum::extract::{ConnectInfo, State};
use axum::http::StatusCode;
use axum::routing::post;
use axum::{Json, Router};
use serde::Deserialize;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

struct AppState<C, CFut>
where
    C: Fn(String, bool) -> CFut,
    CFut: Future<Output = ()>,
{
    api_key: String,
    client_ip: IpAddr,
    callback: C,
}

#[derive(Deserialize)]
struct PaymentUpdate {
    #[serde(flatten)]
    data: serde_json::Value,
    sign: String,
}

#[derive(Deserialize)]
struct PaymentUpdateData {
    r#type: String,
    order_id: String,
    is_final: bool,
    status: String,
}

impl Client {
    pub async fn serve_webhook<C, CFut>(
        &self,
        host: &str,
        client_ip: IpAddr,
        callback: C,
    ) -> std::io::Result<()>
    where
        C: Fn(String, bool) -> CFut + Send + Sync + 'static,
        CFut: Future<Output = ()> + Send + 'static,
    {
        let api_key = self.config.api_key.clone();
        let shared_state = Arc::new(AppState { api_key, client_ip, callback });
        let app = Router::new()
            .route("/", post(handle_payment_update))
            .with_state(shared_state);
        let listener = tokio::net::TcpListener::bind((host, 80)).await?;
        axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await
    }
}

async fn handle_payment_update<C, CFut>(
    State(state): State<Arc<AppState<C, CFut>>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(payload): Json<PaymentUpdate>,
) -> StatusCode
where
    C: Fn(String, bool) -> CFut,
    CFut: Future<Output = ()>,
{
    if state.client_ip != addr.ip() {
        return StatusCode::FORBIDDEN;
    }
    let signature = compute_signature(
        &state.api_key,
        serde_json::to_string(&payload.data).unwrap()
            .replace("/", "\\/"),
    );
    if payload.sign != signature {
        return StatusCode::UNAUTHORIZED;
    }
    let payload = match serde_json::from_value::<PaymentUpdateData>(payload.data) {
        Ok(value) => value,
        Err(_) => return StatusCode::BAD_REQUEST,
    };
    if payload.r#type != "payment" {
        return StatusCode::NO_CONTENT;
    }
    if !payload.is_final {
        return StatusCode::NO_CONTENT;
    }
    if payload.status == "paid" || payload.status == "paid_over" {
        (state.callback)(payload.order_id, true).await;
    } else {
        (state.callback)(payload.order_id, false).await;
    }
    StatusCode::NO_CONTENT
}
