//! When `CreditChain`'s own precompiles switch on.
//!
//! Like every network upgrade on `CreditChain`, this is configuration, never a new genesis
//! (`deploy/NETWORK-UPGRADES.md`). A chain's genesis file schedules it in its `config` object:
//!
//! ```json
//! "config": { "chainId": 2026042404, …, "creditchain": { "mlDsaTime": 1767225600 } }
//! ```
//!
//! The field changes no genesis hash: only the header and allocation do. The chain-spec parser
//! reads it, adds [`MlDsaFork`] to the chain's hardforks — so the fork id (EIP-2124 / EIP-6122)
//! announces it and a node that has not upgraded is disconnected at activation instead of
//! silently following a different chain — and records it here, where the EVM factory looks it up
//! for every block it executes, builds or simulates.

use alloy_genesis::Genesis;
use alloy_hardforks::Hardfork;

/// Key of `CreditChain`'s object inside the genesis `config`.
pub const CONFIG_KEY: &str = "creditchain";
/// Field holding the ML-DSA precompiles' activation time (Unix seconds).
pub const MLDSA_TIME_KEY: &str = "mlDsaTime";

/// The `CreditChain` network upgrade that adds the EIP-8355 ML-DSA precompiles.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MlDsaFork;

impl Hardfork for MlDsaFork {
    fn name(&self) -> &'static str {
        "CreditChainMlDsa"
    }
}

/// The scheduled activation time in a genesis file, if any.
///
/// A present but malformed value is an error, not "unscheduled": a typo must not quietly leave a
/// node without a fork the rest of the network activates.
pub fn mldsa_time(genesis: &Genesis) -> Result<Option<u64>, alloc::string::String> {
    let Some(section) = genesis.config.extra_fields.get(CONFIG_KEY) else { return Ok(None) };
    let Some(object) = section.as_object() else {
        return Err(alloc::format!("genesis config.{CONFIG_KEY} must be an object"));
    };
    match object.get(MLDSA_TIME_KEY) {
        None => Ok(None),
        Some(value) => value.as_u64().map(Some).ok_or_else(|| {
            alloc::format!(
                "genesis config.{CONFIG_KEY}.{MLDSA_TIME_KEY} must be a non-negative integer (Unix seconds)"
            )
        }),
    }
}

#[cfg(feature = "std")]
mod registry {
    use std::sync::RwLock;

    /// (chain id, activation time). Written once per chain at start-up, read for every EVM.
    static MLDSA: RwLock<Vec<(u64, u64)>> = RwLock::new(Vec::new());

    pub(super) fn register(chain_id: u64, time: u64) {
        let mut entries = MLDSA.write().unwrap_or_else(|e| e.into_inner());
        entries.retain(|(id, _)| *id != chain_id);
        entries.push((chain_id, time));
    }

    pub(super) fn active(chain_id: u64, timestamp: u64) -> bool {
        let entries = MLDSA.read().unwrap_or_else(|e| e.into_inner());
        entries.iter().any(|&(id, time)| id == chain_id && timestamp >= time)
    }
}

/// Record that `chain_id` activates the ML-DSA precompiles at `time`.
pub fn register_mldsa(chain_id: u64, time: u64) {
    #[cfg(feature = "std")]
    registry::register(chain_id, time);
    #[cfg(not(feature = "std"))]
    let _ = (chain_id, time);
}

/// Are the ML-DSA precompiles active on `chain_id` for a block at `timestamp`?
pub fn mldsa_active(chain_id: u64, timestamp: u64) -> bool {
    #[cfg(feature = "std")]
    return registry::active(chain_id, timestamp);
    #[cfg(not(feature = "std"))]
    {
        let _ = (chain_id, timestamp);
        false
    }
}
