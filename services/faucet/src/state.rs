//! Shared faucet state: provider, signer, and the in-memory rate limiter.
//!
//! Held in an `Arc` for free cloning into axum handlers.

use std::sync::Arc;

use alloy_network::EthereumWallet;
use alloy_primitives::{Address, U256};
use alloy_provider::{Provider, ProviderBuilder};
use alloy_signer_local::PrivateKeySigner;

use crate::{
    config::{format_base_units, Config},
    limiter::RateLimiter,
};

#[derive(Clone)]
pub struct FaucetState {
    inner: Arc<Inner>,
}

struct Inner {
    cfg: Config,
    signer: PrivateKeySigner,
    provider: alloy_provider::RootProvider<alloy_transport_http::Http<reqwest::Client>>,
    limiter: RateLimiter,
}

impl FaucetState {
    pub async fn connect(cfg: Config) -> anyhow::Result<Self> {
        let signer: PrivateKeySigner = cfg
            .private_key_hex
            .parse()
            .map_err(|e| anyhow::anyhow!("private key parse failed: {e}"))?;

        let provider = ProviderBuilder::new().on_http(cfg.rpc_url.clone());

        let limiter = RateLimiter::new(cfg.per_ip_limit, cfg.per_address_cooldown);
        Ok(Self { inner: Arc::new(Inner { cfg, signer, provider, limiter }) })
    }

    pub fn cfg(&self) -> &Config {
        &self.inner.cfg
    }

    pub fn faucet_address(&self) -> Address {
        self.inner.signer.address()
    }

    pub fn wallet(&self) -> EthereumWallet {
        EthereumWallet::from(self.inner.signer.clone())
    }

    pub fn provider(
        &self,
    ) -> &alloy_provider::RootProvider<alloy_transport_http::Http<reqwest::Client>> {
        &self.inner.provider
    }

    pub fn limiter(&self) -> &RateLimiter {
        &self.inner.limiter
    }

    /// Compare configured chain id against the RPC's `eth_chainId`. Hard-fails
    /// at startup if they diverge — saves us from dripping testnet funds to a
    /// devnet RPC mistakenly listed under the testnet env.
    pub async fn assert_chain_id_matches(&self) -> anyhow::Result<()> {
        let advertised: u64 = self
            .inner
            .provider
            .get_chain_id()
            .await
            .map_err(|e| anyhow::anyhow!("eth_chainId failed: {e}"))?;
        if advertised != self.inner.cfg.chain_id {
            anyhow::bail!(
                "RPC chain id mismatch: configured FAUCET_CHAIN_ID={} but RPC reports {}",
                self.inner.cfg.chain_id,
                advertised
            );
        }
        tracing::info!(chain_id = advertised, "RPC chain id matches configured value");
        Ok(())
    }

    /// Faucet balance in decimal native-token units, for /health and /info.
    pub async fn balance_native(&self) -> anyhow::Result<String> {
        let bal: U256 = self
            .inner
            .provider
            .get_balance(self.faucet_address())
            .await
            .map_err(|e| anyhow::anyhow!("eth_getBalance failed: {e}"))?;
        Ok(format_base_units(bal, self.inner.cfg.native_token_decimals))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::format_base_units;

    #[test]
    fn format_one_native_token() {
        let one = U256::from(10u64).pow(U256::from(18u64));
        assert_eq!(format_base_units(one, 18), "1.0");
    }

    #[test]
    fn format_half_native_token() {
        let half = U256::from(5u64) * U256::from(10u64).pow(U256::from(17u64));
        assert_eq!(format_base_units(half, 18), "0.5");
    }

    #[test]
    fn format_zero() {
        assert_eq!(format_base_units(U256::ZERO, 18), "0.0");
    }

    #[test]
    fn format_large() {
        let v = U256::from(123_456_789u64) * U256::from(10u64).pow(U256::from(18u64));
        assert_eq!(format_base_units(v, 18), "123456789.0");
    }
}
