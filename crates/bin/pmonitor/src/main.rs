use anyhow::Result;
use camino::Utf8PathBuf;
use clap::{self, Parser};
use directories::ProjectDirs;
use std::fs;
use url::Url;

use penumbra_keys::FullViewingKey;

#[tokio::main]
async fn main() -> Result<()> {
    let opt = Opt::parse();
    opt.exec().await
}

pub fn default_home() -> Utf8PathBuf {
    let path = ProjectDirs::from("zone", "penumbra", "pmonitor")
        .expect("Failed to get platform data dir")
        .data_dir()
        .to_path_buf();
    Utf8PathBuf::from_path_buf(path).expect("Platform default data dir was not UTF-8")
}

#[derive(Debug, Parser)]
#[clap(
    name = "pmonitor",
    about = "The Penumbra account activity monitor.",
    version
)]
pub struct Opt {
    /// Command to run.
    #[clap(subcommand)]
    pub cmd: Command,
    /// The path used to store pmonitor state.
    #[clap(long, default_value_t = default_home(), env = "PENUMBRA_PMONITOR_HOME")]
    pub home: Utf8PathBuf,
}

#[derive(Debug, clap::Subcommand)]
pub enum Command {
    /// Generate configs for `pmonitor`.
    Init {
        /// Provide JSON file with the list of full viewing keys to monitor.
        #[clap(long, display_order = 200)]
        fvks_json: String,
        /// Sets the URL of the gRPC endpoint used to sync the wallets.
        #[clap(
            long,
            display_order = 900,
            parse(try_from_str = Url::parse)
        )]
        grpc_url: Url,
    },
    /// Sync to latest block height and verify all configured wallets have the correct balance.
    Audit {},
    /// Delete `pmonitor` storage to reset local state.
    Reset {},
}

impl Opt {
    pub async fn exec(self) -> Result<()> {
        let opt = self;
        match &opt.cmd {
            Command::Reset {} => {
                todo!("need to impl");
                Ok(())
            }
            Command::Init {
                fvks_json,
                grpc_url,
            } => {
                todo!();
                // Parse the JSON file into a list of full viewing keys
                let fvk_list: Vec<FullViewingKey> = serde_json::from_str(fvks_json)?;

                // Create the home directory if it doesn't exist
                if !opt.home.exists() {
                    fs::create_dir_all(&opt.home)?;
                }

                // Now we need to make subdirectories for each of the FVKs and setup their
                // config files with the selected GRPC URL.
                Ok(())
            }
            Command::Audit {} => {
                todo!();
                Ok(())
            }
        }
    }
}
