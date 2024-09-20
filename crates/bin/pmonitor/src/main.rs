use anyhow::{Context, Result};
use camino::Utf8PathBuf;
use clap::{self, Parser};
use directories::ProjectDirs;
use futures::StreamExt;
use penumbra_asset::STAKING_TOKEN_ASSET_ID;
use std::fs;
use std::process::Command as ProcessCommand;
use std::str::FromStr;
use tonic::transport::{Channel, ClientTlsConfig};
use url::Url;

use penumbra_compact_block::CompactBlock;
use penumbra_keys::FullViewingKey;
use penumbra_num::Amount;
use penumbra_proto::box_grpc_svc;
use penumbra_proto::view::v1::{
    view_service_client::ViewServiceClient, view_service_server::ViewServiceServer,
};
use penumbra_proto::{
    core::component::compact_block::v1::CompactBlockRequest,
    core::component::stake::v1::query_service_client::QueryServiceClient as StakeQueryServiceClient,
    penumbra::core::component::compact_block::v1::query_service_client::QueryServiceClient as CompactBlockQueryServiceClient,
};
use penumbra_stake::rate::RateData;
use penumbra_stake::DelegationToken;
use penumbra_view::{Storage, ViewClient, ViewServer};

mod config;
mod genesis;

use config::{AccountConfig, FvkEntry, PmonitorConfig};

// The maximum size of a compact block, in bytes (12MB).
const MAX_CB_SIZE_BYTES: usize = 12 * 1024 * 1024;

// The name of the view database file
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
    /// Set up the view service for a given wallet.
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

    /// Sync a given wallet to the latest block height.
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

    /// Fetch the genesis compact block
    pub async fn fetch_genesis_compact_block(&self, grpc_url: Url) -> Result<CompactBlock> {
        let height = 0;
        let mut client = CompactBlockQueryServiceClient::connect(grpc_url.to_string())
            .await
            .unwrap()
            .max_decoding_message_size(MAX_CB_SIZE_BYTES);
        let compact_block = client
            .compact_block(CompactBlockRequest { height })
            .await?
            .into_inner()
            .compact_block
            .expect("response has compact block");
        compact_block.try_into()
    }

    /// Stolen from pcli
    pub async fn pd_channel(&self, grpc_url: Url) -> anyhow::Result<Channel> {
        match grpc_url.scheme() {
            "http" => Ok(Channel::from_shared(grpc_url.to_string())?
                .connect()
                .await?),
            "https" => Ok(Channel::from_shared(grpc_url.to_string())?
                .tls_config(ClientTlsConfig::new())?
                .connect()
                .await?),
            other => Err(anyhow::anyhow!("unknown url scheme {other}"))
                .with_context(|| format!("could not connect to {}", grpc_url)),
        }
    }

    /// Create wallet given a path and fvk
    pub async fn create_wallet(
        &self,
        wallet_dir: &Utf8PathBuf,
        fvk: &FullViewingKey,
        grpc_url: &Url,
    ) -> Result<()> {
        // Create the wallet directory if it doesn't exist
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
                "Failed to initialize wallet in {}: {}",
                wallet_dir.to_string(),
                String::from_utf8_lossy(&output.stderr)
            );
        }

        Ok(())
    }

    /// Execute the specified command.
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
                    .iter()
                    .map(|fvk| FullViewingKey::from_str(&fvk))
                    .collect::<Result<Vec<_>>>()?;

                // Create the home directory if it doesn't exist
                if !opt.home.exists() {
                    fs::create_dir_all(&opt.home)?;
                } else {
                    anyhow::bail!("pmonitor home directory already exists: {}", opt.home);
                }

                // During init, we also compute and save the genesis balance for each
                // FVK, since that won't change in the future.
                let genesis_compact_block =
                    self.fetch_genesis_compact_block(grpc_url.clone()).await?;
                let genesis_filtered_block =
                    genesis::scan_genesis_block(genesis_compact_block, fvk_list.clone()).await?;

                let mut accounts = Vec::new();

                // Now we need to make subdirectories for each of the FVKs and setup their
                // config files, with the selected FVK and GRPC URL.
                for (index, fvk) in fvk_list.iter().enumerate() {
                    let wallet_dir = opt.home.join(format!("wallet_{}", index));
                    self.create_wallet(&wallet_dir, &fvk, &grpc_url).await?;

                    accounts.push(AccountConfig {
                        original: FvkEntry {
                            fvk: fvk.clone(),
                            path: wallet_dir.to_string(),
                        },
                        genesis_balance: *(genesis_filtered_block
                            .balances
                            .get(&fvk.to_string())
                            .expect("wallet must have genesis allocation")),
                        // We'll populate this later upon sync, if we discover the
                        // account has been migrated.
                        migrations: Vec::new(),
                    });
                }

                let config = PmonitorConfig {
                    grpc_url: grpc_url.clone(),
                    accounts: accounts.clone(),
                };

                // Save the config
                let config_path = opt.home.join("pmonitor_config.toml");
                fs::write(config_path, toml::to_string(&config)?)?;

                println!("Successfully initialized {} wallets", accounts.len());
                Ok(())
            }
            Command::Audit {} => {
                // Parse the config file to get the accounts to monitor.
                //
                // Note that each logical user might have one or more FVKs, depending on if the user
                // migrated their account to a new FVK, i.e. if they migrated once, they'll have two
                // FVKs.
                let config_path = opt.home.join("pmonitor_config.toml");
                let pmonitor_config: PmonitorConfig =
                    toml::from_str(&fs::read_to_string(config_path)?)?;

                let mut genesis_fvks = Vec::new();
                for account in pmonitor_config.accounts.iter() {
                    genesis_fvks.push(account.original.fvk.clone());
                }

                let mut stake_client = StakeQueryServiceClient::new(
                    self.pd_channel(pmonitor_config.grpc_url.clone()).await?,
                );

                // Sync each wallet to the latest block height and check the balances.
                for config in pmonitor_config.accounts.iter() {
                    let original_fvk = config.original.fvk.clone();
                    let original_path: Utf8PathBuf = config.original.path.clone().into();
                    println!("Syncing wallet: {}", original_path.to_string());

                    let mut view_client = self
                        .view(
                            original_path.clone(),
                            original_fvk.clone(),
                            pmonitor_config.grpc_url.clone(),
                        )
                        .await?;

                    // todo: do this in parallel
                    self.sync(&mut view_client).await?;
                    println!("Wallet synced successfully");

                    // Check if the account has been migrated
                    let storage = Storage::load_or_initialize(
                        Some(original_path.join("view.sqlite")),
                        &original_fvk,
                        pmonitor_config.grpc_url.clone(),
                    )
                    .await?;

                    // todo: match on the original FVK and check for further migrations
                    let migration_tx = storage
                        .transactions_matching_memo("Migrating balance from".to_string())
                        .await?;
                    if migration_tx.is_empty() {
                        // continue with the normal flow
                        dbg!("account has not been migrated");
                    } else {
                        println!("❗ Account has been migrated to new FVK");
                        // todo: get the balance from the new FVK
                    }

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
                            let rate_data: RateData = stake_client
                                .current_validator_rate(tonic::Request::new(
                                    (delegation_token.validator()).into(),
                                ))
                                .await?
                                .into_inner()
                                .try_into()?;
                            let um_equivalent_balance = rate_data.unbonded_amount(total_amount);
                            total_um_equivalent_amount += um_equivalent_balance;
                        };
                    }

                    println!("FVK: {:?}", config.original.fvk);
                    let genesis_um_equivalent_amount = config.genesis_balance;
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
                        println!("✘ Unexpected balance! Balance is less than the genesis balance");
                    } else {
                        println!("✅ Expected balance! Balance is greater than or equal to the genesis balance");
                    }
                }
                Ok(())
            }
        }
    }
}
