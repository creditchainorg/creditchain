//! HTTP routes for the faucet service.
//!
//! GET  /health     liveness + advertised chain id + balance
//! GET  /info       human-readable info card (network, drip size, limits)
//! POST /drip       drip the configured amount to the requested address

use std::net::SocketAddr;

use alloy_network::{EthereumWallet, TransactionBuilder};
use alloy_primitives::{Address, U256};
use alloy_provider::Provider;
use alloy_rpc_types_eth::TransactionRequest;
use axum::{
    extract::{ConnectInfo, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::limiter::LimiterError;
use crate::state::FaucetState;

pub fn router(state: FaucetState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/info", get(info))
        .route("/drip", post(drip))
        .with_state(state)
}

#[derive(Serialize)]
struct InfoResponse {
    network: String,
    chain_id: u64,
    native_currency: NativeCurrencyResponse,
    rpc_url: String,
    faucet_address: String,
    drip_amount: String,
    drip_amount_base_units: String,
    per_ip_limit_per_hour: u32,
    per_address_cooldown_seconds: u64,
}

#[derive(Serialize)]
struct NativeCurrencyResponse {
    name: String,
    symbol: String,
    decimals: u8,
}

async fn info(State(s): State<FaucetState>) -> Json<InfoResponse> {
    let cfg = s.cfg();
    Json(InfoResponse {
        network: cfg.network_name.clone(),
        chain_id: cfg.chain_id,
        native_currency: NativeCurrencyResponse {
            name: cfg.native_token_name.clone(),
            symbol: cfg.native_token_symbol.clone(),
            decimals: cfg.native_token_decimals,
        },
        rpc_url: cfg.rpc_url.to_string(),
        faucet_address: format!("{:#x}", s.faucet_address()),
        drip_amount: format!("{} {}", cfg.drip_amount, cfg.native_token_symbol),
        drip_amount_base_units: cfg.drip_amount_base_units.to_string(),
        per_ip_limit_per_hour: cfg.per_ip_limit,
        per_address_cooldown_seconds: cfg.per_address_cooldown.as_secs(),
    })
}

#[derive(Serialize)]
struct HealthResponse {
    ok: bool,
    chain_id: u64,
    native_currency: NativeCurrencyResponse,
    faucet_address: String,
    balance: String,
}

async fn health(State(s): State<FaucetState>) -> Result<Json<HealthResponse>, ErrorResponse> {
    let balance = s
        .balance_native()
        .await
        .map_err(|e| ErrorResponse::internal(format!("balance lookup failed: {e}")))?;
    let cfg = s.cfg();
    Ok(Json(HealthResponse {
        ok: true,
        chain_id: cfg.chain_id,
        native_currency: NativeCurrencyResponse {
            name: cfg.native_token_name.clone(),
            symbol: cfg.native_token_symbol.clone(),
            decimals: cfg.native_token_decimals,
        },
        faucet_address: format!("{:#x}", s.faucet_address()),
        balance,
    }))
}

#[derive(Deserialize)]
struct DripRequest {
    address: String,
}

#[derive(Serialize)]
struct DripResponse {
    network: String,
    chain_id: u64,
    tx: String,
    to: String,
    amount: String,
    amount_base_units: String,
}

async fn drip(
    State(s): State<FaucetState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(req): Json<DripRequest>,
) -> Result<Json<DripResponse>, ErrorResponse> {
    let to: Address = req
        .address
        .parse()
        .map_err(|_| ErrorResponse::bad_request("address is not a valid EVM address"))?;
    if to == Address::ZERO {
        return Err(ErrorResponse::bad_request("address cannot be the zero address"));
    }

    let ip = addr.ip();
    s.limiter().check_and_record(ip, to).await.map_err(ErrorResponse::from_limiter)?;

    let value: U256 = s.cfg().drip_amount_base_units;
    let chain_id = s.cfg().chain_id;

    // Build, sign, broadcast. The provider has no fillers, so nonce/gas/fees
    // must be set explicitly or TransactionBuilder::build refuses the request.
    let nonce = s
        .provider()
        .get_transaction_count(s.faucet_address())
        .pending()
        .await
        .map_err(|e| ErrorResponse::internal(format!("nonce fetch failed: {e}")))?;
    let fees = s
        .provider()
        .estimate_eip1559_fees(None)
        .await
        .map_err(|e| ErrorResponse::internal(format!("fee estimate failed: {e}")))?;

    let tx = TransactionRequest::default()
        .with_from(s.faucet_address())
        .with_to(to)
        .with_value(value)
        .with_chain_id(chain_id)
        .with_nonce(nonce)
        // Plain value transfer costs exactly 21k gas.
        .with_gas_limit(21_000)
        .with_max_fee_per_gas(fees.max_fee_per_gas)
        .with_max_priority_fee_per_gas(fees.max_priority_fee_per_gas);

    let wallet: EthereumWallet = s.wallet();
    let envelope = tx
        .build(&wallet)
        .await
        .map_err(|e| ErrorResponse::internal(format!("tx build failed: {e}")))?;

    let pending = s
        .provider()
        .send_tx_envelope(envelope)
        .await
        .map_err(|e| ErrorResponse::internal(format!("tx submit failed: {e}")))?;
    let tx_hash = *pending.tx_hash();

    tracing::info!(
        %ip,
        to = %req.address,
        tx = %tx_hash,
        amount = %s.cfg().drip_amount,
        symbol = %s.cfg().native_token_symbol,
        "drip sent"
    );

    Ok(Json(DripResponse {
        network: s.cfg().network_name.clone(),
        chain_id,
        tx: format!("{:#x}", tx_hash),
        to: format!("{:#x}", to),
        amount: format!("{} {}", s.cfg().drip_amount, s.cfg().native_token_symbol),
        amount_base_units: value.to_string(),
    }))
}

// ─── error shape ────────────────────────────────────────────────────────────

#[derive(Serialize)]
struct ErrorBody {
    code: &'static str,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    retry_after_seconds: Option<u64>,
}

struct ErrorResponse {
    status: StatusCode,
    body: ErrorBody,
}

impl ErrorResponse {
    fn bad_request(msg: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            body: ErrorBody {
                code: "faucet.bad_request",
                message: msg.into(),
                retry_after_seconds: None,
            },
        }
    }

    fn internal(msg: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            body: ErrorBody {
                code: "faucet.internal",
                message: msg.into(),
                retry_after_seconds: None,
            },
        }
    }

    fn from_limiter(e: LimiterError) -> Self {
        match e {
            LimiterError::IpExceeded { hits, max } => Self {
                status: StatusCode::TOO_MANY_REQUESTS,
                body: ErrorBody {
                    code: "faucet.ip_rate_limited",
                    message: format!(
                        "ip rate limit exceeded: {hits} drips in last hour (max {max})"
                    ),
                    retry_after_seconds: Some(60 * 60),
                },
            },
            LimiterError::AddressCooldown { seconds_remaining } => Self {
                status: StatusCode::TOO_MANY_REQUESTS,
                body: ErrorBody {
                    code: "faucet.address_cooldown",
                    message: format!("address cooldown active: try again in {seconds_remaining}s"),
                    retry_after_seconds: Some(seconds_remaining),
                },
            },
        }
    }
}

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> axum::response::Response {
        (self.status, Json(self.body)).into_response()
    }
}
