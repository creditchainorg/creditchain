//! Reth RPC `eth_` API implementation
//!
//! ## Feature Flags
//!
//! - `client`: Enables JSON-RPC client support.

// `jsonrpsee` and `async-trait` generate boxed `Future` return types and also propagate
// `#[must_use]`, which triggers Clippy's `double_must_use` on generated code in current nightly.
#![allow(clippy::double_must_use)]
#![doc(
    html_logo_url = "https://www.creditchain.org",
    html_favicon_url = "https://avatars0.githubusercontent.com/u/97369466?s=256",
    issue_tracker_base_url = "https://github.com/creditchainorg/creditchain/issues/"
)]
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod bundle;
pub mod core;
pub mod ext;
pub mod filter;
pub mod helpers;
pub mod node;
pub mod pubsub;
pub mod types;

pub use bundle::{EthBundleApiServer, EthCallBundleApiServer};
pub use core::{EthApiServer, FullEthApiServer};
pub use ext::L2EthApiExtServer;
pub use filter::{EngineEthFilter, EthFilterApiServer, QueryLimits};
pub use helpers::config::EthConfigApiServer;
pub use node::{RpcNodeCore, RpcNodeCoreExt};
pub use pubsub::EthPubSubApiServer;
pub use reth_rpc_convert::*;
pub use reth_rpc_eth_types::error::{
    AsEthApiError, FromEthApiError, FromEvmError, IntoEthApiError,
};
pub use types::{EthApiTypes, FullEthApiTypes, RpcBlock, RpcHeader, RpcReceipt, RpcTransaction};

#[cfg(feature = "client")]
pub use bundle::{EthBundleApiClient, EthCallBundleApiClient};
#[cfg(feature = "client")]
pub use core::EthApiClient;
#[cfg(feature = "client")]
pub use ext::L2EthApiExtClient;
#[cfg(feature = "client")]
pub use filter::EthFilterApiClient;
#[cfg(feature = "client")]
pub use helpers::config::EthConfigApiClient;

use reth_trie_common as _;
