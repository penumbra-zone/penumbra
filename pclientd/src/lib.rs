use std::env;

use camino::Utf8PathBuf;
use clap::Parser;
use penumbra_crypto::FullViewingKey;
use penumbra_custody::soft_kms::{self};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ClientConfig {
    /// Optional KMS config for custody mode
    pub kms_config: Option<soft_kms::Config>,
    /// FVK for both view and custody modes
    pub fvk: FullViewingKey,
}

#[derive(Debug, Parser)]
#[clap(
    name = "pclientd",
    about = "The Penumbra view daemon.",
    version = env!("VERGEN_GIT_SEMVER"),
)]
pub struct Opt {
    /// Command to run.
    #[clap(subcommand)]
    pub cmd: Command,
    /// The path used to store pclientd state and config files.
    #[clap(long)]
    pub home: Utf8PathBuf,
    /// The address of the pd+tendermint node.
    #[clap(
        short,
        long,
        default_value = "testnet.penumbra.zone",
        env = "PENUMBRA_NODE_HOSTNAME"
    )]
    pub node: String,
    /// The port to use to speak to pd's gRPC server.
    #[clap(long, default_value = "8080", env = "PENUMBRA_PD_PORT")]
    pub pd_port: u16,
}

#[derive(Debug, clap::Subcommand)]
pub enum Command {
    /// Initialize pclientd with the provided full viewing key (and optional seed phrase in custody mode)
    Init {
        /// The full viewing key to initialize the view service with.
        full_viewing_key: String,
        // If true, initialize in custody mode with the seed phrase provided to stdin
        #[clap(short, long)]
        custody: bool,
    },
    /// Start the view service.
    Start {
        /// Bind the view service to this host.
        #[clap(long, default_value = "127.0.0.1")]
        host: String,
        /// Bind the view gRPC server to this port.
        #[clap(long, default_value = "8081")]
        view_port: u16,
    },
}
