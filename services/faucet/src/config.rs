//! Faucet configuration — env-driven, with strict validation.

use std::time::Duration;

use alloy_primitives::U256;

#[derive(Debug, Clone)]
pub struct Config {
    /// Where the faucet sends transactions. Often an internal address; never published.
    pub rpc_url: url::Url,
    /// The RPC URL `/info` tells users about. Unset means `/info` names no RPC at all.
    pub public_rpc_url: Option<url::Url>,
    pub chain_id: u64,
    pub private_key_hex: String,
    pub native_token_name: String,
    pub native_token_symbol: String,
    pub native_token_decimals: u8,
    /// Drip amount in native display units.
    pub drip_amount: String,
    /// Drip amount in native base units.
    pub drip_amount_base_units: U256,
    pub per_ip_limit: u32,
    pub per_address_cooldown: Duration,
    pub bind: String,
    pub network_name: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let rpc_url = require_env("FAUCET_RPC_URL")?;
        let rpc_url = url::Url::parse(&rpc_url)
            .map_err(|e| anyhow::anyhow!("FAUCET_RPC_URL is not a valid URL: {e}"))?;

        let public_rpc_url = match std::env::var("FAUCET_PUBLIC_RPC_URL") {
            Ok(v) if !v.trim().is_empty() => Some(url::Url::parse(v.trim()).map_err(|e| {
                anyhow::anyhow!("FAUCET_PUBLIC_RPC_URL is not a valid URL: {e}")
            })?),
            _ => None,
        };

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

        let native_token_name = std::env::var("FAUCET_NATIVE_TOKEN_NAME")
            .unwrap_or_else(|_| "CreditChain Token".to_string());
        let native_token_symbol =
            std::env::var("FAUCET_NATIVE_TOKEN_SYMBOL").unwrap_or_else(|_| "CCC".to_string());
        validate_symbol(&native_token_symbol)?;
        let native_token_decimals: u8 = std::env::var("FAUCET_NATIVE_TOKEN_DECIMALS")
            .unwrap_or_else(|_| "18".to_string())
            .parse()
            .map_err(|e| anyhow::anyhow!("FAUCET_NATIVE_TOKEN_DECIMALS: {e}"))?;
        if native_token_decimals > 36 {
            anyhow::bail!("FAUCET_NATIVE_TOKEN_DECIMALS must be <= 36");
        }

        let drip_amount = std::env::var("FAUCET_DRIP_AMOUNT")
            .or_else(|_| std::env::var("FAUCET_DRIP_AMOUNT_CCC"))
            .unwrap_or_else(|_| "1.0".to_string());
        let drip_amount_base_units =
            parse_decimal_to_base_units(&drip_amount, native_token_decimals)?;

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
            public_rpc_url,
            chain_id,
            private_key_hex,
            native_token_name,
            native_token_symbol,
            native_token_decimals,
            drip_amount,
            drip_amount_base_units,
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

fn validate_symbol(symbol: &str) -> anyhow::Result<()> {
    if symbol.is_empty() || symbol.len() > 12 {
        anyhow::bail!("FAUCET_NATIVE_TOKEN_SYMBOL must be 1-12 characters");
    }
    if !symbol.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit()) {
        anyhow::bail!("FAUCET_NATIVE_TOKEN_SYMBOL must use A-Z and 0-9 only");
    }
    Ok(())
}

/// Parse a decimal native-token amount (e.g. "1.5") into base units.
pub fn parse_decimal_to_base_units(s: &str, decimals: u8) -> anyhow::Result<U256> {
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
    if frac_part.len() > decimals as usize {
        anyhow::bail!("amount has more than {decimals} fractional digits: {s}");
    }
    let pad = decimals as usize - frac_part.len();
    let combined = format!("{int_part}{frac_part}{}", "0".repeat(pad));
    U256::from_str_radix(&combined, 10).map_err(|e| anyhow::anyhow!("invalid amount '{s}': {e}"))
}

pub fn format_base_units(value: U256, decimals: u8) -> String {
    let unit = U256::from(10u64).pow(U256::from(decimals));
    let int_part = value / unit;
    let frac_part = value % unit;
    if decimals == 0 {
        return int_part.to_string();
    }
    let frac_str = format!("{:0width$}", frac_part, width = decimals as usize);
    let trimmed = frac_str.trim_end_matches('0');
    if trimmed.is_empty() {
        format!("{int_part}.0")
    } else {
        format!("{int_part}.{trimmed}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_native_token_round_trips() {
        let one = parse_decimal_to_base_units("1", 18).unwrap();
        assert_eq!(one, U256::from(10u64).pow(U256::from(18u64)));
    }

    #[test]
    fn fractional_amount() {
        let half = parse_decimal_to_base_units("0.5", 18).unwrap();
        let expected = U256::from(5u64) * U256::from(10u64).pow(U256::from(17u64));
        assert_eq!(half, expected);
    }

    #[test]
    fn rejects_too_many_decimals() {
        let res = parse_decimal_to_base_units("1.0000000000000000001", 18); // 19 decimals
        assert!(res.is_err());
    }

    #[test]
    fn rejects_garbage() {
        assert!(parse_decimal_to_base_units("hello", 18).is_err());
        assert!(parse_decimal_to_base_units("1.2.3", 18).is_err());
    }

    #[test]
    fn supports_custom_decimals() {
        assert_eq!(parse_decimal_to_base_units("1.25", 6).unwrap(), U256::from(1_250_000));
        assert_eq!(format_base_units(U256::from(1_250_000), 6), "1.25");
    }

    #[test]
    fn validates_symbols() {
        assert!(validate_symbol("CCC").is_ok());
        assert!(validate_symbol("BANK1").is_ok());
        assert!(validate_symbol("bad").is_err());
        assert!(validate_symbol("").is_err());
    }
}
