//! Core `CreditChain` node utilities and libraries.

#![doc(
    html_logo_url = "https://www.creditchain.org",
    html_favicon_url = "https://www.creditchain.org/favicon.ico",
    issue_tracker_base_url = "https://github.com/openibank/creditchain/issues/"
)]
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod args;
pub mod cli;
pub mod dirs;
pub mod exit;
pub mod node_config;
pub mod utils;
pub mod version;

/// Re-exported primitive types
pub mod primitives {
    pub use reth_ethereum_forks::*;
    pub use reth_primitives_traits::*;
}

/// Re-export of `reth_rpc_*` crates.
pub mod rpc {
    /// Re-exported from `reth_rpc_server_types::result`.
    pub mod result {
        pub use reth_rpc_server_types::result::*;
    }

    /// Re-exported from `reth_rpc_convert`.
    pub mod compat {
        pub use reth_rpc_convert::*;
    }
}
