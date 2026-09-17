//! CreditChain Faucet
//!
//! Drips the configured native token to any requested EVM address under per-IP
//! + per-recipient rate limits. CreditChain defaults to CCC; enterprise forks
//! can set FAUCET_NATIVE_TOKEN_* env vars without changing faucet code. The
//! faucet validates the configured chain id matches the RPC's
//! advertised chain id at startup so misconfigured deploys fail loudly
//! rather than dripping to the wrong network.
//!
//! Routes:
//!   POST /drip   { "address": "0x..." }    → { "tx": "0x...", "amount": "1.0 CCC", ... }
//!   GET  /health                            → { "ok": true, "balance": "...", "chain_id": ... }
//!   GET  /info                              → { drip_amount, faucet_address, per_ip_limit, ... }
//!
//! Configuration (env, all required unless marked optional):
//!   FAUCET_RPC_URL               — JSON-RPC HTTP URL of a CreditChain node
//!   FAUCET_CHAIN_ID              — decimal chain id (must match RPC's eth_chainId)
//!   FAUCET_PRIVATE_KEY_FILE      — path to a file containing the 32-byte hex private key (no `0x`)
//!   FAUCET_NATIVE_TOKEN_NAME     — display name (default "CreditChain Token")
//!   FAUCET_NATIVE_TOKEN_SYMBOL   — gas token symbol (default "CCC")
//!   FAUCET_NATIVE_TOKEN_DECIMALS — native token decimals (default 18)
//!   FAUCET_DRIP_AMOUNT           — decimal native token per drip (default "1.0")
//!   FAUCET_DRIP_AMOUNT_CCC       — backwards-compatible alias
//!   FAUCET_PER_IP_LIMIT          — drips per IP per hour (default 5)
//!   FAUCET_PER_ADDRESS_COOLDOWN  — seconds before the same recipient may drip again (default
//! 86400)   FAUCET_BIND                  — listen address (default 0.0.0.0:8080)
//!   FAUCET_NETWORK_NAME          — display label (default "creditchain"); used in /info + logs
//!
//! Hardening notes:
//!   - The private key never leaves memory; we never log it.
//!   - Tower-http imposes a 1MB request body limit + 30s timeout.
//!   - Rate-limits are sliding-window in-memory; sufficient for a single instance. For
//!     multi-instance deployments, front with sticky LB or swap `RateLimiter` for a Redis-backed
//!     implementation.

mod config;
mod limiter;
mod routes;
mod state;

use std::{net::SocketAddr, time::Duration};

use axum::http::StatusCode;
use tower_http::{
    cors::{Any, CorsLayer},
    limit::RequestBodyLimitLayer,
    timeout::TimeoutLayer,
    trace::TraceLayer,
};

use crate::{config::Config, state::FaucetState};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    let cfg = Config::from_env()?;
    tracing::info!(
        chain_id = cfg.chain_id,
        rpc = %cfg.rpc_url,
        network = %cfg.network_name,
        bind = %cfg.bind,
        "starting CreditChain faucet"
    );

    let state = FaucetState::connect(cfg.clone()).await?;
    state.assert_chain_id_matches().await?;

    tracing::info!(
        faucet = %state.faucet_address(),
        balance = %state.balance_native().await?,
        symbol = %state.cfg().native_token_symbol,
        "faucet ready"
    );

    let app = routes::router(state)
        .layer(RequestBodyLimitLayer::new(1024 * 1024))
        .layer(TimeoutLayer::with_status_code(StatusCode::REQUEST_TIMEOUT, Duration::from_secs(30)))
        .layer(CorsLayer::new().allow_methods(Any).allow_headers(Any).allow_origin(Any))
        .layer(TraceLayer::new_for_http());

    let bind: SocketAddr = cfg.bind.parse()?;
    let listener = tokio::net::TcpListener::bind(bind).await?;
    tracing::info!(%bind, "listening");

    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

fn init_tracing() {
    use tracing_subscriber::EnvFilter;
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,tower_http=info,creditchain_faucet=debug"));
    tracing_subscriber::fmt().with_env_filter(filter).with_target(false).compact().init();
}

async fn shutdown_signal() {
    use tokio::signal;
    let ctrl_c = async {
        let _ = signal::ctrl_c().await;
    };
    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("install SIGTERM handler")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! { _ = ctrl_c => {}, _ = terminate => {} }
    tracing::info!("shutting down");
}
