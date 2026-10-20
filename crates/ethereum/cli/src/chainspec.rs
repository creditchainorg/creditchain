use alloy_genesis::Genesis;
use reth_chainspec::{ChainSpec, ForkCondition, DEV, HOLESKY, HOODI, MAINNET, SEPOLIA};
use reth_cli::chainspec::{parse_genesis, ChainSpecParser};
use std::sync::Arc;

/// Chains supported by `CreditChain`.
///
/// The upstream Ethereum aliases remain available so compatibility testing and shallow-fork
/// maintenance can continue without bespoke tooling.
pub const SUPPORTED_CHAINS: &[&str] = &[
    "mainnet",
    "sepolia",
    "holesky",
    "hoodi",
    "dev",
    "local-single",
    "local-multinode",
    "devnet",
    "argos-testnet",
    "testnet",
    "argos",
    "creditchain-mainnet",
];

/// Mainnet names that exist but have no genesis yet. Argos mainnet's allocation and
/// withdrawal addresses come from the offline key ceremony; until that record exists there
/// is nothing honest to start, and a placeholder genesis would be a real chain that nobody
/// intended.
pub const UNPUBLISHED_CHAINS: &[&str] = &["argos", "creditchain-mainnet"];

/// Clap value parser for [`ChainSpec`]s.
///
/// The value parser matches either a known chain, the path
/// to a json file, or a json formatted string in-memory. The json needs to be a Genesis struct.
pub fn chain_value_parser(s: &str) -> eyre::Result<Arc<ChainSpec>, eyre::Error> {
    Ok(match s {
        "mainnet" => MAINNET.clone(),
        "sepolia" => SEPOLIA.clone(),
        "holesky" => HOLESKY.clone(),
        "hoodi" => HOODI.clone(),
        "dev" => DEV.clone(),
        "local-single" => creditchain_genesis("local-single")?,
        "local-multinode" => creditchain_genesis("local-multinode")?,
        "devnet" => creditchain_genesis("devnet")?,
        // `testnet` follows the current testnet release (docs/releases/RELEASE-NAMES.md).
        // It used to read a hand-written genesis/testnet.json that shares Argos's chain id
        // but hashes to a different genesis, so a node started with the obvious name could
        // never sync with the network it was named after.
        "argos-testnet" | "testnet" => creditchain_genesis("argos-testnet")?,
        "argos" | "creditchain-mainnet" => eyre::bail!(
            "`{s}` has no genesis yet: Argos mainnet's allocation and withdrawal addresses are \
             produced by the offline key ceremony, which has not run. Use `argos-testnet`."
        ),
        _ => Arc::new(creditchain_spec(parse_genesis(s)?)?),
    })
}

fn creditchain_genesis(environment: &str) -> eyre::Result<Arc<ChainSpec>> {
    let raw = match environment {
        "local-single" => include_str!("../../../../genesis/local-single.json"),
        "local-multinode" => include_str!("../../../../genesis/local-multinode.json"),
        "devnet" => include_str!("../../../../genesis/devnet.json"),
        // The generator's output for the live chain, not a hand-written file.
        "argos-testnet" => include_str!("../../../../genesis/argos-testnet.json"),
        _ => unreachable!("unknown built-in CreditChain environment"),
    };
    Ok(Arc::new(creditchain_spec(parse_genesis(raw)?)?))
}

/// Build a chain spec from a genesis file, including `CreditChain`'s own scheduled forks.
///
/// A fork scheduled in `config.creditchain` is added to the spec's hardforks, so the fork id
/// (EIP-2124/EIP-6122) announces it and peers that have not upgraded are told apart, and it is
/// registered for the EVM factory, which adds the fork's precompiles from the scheduled block on.
fn creditchain_spec(genesis: Genesis) -> eyre::Result<ChainSpec> {
    let mldsa_time = cc_precompiles::schedule::mldsa_time(&genesis).map_err(|e| eyre::eyre!(e))?;
    let mut spec: ChainSpec = genesis.into();
    if let Some(time) = mldsa_time {
        spec.hardforks.insert(cc_precompiles::schedule::MlDsaFork, ForkCondition::Timestamp(time));
        cc_precompiles::schedule::register_mldsa(spec.chain.id(), time);
    }
    Ok(spec)
}

/// `CreditChain` chain specification parser.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct EthereumChainSpecParser;

impl ChainSpecParser for EthereumChainSpecParser {
    type ChainSpec = ChainSpec;

    const SUPPORTED_CHAINS: &'static [&'static str] = SUPPORTED_CHAINS;

    fn parse(s: &str) -> eyre::Result<Arc<ChainSpec>> {
        chain_value_parser(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reth_chainspec::EthereumHardforks;

    #[test]
    fn parse_known_chain_spec() {
        for &chain in EthereumChainSpecParser::SUPPORTED_CHAINS {
            if UNPUBLISHED_CHAINS.contains(&chain) {
                continue;
            }
            assert!(<EthereumChainSpecParser as ChainSpecParser>::parse(chain).is_ok(), "{chain}");
        }
    }

    /// The hash of block 0 on the live Argos testnet, read from the running network.
    const ARGOS_TESTNET_GENESIS_HASH: &str =
        "0xcbb0f12e2a67c07a59218baefb4eb8211d9867b52b716fb190e4df12628042c9";

    #[test]
    fn argos_testnet_is_the_live_chain() {
        for name in ["argos-testnet", "testnet"] {
            let spec = <EthereumChainSpecParser as ChainSpecParser>::parse(name).unwrap();
            assert_eq!(spec.chain.id(), 2026042404, "{name}");
            assert_eq!(format!("{:#x}", spec.genesis_hash()), ARGOS_TESTNET_GENESIS_HASH, "{name}");
        }
    }

    /// Argos's genesis file with the ML-DSA fork scheduled at `time`.
    fn argos_with_mldsa(time: &str) -> String {
        let raw = include_str!("../../../../genesis/argos-testnet.json");
        let mut value: serde_json::Value = serde_json::from_str(raw).unwrap();
        value["config"]["creditchain"] =
            serde_json::from_str(&format!(r#"{{"mlDsaTime":{time}}}"#)).unwrap();
        value.to_string()
    }

    #[test]
    fn scheduling_the_mldsa_fork_leaves_the_genesis_hash_alone() {
        let spec = chain_value_parser(&argos_with_mldsa("4102444800")).unwrap();
        assert_eq!(format!("{:#x}", spec.genesis_hash()), ARGOS_TESTNET_GENESIS_HASH);
    }

    #[test]
    fn the_mldsa_fork_is_announced_in_the_fork_id() {
        use reth_chainspec::Head;
        let time = 4_102_444_800u64; // 2100-01-01
        let plain = chain_value_parser("argos-testnet").unwrap();
        let forked = chain_value_parser(&argos_with_mldsa(&time.to_string())).unwrap();
        let head = Head { timestamp: time - 1, ..Default::default() };
        assert_eq!(plain.fork_id(&head).next, 0, "nothing scheduled on plain Argos");
        assert_eq!(forked.fork_id(&head).next, time, "the scheduled fork is the next one");
        assert_eq!(plain.fork_id(&head).hash, forked.fork_id(&head).hash, "same chain until then");
        let after = Head { timestamp: time, ..Default::default() };
        assert_ne!(plain.fork_id(&after).hash, forked.fork_id(&after).hash, "distinct after");
        assert!(cc_precompiles::schedule::mldsa_active(2026042404, time));
        assert!(!cc_precompiles::schedule::mldsa_active(2026042404, time - 1));
    }

    #[test]
    fn a_malformed_schedule_stops_the_node() {
        let err = chain_value_parser(&argos_with_mldsa(r#""tomorrow""#)).unwrap_err();
        assert!(err.to_string().contains("mlDsaTime"), "{err}");
    }

    #[test]
    fn mainnet_names_refuse_until_the_ceremony() {
        for &name in UNPUBLISHED_CHAINS {
            let err = <EthereumChainSpecParser as ChainSpecParser>::parse(name).unwrap_err();
            assert!(err.to_string().contains("key ceremony"), "{name}: {err}");
        }
    }

    #[test]
    fn parse_raw_chainspec_hardforks() {
        let s = r#"{
  "parentHash": "0x0000000000000000000000000000000000000000000000000000000000000000",
  "uncleHash": "0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347",
  "coinbase": "0x0000000000000000000000000000000000000000",
  "stateRoot": "0x76f118cb05a8bc558388df9e3b4ad66ae1f17ef656e5308cb8f600717251b509",
  "transactionsTrie": "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421",
  "receiptTrie": "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421",
  "bloom": "0x000...000",
  "difficulty": "0x00",
  "number": "0x00",
  "gasLimit": "0x016345785d8a0000",
  "gasUsed": "0x00",
  "timestamp": "0x01",
  "extraData": "0x00",
  "mixHash": "0x0000000000000000000000000000000000000000000000000000000000000000",
  "nonce": "0x0000000000000000",
  "baseFeePerGas": "0x07",
  "withdrawalsRoot": "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421",
  "blobGasUsed": "0x00",
  "excessBlobGas": "0x00",
  "parentBeaconBlockRoot": "0x0000000000000000000000000000000000000000000000000000000000000000",
  "requestsRoot": "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421",
  "hash": "0xc20e1a771553139cdc77e6c3d5f64a7165d972d327eee9632c9c7d0fe839ded4",
  "alloc": {},
  "config": {
    "ethash": {},
    "chainId": 1,
    "homesteadBlock": 0,
    "daoForkSupport": true,
    "eip150Block": 0,
    "eip155Block": 0,
    "eip158Block": 0,
    "byzantiumBlock": 0,
    "constantinopleBlock": 0,
    "petersburgBlock": 0,
    "istanbulBlock": 0,
    "berlinBlock": 0,
    "londonBlock": 0,
    "terminalTotalDifficulty": 0,
    "shanghaiTime": 0,
    "cancunTime": 0,
    "pragueTime": 0,
    "osakaTime": 0
  }
}"#;

        let spec = <EthereumChainSpecParser as ChainSpecParser>::parse(s).unwrap();
        assert!(spec.is_shanghai_active_at_timestamp(0));
        assert!(spec.is_cancun_active_at_timestamp(0));
        assert!(spec.is_prague_active_at_timestamp(0));
        assert!(spec.is_osaka_active_at_timestamp(0));
    }
}
