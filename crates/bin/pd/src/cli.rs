//! Command-line interface utilities for the `pd` daemon.

use {
    clap::{Parser, Subcommand},
    std::{net::SocketAddr, path::PathBuf},
    url::Url,
};

#[derive(Debug, Parser)]
#[clap(name = "pd", about = "The Penumbra daemon.", version)]
pub struct Opt {
    /// Enable Tokio Console support.
    #[clap(long)]
    pub tokio_console: bool,
    /// Command to run.
    #[clap(subcommand)]
    pub cmd: RootCommand,
}

#[derive(Debug, Subcommand)]
pub enum RootCommand {
    /// Starts the Penumbra daemon.
    Start {
        /// The path used to store all `pd`-related data and configuration.
        /// If unset, defaults to ~/.penumbra/testnet_data/node0/pd.
        #[clap(long, env = "PENUMBRA_PD_HOME", display_order = 100)]
        home: Option<PathBuf>,
        /// Bind the ABCI server to this socket.
        ///
        /// The ABCI server is used by Tendermint to drive the application state.
        #[clap(
            short,
            long,
            env = "PENUMBRA_PD_ABCI_BIND",
            default_value = "127.0.0.1:26658",
            display_order = 400
        )]
        // TODO: Add support for Unix domain sockets, available in tower-abci >=0.10.0
        abci_bind: SocketAddr,
        /// Bind the gRPC server to this socket.
        ///
        /// The gRPC server supports both grpc (HTTP/2) and grpc-web (HTTP/1.1) clients.
        ///
        /// If `grpc_auto_https` is set, this defaults to `0.0.0.0:443` and uses HTTPS.
        ///
        /// If `grpc_auto_https` is not set, this defaults to `127.0.0.1:8080` without HTTPS.
        #[clap(short, long, env = "PENUMBRA_PD_GRPC_BIND", display_order = 201)]
        grpc_bind: Option<SocketAddr>,
        /// If set, serve gRPC using auto-managed HTTPS with this domain name.
        ///
        /// NOTE: This option automatically provisions TLS certificates from
        /// Let's Encrypt and caches them in the `home` directory.  The
        /// production LE CA has rate limits, so be careful using this option
        /// with `pd testnet unsafe-reset-all`, which will delete the certificates
        /// and force re-issuance, possibly hitting the rate limit. See the
        /// `--acme-staging` option.
        #[clap(long, value_name = "DOMAIN", display_order = 200)]
        grpc_auto_https: Option<String>,
        /// Enable use of the LetsEncrypt ACME staging API (https://letsencrypt.org/docs/staging-environment/),
        /// which is more forgiving of ratelimits. Set this option to `true`
        /// if you're trying out the `--grpc-auto-https` option for the first time,
        /// to validate your configuration, before subjecting yourself to production
        /// ratelimits. This option has no effect if `--grpc-auto-https` is not set.
        #[clap(long, display_order = 201, default_value = "false")]
        acme_staging: bool,
        /// Bind the metrics endpoint to this socket.
        #[clap(
            short,
            long,
            env = "PENUMBRA_PD_METRICS_BIND",
            default_value = "127.0.0.1:9000",
            display_order = 300
        )]
        metrics_bind: SocketAddr,
        /// The JSON-RPC address of the CometBFT node driving this `pd`
        /// instance.
        ///
        /// This is used to proxy requests from the gRPC server to CometBFT,
        /// so clients only need to connect to one endpoint and don't need to
        /// worry about the peculiarities of CometBFT's JSON-RPC encoding
        /// format.
        #[clap(
            short,
            long,
            env = "PENUMBRA_PD_COMETBFT_PROXY_URL",
            default_value = "http://127.0.0.1:26657",
            display_order = 401,
            // Support old arg name for a while, as we migrate Tendermint -> CometBFT.
            alias = "tendermint-addr",
        )]
        cometbft_addr: Url,

        /// Enable expensive RPCs, such as the trade simulation service.
        /// The trade simulation service allows clients to simulate trades without submitting them.
        /// This is useful for approximating the cost of a trade before submitting it.
        /// But, it is a potential DoS vector, so it is disabled by default.
        #[clap(short, long, display_order = 500)]
        enable_expensive_rpc: bool,
    },
    /// Generate, join, or reset a testnet.
    Testnet {
        /// Path to directory to store output in. Must not exist. Defaults to
        /// ~/.penumbra/testnet_data".
        #[clap(long)]
        testnet_dir: Option<PathBuf>,

        #[clap(subcommand)]
        tn_cmd: TestnetCommand,
    },

    /// Export the storage state the full node.
    Export {
        /// The home directory of the full node.
        #[clap(long, env = "PENUMBRA_PD_HOME", display_order = 100)]
        home: PathBuf,
        /// The directory that the exported state will be written to.
        #[clap(long, display_order = 200)]
        export_path: PathBuf,
        /// Whether to prune the JMT tree.
        #[clap(long, display_order = 300)]
        prune: bool,
    },
    /// Run a migration on the exported storage state of the full node,
    /// and create a genesis file.
    Migrate {
        /// The directory containing exported state to which the upgrade will be applied.
        #[clap(long, display_order = 200)]
        target_dir: PathBuf,
        #[clap(long, display_order = 300)]
        /// Timestamp of the genesis file in RFC3339 format. If unset, defaults to the current time,
        /// unless the migration logic overrides it.
        genesis_start: Option<tendermint::time::Time>,
    },
}

#[derive(Debug, Subcommand)]
pub enum TestnetCommand {
    /// Generates a directory structure containing necessary files to create a new
    /// testnet from genesis, based on input configuration.
    Generate {
        /// The `timeout_commit` parameter (block interval) to configure Tendermint with.
        #[clap(long)]
        timeout_commit: Option<tendermint::Timeout>,
        /// Number of blocks per epoch.
        #[clap(long)]
        epoch_duration: Option<u64>,
        /// Number of epochs before unbonding stake is released.
        #[clap(long)]
        unbonding_epochs: Option<u64>,
        /// Maximum number of validators in the consensus set.
        #[clap(long)]
        active_validator_limit: Option<u64>,
        /// Whether to preserve the chain ID (useful for public testnets) or append a random suffix (useful for dev/testing).
        #[clap(long)]
        preserve_chain_id: bool,
        /// Path to CSV file containing initial allocations [default: latest testnet].
        #[clap(long, parse(from_os_str))]
        allocations_input_file: Option<PathBuf>,
        /// Path to JSON file containing initial validator configs [default: latest testnet].
        #[clap(long, parse(from_os_str))]
        validators_input_file: Option<PathBuf>,
        /// Testnet name [default: latest testnet].
        #[clap(long)]
        chain_id: Option<String>,
        /// The duration, in number of blocks, that a governance proposal
        /// can be voted on.
        #[clap(long)]
        proposal_voting_blocks: Option<u64>,
        /// Base hostname for a validator's p2p service. If multiple validators
        /// exist in the genesis, e.g. via `--validators-input-file`, then
        /// numeric suffixes are automatically added, e.g. "-0", "-1", etc.
        /// Helpful for when you know the validator DNS names ahead of time,
        /// e.g. in Kubernetes service addresses. These option is most useful
        /// to provide peering on a private network setup. If you plan to expose
        /// the validator P2P services to the internet, see the `--external-addresses` option.
        #[clap(long)]
        peer_address_template: Option<String>,

        /// Public addresses and ports for the Tendermint P2P services of the genesis
        /// validator. Accepts comma-separated values, to support multiple validators.
        /// If `--validators-input-file` is used to increase the number
        /// of validators, and the `--external-addresses` flag is set, then the number of
        /// external addresses must equal the number of validators. See the
        /// `--peer-address-template` flag if you don't plan to expose the network
        /// to public peers.
        #[clap(long)]
        // TODO we should support DNS names here. However, there are complications:
        // https://github.com/tendermint/tendermint/issues/1521
        external_addresses: Option<String>,
    },

    /// Like `testnet generate`, but joins the testnet to which the specified node belongs
    Join {
        /// URL of the remote Tendermint RPC endpoint for bootstrapping connection.
        #[clap(
            env = "PENUMBRA_PD_JOIN_URL",
            default_value = "https://rpc.testnet.penumbra.zone"
        )]
        node: Url,
        /// Human-readable name to identify node on network
        // Default: 'node-#'
        #[clap(long, env = "PENUMBRA_PD_TM_MONIKER")]
        moniker: Option<String>,
        /// Public URL to advertise for this node's Tendermint P2P service.
        /// Setting this option will instruct other nodes on the network to connect
        /// to yours. Must be in the form of a socket, e.g. "1.2.3.4:26656".
        #[clap(long, env = "PENUMBRA_PD_TM_EXTERNAL_ADDR")]
        external_address: Option<SocketAddr>,
        /// When generating Tendermint config, use this socket to bind the Tendermint RPC service.
        #[clap(long, env = "PENUMBRA_PD_TM_RPC_BIND", default_value = "0.0.0.0:26657")]
        tendermint_rpc_bind: SocketAddr,
        /// When generating Tendermint config, use this socket to bind the Tendermint P2P service.
        #[clap(long, env = "PENUMBRA_PD_TM_P2P_BIND", default_value = "0.0.0.0:26656")]
        tendermint_p2p_bind: SocketAddr,
    },

    /// Reset all `pd` testnet state.
    UnsafeResetAll {},
}
