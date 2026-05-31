//! Faucet configuration — env-driven, with strict validation.

use std::time::Duration;

use alloy_primitives::U256;

#[derive(Debug, Clone)]
pub struct Config {
    pub rpc_url: url::Url,
    pub chain_id: u64,
    pub private_key_hex: String,
    /// Drip amount in **CCC** (display unit, 18 decimals).
    pub drip_amount_ccc: String,
    /// Drip amount in **wei-equivalent base units**.
    pub drip_amount_wei: U256,
    pub per_ip_limit: u32,
    pub per_address_cooldown: Duration,
    pub bind: String,
    pub network_name: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let rpc_url = require_env("FAUCET_RPC_URL")?;
        let rpc_url = url::Url::parse(&rpc_url).map_err(|e| {
            anyhow::anyhow!("FAUCET_RPC_URL is not a valid URL: {e}")
        })?;

        let chain_id: u64 = require_env("FAUCET_CHAIN_ID")?
            .parse()
            .map_err(|e| anyhow::anyhow!("FAUCET_CHAIN_ID must be a decimal integer: {e}"))?;

        let key_file = require_env("FAUCET_PRIVATE_KEY_FILE")?;
        let private_key_hex = std::fs::read_to_string(&key_file)
            .map_err(|e| anyhow::anyhow!("cannot read FAUCET_PRIVATE_KEY_FILE={key_file}: {e}"))?
            .trim()
            .trim_start_matches("0x")
            .to_string();
        if private_key_hex.len() != 64 {
            anyhow::bail!(
                "FAUCET_PRIVATE_KEY_FILE must contain a 32-byte (64 hex char) key, got {}",
                private_key_hex.len()
            );
        }
        if !private_key_hex.chars().all(|c| c.is_ascii_hexdigit()) {
            anyhow::bail!("FAUCET_PRIVATE_KEY_FILE contains non-hex characters");
        }

        let drip_amount_ccc = std::env::var("FAUCET_DRIP_AMOUNT_CCC")
            .unwrap_or_else(|_| "1.0".to_string());
        let drip_amount_wei = parse_ccc_to_wei(&drip_amount_ccc)?;

        let per_ip_limit: u32 = std::env::var("FAUCET_PER_IP_LIMIT")
            .unwrap_or_else(|_| "5".to_string())
            .parse()
            .map_err(|e| anyhow::anyhow!("FAUCET_PER_IP_LIMIT: {e}"))?;

        let per_address_cooldown_secs: u64 = std::env::var("FAUCET_PER_ADDRESS_COOLDOWN")
            .unwrap_or_else(|_| "86400".to_string())
            .parse()
            .map_err(|e| anyhow::anyhow!("FAUCET_PER_ADDRESS_COOLDOWN: {e}"))?;

        let bind = std::env::var("FAUCET_BIND").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
        let network_name =
            std::env::var("FAUCET_NETWORK_NAME").unwrap_or_else(|_| "creditchain".to_string());

        Ok(Self {
            rpc_url,
            chain_id,
            private_key_hex,
            drip_amount_ccc,
            drip_amount_wei,
            per_ip_limit,
            per_address_cooldown: Duration::from_secs(per_address_cooldown_secs),
            bind,
            network_name,
        })
    }
}

fn require_env(key: &str) -> anyhow::Result<String> {
    std::env::var(key).map_err(|_| anyhow::anyhow!("missing required env var: {key}"))
}

/// Parse a decimal CCC amount (e.g. "1.5") into base units (18 decimals).
fn parse_ccc_to_wei(s: &str) -> anyhow::Result<U256> {
    let s = s.trim();
    let (int_part, frac_part) = match s.split_once('.') {
        Some((i, f)) => (i, f),
        None => (s, ""),
    };
    if !int_part.chars().all(|c| c.is_ascii_digit()) {
        anyhow::bail!("amount has non-digit integer part: {s}");
    }
    if !frac_part.chars().all(|c| c.is_ascii_digit()) {
        anyhow::bail!("amount has non-digit fractional part: {s}");
    }
    if frac_part.len() > 18 {
        anyhow::bail!("amount has more than 18 fractional digits: {s}");
    }
    let pad = 18usize.saturating_sub(frac_part.len());
    let combined = format!("{int_part}{frac_part}{}", "0".repeat(pad));
    U256::from_str_radix(&combined, 10).map_err(|e| anyhow::anyhow!("invalid amount '{s}': {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_ccc_round_trips() {
        let one = parse_ccc_to_wei("1").unwrap();
        assert_eq!(one, U256::from(10u64).pow(U256::from(18u64)));
    }

    #[test]
    fn fractional_amount() {
        let half = parse_ccc_to_wei("0.5").unwrap();
        let expected = U256::from(5u64) * U256::from(10u64).pow(U256::from(17u64));
        assert_eq!(half, expected);
    }

    #[test]
    fn rejects_too_many_decimals() {
        let res = parse_ccc_to_wei("1.0000000000000000001"); // 19 decimals
        assert!(res.is_err());
    }

    #[test]
    fn rejects_garbage() {
        assert!(parse_ccc_to_wei("hello").is_err());
        assert!(parse_ccc_to_wei("1.2.3").is_err());
    }
}
