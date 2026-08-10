//! `CreditChain` CLI implementation.

#![doc(
    html_logo_url = "https://www.creditchain.org",
    html_favicon_url = "https://www.creditchain.org/favicon.ico",
    issue_tracker_base_url = "https://github.com/openibank/creditchain/issues/"
)]
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(docsrs, feature(doc_cfg))]

/// A configurable App on top of the cli parser.
pub mod app;
/// Chain specification parser.
pub mod chainspec;
pub mod interface;

pub use app::{CliApp, ExtendedCommand};
pub use interface::{Cli, Commands, NoSubCmd};

#[cfg(test)]
mod test {
    use crate::chainspec::EthereumChainSpecParser;
    use clap::Parser;
    use reth_chainspec::DEV;
    use reth_cli_commands::NodeCommand;

    #[test]
    #[ignore = "reth cmd output differs when optimism feature enabled"]
    fn parse_dev() {
        let cmd: NodeCommand<EthereumChainSpecParser> = NodeCommand::parse_from(["reth", "--dev"]);
        let chain = DEV.clone();
        assert_eq!(cmd.chain.chain, chain.chain);
        assert_eq!(cmd.chain.genesis_hash(), chain.genesis_hash());
        assert_eq!(
            cmd.chain.paris_block_and_final_difficulty,
            chain.paris_block_and_final_difficulty
        );
        assert_eq!(cmd.chain.hardforks, chain.hardforks);

        assert!(cmd.rpc.http);
        assert!(cmd.network.discovery.disable_discovery);

        assert!(cmd.dev.dev);
    }
}
