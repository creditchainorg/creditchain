//! The fork schedule: read from genesis config, consulted per block.

use alloy_evm::precompiles::PrecompilesMap;
use alloy_genesis::Genesis;
use cc_precompiles::{apply, mldsa, schedule};
use revm::{precompile::Precompiles, primitives::hardfork::SpecId};

fn genesis(config: &str) -> Genesis {
    serde_json::from_str(&format!(
        r#"{{"config":{{"chainId":7,{config}}},"gasLimit":"0x1c9c380","difficulty":"0x0","alloc":{{}}}}"#
    ))
    .unwrap()
}

#[test]
fn reads_the_scheduled_time() {
    let g = genesis(r#""creditchain":{"mlDsaTime":1767225600}"#);
    assert_eq!(schedule::mldsa_time(&g).unwrap(), Some(1767225600));
}

#[test]
fn absent_means_unscheduled() {
    assert_eq!(schedule::mldsa_time(&genesis(r#""shanghaiTime":0"#)).unwrap(), None);
    assert_eq!(schedule::mldsa_time(&genesis(r#""creditchain":{}"#)).unwrap(), None);
}

#[test]
fn malformed_is_an_error_not_silence() {
    for bad in [
        r#""creditchain":{"mlDsaTime":"soon"}"#,
        r#""creditchain":{"mlDsaTime":-1}"#,
        r#""creditchain":{"mlDsaTime":1.5}"#,
        r#""creditchain":7"#,
    ] {
        assert!(schedule::mldsa_time(&genesis(bad)).is_err(), "{bad} accepted");
    }
}

#[test]
fn active_from_the_scheduled_second_on_that_chain_only() {
    schedule::register_mldsa(424_242, 1_000);
    assert!(!schedule::mldsa_active(424_242, 999));
    assert!(schedule::mldsa_active(424_242, 1_000));
    assert!(schedule::mldsa_active(424_242, 5_000));
    assert!(!schedule::mldsa_active(424_243, 5_000), "another chain id");
    assert!(!schedule::mldsa_active(1, u64::MAX), "Ethereum mainnet is never registered");
}

#[test]
fn apply_installs_only_when_active() {
    schedule::register_mldsa(515_151, 100);
    let base = || PrecompilesMap::from_static(Precompiles::new(SpecId::PRAGUE.into()));

    let mut before = base();
    apply(&mut before, 515_151, 99);
    for p in mldsa::ALL {
        assert!(before.get(&p.address).is_none(), "{} present before the fork", p.name);
    }

    let mut after = base();
    apply(&mut after, 515_151, 100);
    for p in mldsa::ALL {
        assert!(after.get(&p.address).is_some(), "{} missing after the fork", p.name);
    }
    // Ethereum's own precompiles are untouched.
    assert!(after
        .get(&alloy_primitives::address!("0x0000000000000000000000000000000000000001"))
        .is_some());
}
