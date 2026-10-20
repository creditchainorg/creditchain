//! ML-DSA (FIPS 204) signature verification precompiles, as specified by EIP-8355.
//!
//! | Address | Parameter set | Public key | Signature | Base gas |
//! |---|---|---|---|---|
//! | `0x12` | ML-DSA-44 | 1312 B | 2420 B | 6500 |
//! | `0x13` | ML-DSA-65 | 1952 B | 3309 B | 9000 |
//! | `0x14` | ML-DSA-87 | 2592 B | 4627 B | 13500 |
//!
//! Input is `pubkey ++ signature ++ message`, no length fields; the message is whatever follows
//! the two fixed-size fields and may be empty. Verification is FIPS 204 `ML-DSA.Verify` with the
//! empty context. Output is always exactly 32 bytes: `1` for a valid signature, `0` for anything
//! else — invalid, malformed, or too short. The precompile never reverts; only running out of gas
//! ends a call abnormally. Gas is `base + 6 * ceil(max(0, len - pk - sig) / 32)`, charged in full
//! before any verification work.
//!
//! Returning a fixed 32 bytes lets a contract tell "invalid signature" (32 zero bytes) from "no
//! such precompile on this chain" (empty return data), which matters for code deployed to many
//! chains. `QuantumEscapeRegistry` relies on it.

use alloc::boxed::Box;
use alloy_evm::precompiles::{DynPrecompile, PrecompilesMap};
use alloy_primitives::{address, Address, Bytes};
use ml_dsa::{
    EncodedSignature, EncodedVerifyingKey, MlDsa44, MlDsa65, MlDsa87, MlDsaParams, Signature,
    VerifyingKey,
};
use revm::precompile::{
    EthPrecompileOutput, EthPrecompileResult, PrecompileHalt, PrecompileId, PrecompileOutput,
};

/// Gas per 32-byte word of message, on top of the parameter set's base cost.
pub const GAS_PER_MESSAGE_WORD: u64 = 6;

/// One ML-DSA parameter set and the precompile that verifies it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MlDsaPrecompile {
    /// Precompile address.
    pub address: Address,
    /// EIP-8355 name.
    pub name: &'static str,
    /// Encoded public key length (FIPS 204 `pkEncode`).
    pub pk_len: usize,
    /// Encoded signature length (FIPS 204 `sigEncode`).
    pub sig_len: usize,
    /// Gas charged for any call, before the per-word message cost.
    pub base_gas: u64,
    level: Level,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Level {
    L44,
    L65,
    L87,
}

/// `VERIFY_MLDSA44` at `0x12`.
pub const MLDSA44: MlDsaPrecompile = MlDsaPrecompile {
    address: address!("0x0000000000000000000000000000000000000012"),
    name: "VERIFY_MLDSA44",
    pk_len: 1312,
    sig_len: 2420,
    base_gas: 6500,
    level: Level::L44,
};

/// `VERIFY_MLDSA65` at `0x13`.
pub const MLDSA65: MlDsaPrecompile = MlDsaPrecompile {
    address: address!("0x0000000000000000000000000000000000000013"),
    name: "VERIFY_MLDSA65",
    pk_len: 1952,
    sig_len: 3309,
    base_gas: 9000,
    level: Level::L65,
};

/// `VERIFY_MLDSA87` at `0x14`.
pub const MLDSA87: MlDsaPrecompile = MlDsaPrecompile {
    address: address!("0x0000000000000000000000000000000000000014"),
    name: "VERIFY_MLDSA87",
    pk_len: 2592,
    sig_len: 4627,
    base_gas: 13500,
    level: Level::L87,
};

/// All three, in address order.
pub const ALL: [MlDsaPrecompile; 3] = [MLDSA44, MLDSA65, MLDSA87];

impl MlDsaPrecompile {
    /// Gas for an input of `len` bytes. Depends only on the length, so it can be charged before
    /// any work is done; an input shorter than key + signature costs exactly the base.
    pub const fn gas(&self, len: usize) -> u64 {
        let fixed = self.pk_len + self.sig_len;
        let message = len.saturating_sub(fixed);
        let words = (message as u64).div_ceil(32);
        self.base_gas.saturating_add(GAS_PER_MESSAGE_WORD.saturating_mul(words))
    }

    /// Run the precompile: charge gas, then verify.
    pub fn run(&self, input: &[u8], gas_limit: u64) -> EthPrecompileResult {
        let cost = self.gas(input.len());
        if cost > gas_limit {
            return Err(PrecompileHalt::OutOfGas);
        }
        let mut word = [0u8; 32];
        word[31] = u8::from(self.verify(input));
        Ok(EthPrecompileOutput::new(cost, Bytes::copy_from_slice(&word)))
    }

    /// `true` iff `input` is `pubkey ++ signature ++ message` and the signature is a valid FIPS 204
    /// ML-DSA signature on `message` under `pubkey` with the empty context. Never panics.
    pub fn verify(&self, input: &[u8]) -> bool {
        if input.len() < self.pk_len + self.sig_len {
            return false;
        }
        let (pk, rest) = input.split_at(self.pk_len);
        let (sig, message) = rest.split_at(self.sig_len);
        match self.level {
            Level::L44 => verify_with::<MlDsa44>(pk, sig, message, &[]),
            Level::L65 => verify_with::<MlDsa65>(pk, sig, message, &[]),
            Level::L87 => verify_with::<MlDsa87>(pk, sig, message, &[]),
        }
    }

    /// Wrap as a precompile the EVM can call.
    pub fn dyn_precompile(&self) -> DynPrecompile {
        let this = *self;
        DynPrecompile::new(PrecompileId::custom(this.name), move |input| {
            Ok(PrecompileOutput::from_eth_result(this.run(input.data, input.gas), input.reservoir))
        })
    }
}

/// FIPS 204 `ML-DSA.Verify(pk, M, σ, ctx)` for one parameter set. Malformed encodings (wrong
/// length, out-of-range coefficients, a non-canonical hint) are invalid signatures, not errors.
pub fn verify_with<P: MlDsaParams>(pk: &[u8], sig: &[u8], message: &[u8], ctx: &[u8]) -> bool {
    let Ok(encoded_pk) = EncodedVerifyingKey::<P>::try_from(pk) else { return false };
    let Ok(encoded_sig) = EncodedSignature::<P>::try_from(sig) else { return false };
    let Some(signature) = Signature::<P>::decode(&encoded_sig) else { return false };
    VerifyingKey::<P>::decode(&encoded_pk).verify_with_context(message, ctx, &signature)
}

/// Add the three ML-DSA precompiles to `map`.
pub fn install(map: &mut PrecompilesMap) {
    map.extend_precompiles(ALL.iter().map(|p| (p.address, p.dyn_precompile())));
}

/// Boxed list, for callers that want to inspect what would be installed.
pub fn precompiles() -> Box<[(Address, DynPrecompile)]> {
    ALL.iter().map(|p| (p.address, p.dyn_precompile())).collect()
}
