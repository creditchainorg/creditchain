//! Static file producer implementation.

#![doc(
    html_logo_url = "https://www.creditchain.org",
    html_favicon_url = "https://avatars0.githubusercontent.com/u/97369466?s=256",
    issue_tracker_base_url = "https://github.com/openibank/creditchain/issues/"
)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(not(test), warn(unused_crate_dependencies))]

pub mod segments;
mod static_file_producer;

pub use static_file_producer::{
    StaticFileProducer, StaticFileProducerInner, StaticFileProducerResult,
    StaticFileProducerWithResult,
};

// Re-export for convenience.
pub use reth_static_file_types::*;
