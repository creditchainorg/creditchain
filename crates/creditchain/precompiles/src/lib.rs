//! `CreditChain`'s precompiles.
//!
//! Contracts the chain adds to Ethereum's precompile set, each switched on by a fork scheduled in
//! the chain's genesis config (see [`schedule`]). Today that is one upgrade: ML-DSA (FIPS 204)
//! signature verification at `0x12`–`0x14`, specified by EIP-8355, so that contracts can verify
//! post-quantum signatures — from HSMs, cloud key services or air-gapped wallets — for a few
//! thousand gas instead of millions.
//!
//! The EVM factory calls [`apply`] for every EVM it creates, so block execution, block building,
//! `eth_call` and offline re-execution all see the same precompiles at the same block.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod mldsa;
pub mod schedule;

use alloy_evm::precompiles::PrecompilesMap;

/// Add every `CreditChain` precompile active on `chain_id` for a block at `timestamp`.
pub fn apply(map: &mut PrecompilesMap, chain_id: u64, timestamp: u64) {
    if schedule::mldsa_active(chain_id, timestamp) {
        mldsa::install(map);
    }
}
