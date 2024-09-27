use anyhow::{Context, Result};
use camino::Utf8PathBuf;
use clap::{self, Parser};
use directories::ProjectDirs;
use futures::StreamExt;
use penumbra_asset::STAKING_TOKEN_ASSET_ID;
use std::fs;
use std::io::IsTerminal as _;
use std::process::Command as ProcessCommand;
use std::str::FromStr;
use tonic::transport::{Channel, ClientTlsConfig};
use tracing_subscriber::{prelude::*, EnvFilter};
use url::Url;
use uuid::Uuid;

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

use config::{parse_dest_fvk_from_memo, AccountConfig, FvkEntry, PmonitorConfig};

// The maximum size of a compact block, in bytes (12MB).
const MAX_CB_SIZE_BYTES: usize = 12 * 1024 * 1024;

// The name of the view database file
const VIEW_FILE_NAME: &str = "pcli-view.sqlite";

/// Configure tracing_subscriber for logging messages
fn init_tracing() -> anyhow::Result<()> {
    // Instantiate tracing layers.
    // The `FmtLayer` is used to print to the console.
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_ansi(std::io::stdout().is_terminal())
        .with_target(true);
    // The `EnvFilter` layer is used to filter events based on `RUST_LOG`.
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info,penumbra_view=off"))?;

    // Register the tracing subscribers.
    let registry = tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer);
    registry.init();
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let opt = Opt::parse();
    init_tracing()?;
    tracing::info!(?opt, version = env!("CARGO_PKG_VERSION"), "running command");
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

    /// Get the path to the wallet directory for a given wallet ID.
    pub fn wallet_path(&self, wallet_id: &Uuid) -> Utf8PathBuf {
        self.home.join(format!("wallet_{}", wallet_id))
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

        tracing::debug!(
            "scanning blocks from last sync height {} to latest height {}",
            initial_status.full_sync_height,
            initial_status.latest_known_block_height,
        );

        // use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
        // let progress_bar = ProgressBar::with_draw_target(
        //     initial_status.latest_known_block_height - initial_status.full_sync_height,
        //     ProgressDrawTarget::stdout(),
        // )
        // .with_style(
        //     ProgressStyle::default_bar()
        //         .template("[{elapsed}] {bar:50.cyan/blue} {pos:>7}/{len:7} {per_sec} ETA: {eta}"),
        // );
        // progress_bar.set_position(0);

        // On large networks, logging an update every 100k blocks or so seems reasonable.
        // let log_every_n_blocks = 100000;
        let log_every_n_blocks = 100;
        while let Some(status) = status_stream.next().await.transpose()? {
            if status.full_sync_height % log_every_n_blocks == 0 {
                tracing::debug!("synced {} blocks", status.full_sync_height);
            }
            // progress_bar.set_position(status.full_sync_height - initial_status.full_sync_height);
        }
        // progress_bar.finish();

        Ok(())
    }

    /// Fetch the genesis compact block
    pub async fn fetch_genesis_compact_block(&self, grpc_url: Url) -> Result<CompactBlock> {
        let height = 0;
        let mut client = CompactBlockQueryServiceClient::connect(grpc_url.to_string())
            .await?
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

    /// Compute the UM-equivalent balance for a given (synced) wallet.
    pub async fn compute_um_equivalent_balance(
        &self,
        view_client: &mut ViewServiceClient<box_grpc_svc::BoxGrpcService>,
        stake_client: &mut StakeQueryServiceClient<Channel>,
    ) -> Result<Amount> {
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
            } else if let Ok(delegation_token) = DelegationToken::from_str(&asset_id.to_string()) {
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
        Ok(total_um_equivalent_amount)
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
                println!("Successfully read FVKs from provided file");

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
                println!("About to scan the genesis block... this may take a moment");
                let genesis_filtered_block =
                    genesis::scan_genesis_block(genesis_compact_block, fvk_list.clone()).await?;

                let mut accounts = Vec::new();

                // Now we need to make subdirectories for each of the FVKs and setup their
                // config files, with the selected FVK and GRPC URL.
                for fvk in fvk_list.iter() {
                    let wallet_id = Uuid::new_v4();
                    let wallet_dir = self.wallet_path(&wallet_id);
                    tracing::debug!("creating wallet at {}", wallet_dir.to_string());
                    self.create_wallet(&wallet_dir, &fvk, &grpc_url).await?;

                    accounts.push(AccountConfig::new(
                        FvkEntry {
                            fvk: fvk.clone(),
                            wallet_id,
                        },
                        *(genesis_filtered_block
                            .balances
                            .get(&fvk.to_string())
                            .unwrap_or(&Amount::from(0u64))),
                    ));
                }

                tracing::info!("successfully initialized {} wallets", accounts.len());
                let pmonitor_config = PmonitorConfig::new(grpc_url.clone(), accounts);

                // Save the config
                let config_path = opt.home.join("pmonitor_config.toml");
                fs::write(config_path, toml::to_string(&pmonitor_config)?)?;

                Ok(())
            }
            Command::Audit {} => {
                // Parse the config file to get the accounts to monitor.
                //
                // Note that each logical genesis entry might now have one or more FVKs, depending on if the
                // user migrated their account to a new FVK, i.e. if they migrated once, they'll have two
                // FVKs. This can happen an unlimited number of times.
                let config_path = opt.home.join("pmonitor_config.toml");
                let pmonitor_config: PmonitorConfig =
                    toml::from_str(&fs::read_to_string(config_path.clone())?)?;

                let mut stake_client = StakeQueryServiceClient::new(
                    self.pd_channel(pmonitor_config.grpc_url()).await?,
                );

                // Sync each wallet to the latest block height, check for new migrations, and check the balance.
                let mut updated_config = pmonitor_config.clone();
                let mut config_updated = false;

                let num_accounts = pmonitor_config.accounts().len();

                for (index, config) in pmonitor_config.accounts().iter().enumerate() {
                    let active_fvk = config.active_fvk();
                    let active_path = self.wallet_path(&config.active_uuid());
                    tracing::info!(
                        "syncing wallet {}/{}: {}",
                        index + 1,
                        num_accounts,
                        active_path.to_string()
                    );
                    // println!("Syncing wallet: {}", active_path.to_string());

                    let mut view_client = self
                        .view(
                            active_path.clone(),
                            active_fvk.clone(),
                            pmonitor_config.grpc_url(),
                        )
                        .await?;

                    // todo: do this in parallel?
                    self.sync(&mut view_client).await?;
                    tracing::debug!("finished syncing wallet {}/{}", index + 1, num_accounts);

                    // Check if the account has been migrated
                    let storage = Storage::load_or_initialize(
                        Some(active_path.join("view.sqlite")),
                        &active_fvk,
                        pmonitor_config.grpc_url(),
                    )
                    .await?;

                    let migration_tx = storage
                        .transactions_matching_memo(format!(
                            // N.B. the `%` symbol is an SQLite wildcard, required to match the
                            // remainder of the memo field.
                            "Migrating balance from {}%",
                            active_fvk.to_string()
                        ))
                        .await?;
                    if migration_tx.is_empty() {
                        tracing::debug!(
                            "account has not been migrated, continuing using existing FVK..."
                        );
                    } else if migration_tx.len() == 1 {
                        tracing::warn!(
                            "❗ account has been migrated to new FVK, continuing using new FVK..."
                        );
                        let (_, _, _tx, memo_text) = &migration_tx[0];
                        let new_fvk = parse_dest_fvk_from_memo(&memo_text)?;
                        let wallet_id = Uuid::new_v4();
                        let wallet_dir = self.wallet_path(&wallet_id);
                        self.create_wallet(&wallet_dir, &new_fvk, &pmonitor_config.grpc_url())
                            .await?;

                        let new_fvk_entry = FvkEntry {
                            fvk: new_fvk.clone(),
                            wallet_id,
                        };
                        // Mark that the config needs to get saved again for the next time we run the audit command.
                        config_updated = true;

                        // We need to update the config with the new FVK and path on disk
                        // to the wallet for the next time we run the audit command.
                        let mut new_config_entry = config.clone();
                        new_config_entry.add_migration(new_fvk_entry);
                        updated_config.set_account(index, new_config_entry.clone());

                        let mut view_client = self
                            .view(wallet_dir, new_fvk.clone(), pmonitor_config.grpc_url())
                            .await?;

                        println!("Syncing new wallet...");
                        self.sync(&mut view_client).await?;
                        println!("Wallet synced successfully");
                        // Now we can exit the else if statement and continue by computing the balance,
                        // which will use the new migrated wallet.
                    } else {
                        // we expect a single migration tx per FVK, if this assumption is violated we should bail.
                        anyhow::bail!(
                            "Expected a single migration tx, found {}",
                            migration_tx.len()
                        );
                    }

                    let current_um_equivalent_amount = self
                        .compute_um_equivalent_balance(&mut view_client, &mut stake_client)
                        .await?;

                    tracing::debug!("original FVK: {:?}", config.original_fvk());
                    let genesis_um_equivalent_amount = config.genesis_balance();
                    // Let the user know if the balance is unexpected or not
                    if current_um_equivalent_amount < genesis_um_equivalent_amount {
                        tracing::error!(
                            ?genesis_um_equivalent_amount,
                            ?current_um_equivalent_amount,
                            "❌ unexpected balance! balance is less than the genesis balance"
                        );
                    } else {
                        tracing::info!(?genesis_um_equivalent_amount, ?current_um_equivalent_amount, "✅ expected balance! balance is greater than or equal to the genesis balance");
                    }
                }

                // If at any point we marked the config for updating, we need to save it.
                if config_updated {
                    fs::write(config_path.clone(), toml::to_string(&updated_config)?)?;
                }

                Ok(())
            }
        }
    }
}
