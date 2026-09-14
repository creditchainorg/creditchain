//! A local engine service that can be used to drive a dev chain.

#![doc(
    html_logo_url = "https://www.creditchain.org",
    html_favicon_url = "https://avatars0.githubusercontent.com/u/97369466?s=256",
    issue_tracker_base_url = "https://github.com/creditchainorg/creditchain/issues/"
)]
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod miner;
pub mod payload;

pub use miner::{LocalMiner, MiningMode, DEFAULT_FINALITY_DEPTH};
pub use payload::LocalPayloadAttributesBuilder;
