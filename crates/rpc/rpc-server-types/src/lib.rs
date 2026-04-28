//! CreditChain RPC server types.

#![doc(
    html_logo_url = "https://www.creditchain.org",
    html_favicon_url = "https://www.creditchain.org/favicon.ico",
    issue_tracker_base_url = "https://github.com/openibank/creditchain/issues/"
)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(not(test), warn(unused_crate_dependencies))]

/// Common RPC constants.
pub mod constants;
pub mod result;

mod module;
pub use module::{
    DefaultRpcModuleValidator, LenientRpcModuleValidator, RethRpcModule, RpcModuleSelection,
    RpcModuleValidator,
};

pub use result::ToRpcResult;
