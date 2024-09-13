use anyhow::Result;
use camino::Utf8PathBuf;
use clap::{self, Parser};
use directories::ProjectDirs;
use futures::StreamExt;
use penumbra_asset::STAKING_TOKEN_ASSET_ID;
use std::fs;
use std::process::Command as ProcessCommand;
use std::str::FromStr;
use url::Url;

use pcli::config::PcliConfig;
use penumbra_keys::FullViewingKey;
use penumbra_num::Amount;
use penumbra_proto::box_grpc_svc;
use penumbra_proto::view::v1::{
    view_service_client::ViewServiceClient, view_service_server::ViewServiceServer,
};
use penumbra_stake::rate::{BaseRateData, RateData};
use penumbra_stake::DelegationToken;
use penumbra_view::{ViewClient, ViewServer};

mod genesis;

const VIEW_FILE_NAME: &str = "pcli-view.sqlite";

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
        fvks: String,
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
    pub async fn view(
        &self,
        path: Utf8PathBuf,
        fvk: FullViewingKey,
        grpc_url: Url,
    ) -> Result<ViewServiceClient<box_grpc_svc::BoxGrpcService>> {
        let registry_path = path.join("registry.json");
        // Check if the path exists or set it to none
        let registry_path = if registry_path.exists() {
            Some(registry_path)
        } else {
            None
        };
        let db_path: Utf8PathBuf = path.join(VIEW_FILE_NAME);

        let svc: ViewServer =
            ViewServer::load_or_initialize(Some(db_path), registry_path, &fvk, grpc_url).await?;

        let svc: ViewServiceServer<ViewServer> = ViewServiceServer::new(svc);
        let view_service = ViewServiceClient::new(box_grpc_svc::local(svc));
        Ok(view_service)
    }

    pub async fn sync(
        &self,
        view_service: &mut ViewServiceClient<box_grpc_svc::BoxGrpcService>,
    ) -> Result<()> {
        let mut status_stream = ViewClient::status_stream(view_service).await?;

        let initial_status = status_stream
            .next()
            .await
            .transpose()?
            .ok_or_else(|| anyhow::anyhow!("view service did not report sync status"))?;

        eprintln!(
            "Scanning blocks from last sync height {} to latest height {}",
            initial_status.full_sync_height, initial_status.latest_known_block_height,
        );

        use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
        let progress_bar = ProgressBar::with_draw_target(
            initial_status.latest_known_block_height - initial_status.full_sync_height,
            ProgressDrawTarget::stdout(),
        )
        .with_style(
            ProgressStyle::default_bar()
                .template("[{elapsed}] {bar:50.cyan/blue} {pos:>7}/{len:7} {per_sec} ETA: {eta}"),
        );
        progress_bar.set_position(0);

        while let Some(status) = status_stream.next().await.transpose()? {
            progress_bar.set_position(status.full_sync_height - initial_status.full_sync_height);
        }
        progress_bar.finish();

        Ok(())
    }

    pub async fn exec(&self) -> Result<()> {
        let opt = self;
        match &opt.cmd {
            Command::Reset {} => {
                // Delete the home directory
                fs::remove_dir_all(&opt.home)?;
                println!(
                    "Successfully cleaned up pmonitor directory: \"{}\"",
                    opt.home
                );
                Ok(())
            }
            Command::Init { fvks, grpc_url } => {
                // Parse the JSON file into a list of full viewing keys
                let fvks_str = fs::read_to_string(fvks)?;
                // Take elements from the array and parse them into FullViewingKeys
                let fvk_string_list: Vec<String> = serde_json::from_str(&fvks_str)?;

                let fvk_list: Vec<FullViewingKey> = fvk_string_list
                    .into_iter()
                    .map(|fvk| FullViewingKey::from_str(&fvk))
                    .collect::<Result<Vec<_>>>()?;

                // Create the home directory if it doesn't exist
                if !opt.home.exists() {
                    fs::create_dir_all(&opt.home)?;
                }

                // Now we need to make subdirectories for each of the FVKs and setup their
                // config files with the selected GRPC URL.
                for (index, fvk) in fvk_list.iter().enumerate() {
                    let wallet_dir = opt.home.join(format!("wallet_{}", index));

                    if !wallet_dir.exists() {
                        fs::create_dir_all(&wallet_dir)?;
                    }

                    // Invoke pcli to initialize the wallet (hacky)
                    let output = ProcessCommand::new("cargo")
                        .args(&["run", "--bin", "pcli", "--"])
                        .arg("--home")
                        .arg(wallet_dir.as_str())
                        .arg("init")
                        .arg("--grpc-url")
                        .arg(grpc_url.as_str())
                        .arg("view-only")
                        .arg(fvk.to_string())
                        .output()?;

                    if !output.status.success() {
                        anyhow::bail!(
                            "Failed to initialize wallet {}: {}",
                            index,
                            String::from_utf8_lossy(&output.stderr)
                        );
                    }
                }

                println!("Successfully initialized {} wallets", fvk_list.len());
                Ok(())
            }
            Command::Audit {} => {
                // todo: fix this
                let dummy_base_rate = BaseRateData {
                    epoch_index: 0,
                    base_reward_rate: 0u128.into(),
                    base_exchange_rate: 1_0000_0000u128.into(),
                };

                // First, we need to sync each wallet to the latest block height.
                for entry in fs::read_dir(&opt.home)? {
                    let entry = entry?;
                    let path = entry.path();
                    if path.is_dir() {
                        println!("Syncing wallet: {}", path.to_str().unwrap());

                        let utf8_path =
                            Utf8PathBuf::from_path_buf(path).expect("should be valid utf8");
                        let config = PcliConfig::load(utf8_path.join("config.toml"))?;
                        let mut view_client = self
                            .view(utf8_path, config.full_viewing_key.clone(), config.grpc_url)
                            .await?;
                        // todo: do this in parallel
                        self.sync(&mut view_client).await?;
                        println!("Wallet synced successfully");

                        let notes = view_client.unspent_notes_by_asset_and_address().await?;
                        let mut total_um_equivalent_amount = Amount::from(0u64);
                        for (asset_id, map) in notes.iter() {
                            if *asset_id == *STAKING_TOKEN_ASSET_ID {
                                let total_amount = map
                                    .iter()
                                    .map(|(_, spendable_notes)| {
                                        spendable_notes
                                            .iter()
                                            .map(|spendable_note| spendable_note.note.amount())
                                            .sum::<Amount>()
                                    })
                                    .sum::<Amount>();
                                total_um_equivalent_amount += total_amount;
                            } else if let Ok(delegation_token) =
                                DelegationToken::from_str(&asset_id.to_string())
                            {
                                let total_amount = map
                                    .iter()
                                    .map(|(_, spendable_notes)| {
                                        spendable_notes
                                            .iter()
                                            .map(|spendable_note| spendable_note.note.amount())
                                            .sum::<Amount>()
                                    })
                                    .sum::<Amount>();

                                // We need to convert the amount to the UM-equivalent amount using the appropriate rate data
                                let dummy_rate_data = RateData {
                                    identity_key: delegation_token.validator(),
                                    validator_reward_rate: 0u128.into(),
                                    validator_exchange_rate: dummy_base_rate.base_exchange_rate,
                                };
                                let um_equivalent_balance =
                                    dummy_rate_data.unbonded_amount(total_amount);
                                total_um_equivalent_amount += um_equivalent_balance;
                            };
                        }

                        println!("FVK: {:?}", config.full_viewing_key);
                        // todo: calculate the expected um equivalent balance from calling the genesis scanning method
                        let genesis_um_equivalent_amount = Amount::from(0u64);
                        println!(
                            "Genesis UM-equivalent balance: {:?}",
                            genesis_um_equivalent_amount
                        );
                        println!(
                            "Current UM-equivalent balance: {:?}",
                            total_um_equivalent_amount
                        );

                        // Let the user know if the balance is unexpected or not
                        if total_um_equivalent_amount < genesis_um_equivalent_amount {
                            println!(
                                "✘ Unexpected balance! Balance is less than the genesis balance"
                            );
                        } else {
                            println!("✅ Expected balance! Balance is greater than or equal to the genesis balance");
                        }
                    }
                }
                Ok(())
            }
        }
    }
}
