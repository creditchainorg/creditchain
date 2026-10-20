//! ML-DSA precompile tests against NIST's ACVP vectors and against signatures from `OpenSSL`, an
//! implementation independent of the one under test.

use alloy_primitives::hex;
use cc_precompiles::mldsa::{verify_with, MlDsaPrecompile, ALL, MLDSA44, MLDSA65, MLDSA87};
use ml_dsa::{MlDsa44, MlDsa65, MlDsa87};
use revm::precompile::PrecompileHalt;
use serde::Deserialize;

#[derive(Deserialize)]
struct Acvp {
    tests: Vec<AcvpTest>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AcvpTest {
    tc_id: u64,
    parameter_set: String,
    pk: String,
    message: String,
    context: String,
    signature: String,
    valid: bool,
    reason: String,
}

#[derive(Deserialize)]
struct Ossl {
    tests: Vec<OsslTest>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct OsslTest {
    parameter_set: String,
    pk: String,
    message: String,
    signature: String,
}

fn acvp() -> Vec<AcvpTest> {
    serde_json::from_str::<Acvp>(include_str!("../testdata/acvp-mldsa-sigver-external-pure.json"))
        .unwrap()
        .tests
}

fn openssl() -> Vec<OsslTest> {
    serde_json::from_str::<Ossl>(include_str!("../testdata/openssl-mldsa.json")).unwrap().tests
}

fn precompile_for(set: &str) -> MlDsaPrecompile {
    match set {
        "ML-DSA-44" => MLDSA44,
        "ML-DSA-65" => MLDSA65,
        "ML-DSA-87" => MLDSA87,
        other => panic!("unknown parameter set {other}"),
    }
}

fn input(pk: &[u8], sig: &[u8], msg: &[u8]) -> Vec<u8> {
    [pk, sig, msg].concat()
}

/// Calls the precompile with plenty of gas and returns the 32-byte answer as a bool.
fn call(p: &MlDsaPrecompile, data: &[u8]) -> bool {
    let out = p.run(data, u64::MAX).expect("never halts with enough gas");
    assert_eq!(out.bytes.len(), 32, "always exactly 32 bytes");
    assert!(out.bytes[..31].iter().all(|b| *b == 0));
    match out.bytes[31] {
        0 => false,
        1 => true,
        other => panic!("answer must be 0 or 1, got {other}"),
    }
}

#[test]
fn acvp_every_vector_with_its_context() {
    let tests = acvp();
    assert_eq!(tests.len(), 45);
    for t in &tests {
        let (pk, sig, msg, ctx) = (
            hex::decode(&t.pk).unwrap(),
            hex::decode(&t.signature).unwrap(),
            hex::decode(&t.message).unwrap(),
            hex::decode(&t.context).unwrap(),
        );
        let got = match t.parameter_set.as_str() {
            "ML-DSA-44" => verify_with::<MlDsa44>(&pk, &sig, &msg, &ctx),
            "ML-DSA-65" => verify_with::<MlDsa65>(&pk, &sig, &msg, &ctx),
            "ML-DSA-87" => verify_with::<MlDsa87>(&pk, &sig, &msg, &ctx),
            other => panic!("{other}"),
        };
        assert_eq!(got, t.valid, "tcId {} ({}): {}", t.tc_id, t.parameter_set, t.reason);
    }
}

#[test]
fn acvp_empty_context_vectors_through_the_precompile() {
    let tests: Vec<_> = acvp().into_iter().filter(|t| t.context.is_empty()).collect();
    assert!(!tests.is_empty());
    for t in tests {
        let p = precompile_for(&t.parameter_set);
        let data = input(
            &hex::decode(&t.pk).unwrap(),
            &hex::decode(&t.signature).unwrap(),
            &hex::decode(&t.message).unwrap(),
        );
        assert_eq!(call(&p, &data), t.valid, "tcId {}: {}", t.tc_id, t.reason);
    }
}

#[test]
fn acvp_signatures_with_a_context_do_not_verify_without_it() {
    // The precompile always uses the empty context; a signature made under any other context
    // must not be accepted, or a signature for one application could be replayed in another.
    for t in acvp().into_iter().filter(|t| t.valid && !t.context.is_empty()) {
        let p = precompile_for(&t.parameter_set);
        let data = input(
            &hex::decode(&t.pk).unwrap(),
            &hex::decode(&t.signature).unwrap(),
            &hex::decode(&t.message).unwrap(),
        );
        assert!(!call(&p, &data), "tcId {} verified without its context", t.tc_id);
    }
}

#[test]
fn openssl_signatures_verify_and_tampering_does_not() {
    let tests = openssl();
    assert_eq!(tests.len(), 3);
    for t in tests {
        let p = precompile_for(&t.parameter_set);
        let (pk, sig, msg) = (
            hex::decode(&t.pk).unwrap(),
            hex::decode(&t.signature).unwrap(),
            hex::decode(&t.message).unwrap(),
        );
        assert_eq!(pk.len(), p.pk_len);
        assert_eq!(sig.len(), p.sig_len);
        assert!(call(&p, &input(&pk, &sig, &msg)), "{} OpenSSL signature", t.parameter_set);

        let mut m = msg.clone();
        m[17] ^= 0x40;
        assert!(!call(&p, &input(&pk, &sig, &m)), "tampered message");

        for byte in [0, 100, sig.len() - 1] {
            let mut s = sig.clone();
            s[byte] ^= 0x01;
            assert!(!call(&p, &input(&pk, &s, &msg)), "tampered signature byte {byte}");
        }

        let mut k = pk.clone();
        k[40] ^= 0x01;
        assert!(!call(&p, &input(&k, &sig, &msg)), "tampered key");

        let mut longer = msg.clone();
        longer.push(0);
        assert!(!call(&p, &input(&pk, &sig, &longer)), "extra trailing byte");

        for other in ALL.iter().filter(|o| o.address != p.address) {
            assert!(!call(other, &input(&pk, &sig, &msg)), "accepted at the wrong level");
        }
    }
}

#[test]
fn short_input_is_a_charged_zero_not_a_revert() {
    for p in ALL {
        for len in [0, 1, 32, p.pk_len, p.pk_len + p.sig_len - 1] {
            let out = p.run(&vec![0u8; len], p.base_gas).unwrap();
            assert_eq!(out.gas_used, p.base_gas);
            assert_eq!(out.bytes.as_ref(), [0u8; 32].as_slice());
        }
    }
}

#[test]
fn gas_follows_eip_8355() {
    for p in ALL {
        let fixed = p.pk_len + p.sig_len;
        assert_eq!(p.gas(fixed), p.base_gas, "empty message");
        assert_eq!(p.gas(fixed + 1), p.base_gas + 6);
        assert_eq!(p.gas(fixed + 32), p.base_gas + 6);
        assert_eq!(p.gas(fixed + 33), p.base_gas + 12);
        assert_eq!(p.gas(fixed + 32 * 1000), p.base_gas + 6000);
        assert!(p.gas(usize::MAX) > p.base_gas, "saturates instead of overflowing");
    }
    assert_eq!((MLDSA44.base_gas, MLDSA65.base_gas, MLDSA87.base_gas), (6500, 9000, 13500));
}

#[test]
fn out_of_gas_halts_before_any_work() {
    let t = &openssl()[0];
    let data = input(
        &hex::decode(&t.pk).unwrap(),
        &hex::decode(&t.signature).unwrap(),
        &hex::decode(&t.message).unwrap(),
    );
    let cost = MLDSA44.gas(data.len());
    assert_eq!(MLDSA44.run(&data, cost - 1).unwrap_err(), PrecompileHalt::OutOfGas);
    assert_eq!(MLDSA44.run(&data, cost).unwrap().gas_used, cost);
}

#[test]
fn addresses_and_sizes() {
    assert_eq!(
        MLDSA44.address,
        alloy_primitives::address!("0x0000000000000000000000000000000000000012")
    );
    assert_eq!(
        MLDSA65.address,
        alloy_primitives::address!("0x0000000000000000000000000000000000000013")
    );
    assert_eq!(
        MLDSA87.address,
        alloy_primitives::address!("0x0000000000000000000000000000000000000014")
    );
    assert_eq!((MLDSA44.pk_len, MLDSA44.sig_len), (1312, 2420));
    assert_eq!((MLDSA65.pk_len, MLDSA65.sig_len), (1952, 3309));
    assert_eq!((MLDSA87.pk_len, MLDSA87.sig_len), (2592, 4627));
}

/// A precompile that panics takes the node down with it, so it must not panic on any input.
/// Deterministic xorshift so a failure reproduces.
#[test]
fn never_panics_on_arbitrary_input() {
    let mut x: u64 = 0x9E37_79B9_7F4A_7C15;
    let mut next = || {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        x
    };
    let valid = openssl();
    for (i, p) in ALL.iter().enumerate() {
        let pk = hex::decode(&valid[i].pk).unwrap();
        let fixed = p.pk_len + p.sig_len;
        for round in 0..400 {
            let len = match round % 4 {
                0 => fixed,
                1 => fixed + (next() % 200) as usize,
                2 => (next() as usize) % fixed,
                _ => fixed + 1 + (next() % 4096) as usize,
            };
            let mut data: Vec<u8> = (0..len).map(|_| next() as u8).collect();
            // Half the time, a real public key with a random signature: exercises decoding of
            // hints and z coefficients rather than just failing on the key.
            if round % 2 == 0 && len >= p.pk_len {
                data[..p.pk_len].copy_from_slice(&pk);
            }
            let result = std::panic::catch_unwind(|| p.run(&data, u64::MAX));
            let out =
                result.unwrap_or_else(|_| panic!("{} panicked on round {round}", p.name)).unwrap();
            assert_eq!(out.bytes.len(), 32);
            assert_eq!(out.bytes[31], 0, "random bytes verified");
        }
    }
}

/// Timing of a valid verification per level next to ecrecover, to sanity-check EIP-8355's prices.
/// `cargo test -p cc-precompiles --release -- --ignored --nocapture timing`
#[test]
#[ignore = "timing, not a correctness check"]
fn timing() {
    use std::time::Instant;
    let n = 2000;
    for (i, t) in openssl().iter().enumerate() {
        let p = ALL[i];
        let data = input(
            &hex::decode(&t.pk).unwrap(),
            &hex::decode(&t.signature).unwrap(),
            &hex::decode(&t.message).unwrap(),
        );
        let start = Instant::now();
        for _ in 0..n {
            assert!(p.verify(std::hint::black_box(&data)));
        }
        let per = start.elapsed().as_nanos() as f64 / n as f64 / 1000.0;
        println!(
            "{}: {per:.1} µs per verify, {} gas -> {:.0} gas/µs",
            p.name,
            p.base_gas,
            p.base_gas as f64 / per
        );
    }
    let ec = revm::precompile::secp256k1::ec_recover_run;
    let mut input = [0u8; 128];
    input[63] = 27;
    input[64..96].copy_from_slice(&[1u8; 32]);
    input[96..128].copy_from_slice(&[2u8; 32]);
    let start = Instant::now();
    for _ in 0..n {
        let _ = ec(std::hint::black_box(&input), 3000);
    }
    let per = start.elapsed().as_nanos() as f64 / n as f64 / 1000.0;
    println!("ECRECOVER: {per:.1} µs per call, 3000 gas -> {:.0} gas/µs", 3000.0 / per);
}
