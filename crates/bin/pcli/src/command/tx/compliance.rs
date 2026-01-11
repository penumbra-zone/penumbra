use std::path::PathBuf;

use anyhow::{Context, Result};
use penumbra_sdk_asset::asset;
use penumbra_sdk_compliance::structs::{MsgRegisterAsset, MsgRegisterUser};
use penumbra_sdk_compliance::{
    scan_transaction_for_compliance_with_daily_keys, ComplianceStorage, DetectedTransfer,
};
use penumbra_sdk_custody::soft_kms::Config as SoftKmsConfig;
use penumbra_sdk_keys::keys::{DailyKeySet, KeyType, MasterComplianceKey};
use penumbra_sdk_proto::core::app::v1::{
    query_service_client::QueryServiceClient as AppQueryServiceClient, TransactionsByHeightRequest,
};
use penumbra_sdk_proto::util::tendermint_proxy::v1::{
    tendermint_proxy_service_client::TendermintProxyServiceClient, GetStatusRequest,
};
use penumbra_sdk_transaction::{ActionPlan, TransactionPlan};
use tonic::transport::Channel;
use tracing::info;
use url::Url;

use super::FeeTier;
use crate::config::CustodyConfig;

/// Compliance-related transaction commands.
#[derive(Debug, clap::Subcommand)]
pub enum ComplianceCmd {
    /// Register an asset's regulation status in the compliance registry.
    ///
    /// This marks whether an asset requires compliance (regulated) or not (unregulated).
    /// Once registered, an asset's status cannot be changed in this demo version.
    RegisterAsset {
        /// The asset ID to register (e.g., "uusdc" or a full asset ID).
        asset_id: String,
        /// Mark this asset as regulated (requires compliance ciphertexts).
        #[clap(long)]
        regulated: bool,
        /// Mark this asset as unregulated (no compliance required).
        #[clap(long, conflicts_with = "regulated")]
        unregulated: bool,
        /// The selected fee tier to multiply the fee amount by.
        #[clap(short, long, default_value_t)]
        fee_tier: FeeTier,
    },

    /// Register your wallet's compliance key for a regulated asset.
    ///
    /// This allows you to transact with regulated assets by publishing your
    /// compliance viewing key on-chain.
    RegisterUser {
        /// The asset ID to register for (e.g., "uusdc").
        asset_id: String,
        /// The selected fee tier to multiply the fee amount by.
        #[clap(short, long, default_value_t)]
        fee_tier: FeeTier,
    },

    /// Derive a daily key from a Master Compliance Key for a specific date.
    ///
    /// This command is used by the asset issuer to create time-limited keys
    /// that can be shared with auditors. The auditor can then use the daily
    /// key to scan transactions for that specific date, without having access
    /// to the full Master Compliance Key.
    ///
    /// Example workflow:
    /// 1. Issuer runs: pcli tx compliance derive-daily-key --mck-hex <MCK> --date <DAY>
    /// 2. Issuer shares the daily_key_hex with the auditor
    /// 3. Auditor runs: pcli tx compliance scan --daily-key-hex <KEY> --node <URL>
    DeriveDailyKey {
        /// Master Compliance Key as hex string (64 hex chars = 32 bytes).
        #[clap(long)]
        mck_hex: String,

        /// The day index to derive the key for (e.g., 20459 for a specific day).
        /// This is typically computed as: unix_timestamp / 86400
        #[clap(long)]
        date: u64,
    },

    /// Scan the chain for regulated asset transfers (auditor tool).
    ///
    /// This command scans blocks for compliance ciphertexts, decrypts them using
    /// a pre-derived daily key, and displays/stores the detected transfers.
    ///
    /// The daily key should be obtained from the asset issuer using the
    /// derive-daily-key command. This separation ensures auditors only have
    /// access to specific dates, not the full Master Compliance Key.
    Scan {
        /// The URL of the pd gRPC endpoint (e.g., http://localhost:8080).
        /// Can also be set via PENUMBRA_NODE_PD_URL environment variable.
        #[clap(long, env = "PENUMBRA_NODE_PD_URL")]
        node: Url,

        /// Start scanning from this block height.
        #[clap(long, default_value = "1")]
        start_height: u64,

        /// Stop scanning at this block height (default: latest).
        #[clap(long)]
        end_height: Option<u64>,

        /// Filter for a specific asset ID (optional, scans all if not specified).
        #[clap(long)]
        asset: Option<String>,

        /// Path to SQLite database for storing results (optional).
        #[clap(long)]
        db: Option<PathBuf>,

        /// Daily key as hex string (64 hex chars = 32 bytes).
        /// Obtain this from the asset issuer using derive-daily-key command.
        #[clap(long)]
        daily_key_hex: String,
    },
}

impl ComplianceCmd {
    /// Determine if this command requires a network sync before executing.
    pub fn offline(&self) -> bool {
        match self {
            ComplianceCmd::RegisterAsset { .. } => false,
            ComplianceCmd::RegisterUser { .. } => false,
            ComplianceCmd::DeriveDailyKey { .. } => true, // No network needed
            ComplianceCmd::Scan { .. } => true,           // Scanner doesn't need wallet sync
        }
    }

    /// Check if this command is a scan command (doesn't create a transaction).
    pub fn is_scan(&self) -> bool {
        matches!(self, ComplianceCmd::Scan { .. })
    }

    /// Check if this command is a derive-daily-key command (doesn't create a transaction).
    pub fn is_derive_daily_key(&self) -> bool {
        matches!(self, ComplianceCmd::DeriveDailyKey { .. })
    }

    /// Execute the derive-daily-key command (pure computation, no network).
    pub fn exec_derive_daily_key(&self) -> Result<()> {
        match self {
            ComplianceCmd::DeriveDailyKey { mck_hex, date } => {
                // Parse the MCK from hex
                let mck = parse_mck_from_hex(mck_hex)?;

                // Derive all three daily keys
                let daily_keys = mck.derive_daily_keys(*date);

                // Output each key type
                let detection_hex = hex::encode(daily_keys.detection.to_bytes());
                let core_hex = hex::encode(daily_keys.core.to_bytes());
                let extension_hex = hex::encode(daily_keys.extension.to_bytes());

                // Combined format for full scanning (all three keys concatenated)
                let full_hex = format!("{}{}{}", detection_hex, core_hex, extension_hex);

                println!("=== Daily Key Derivation ===");
                println!("Date (day index): {}", date);
                println!();
                println!("Individual keys (for selective disclosure):");
                println!("  Detection Key: {}", detection_hex);
                println!("  Core Key:      {}", core_hex);
                println!("  Extension Key: {}", extension_hex);
                println!();
                println!("Combined key (for full scanning):");
                println!("  Full Key Set:  {}", full_hex);
                println!();
                println!("To scan with full decryption:");
                println!(
                    "  pcli tx compliance scan --daily-key-hex {} --node <URL>",
                    full_hex
                );

                Ok(())
            }
            _ => anyhow::bail!("exec_derive_daily_key called on wrong command"),
        }
    }

    /// Execute the scan command directly (doesn't create a transaction).
    /// This command doesn't require wallet initialization - only a gRPC connection.
    pub async fn exec_scan(&self) -> Result<()> {
        match self {
            ComplianceCmd::Scan {
                node,
                start_height,
                end_height,
                asset,
                db,
                daily_key_hex,
            } => {
                // Parse the daily key set from hex (expects all three keys concatenated)
                let daily_keys = parse_daily_keys_from_hex(daily_key_hex)?;

                // Parse optional asset filter
                let target_asset = if let Some(asset_str) = asset {
                    Some(Self::parse_asset_id(asset_str)?)
                } else {
                    None
                };

                // Initialize storage if requested
                let storage = if let Some(db_path) = db {
                    Some(ComplianceStorage::new(db_path)?)
                } else {
                    None
                };

                // Connect to node directly (no wallet required)
                let channel = connect_to_node(node).await?;
                let end = if let Some(h) = end_height {
                    *h
                } else {
                    get_latest_height(channel.clone()).await?
                };

                println!(
                    "Scanning blocks {} to {} for regulated transfers...",
                    start_height, end
                );
                if let Some(asset_id) = &target_asset {
                    println!("Filtering for asset: {}", asset_id);
                }

                let mut total_detected = 0u64;

                // Scan each block
                for height in *start_height..=end {
                    // Fetch transactions for this block
                    let transactions = fetch_transactions(channel.clone(), height).await?;

                    for tx in transactions {
                        // Use the daily key set for full decryption
                        let detected = scan_transaction_for_compliance_with_daily_keys(
                            &tx,
                            height,
                            &daily_keys,
                            target_asset,
                        )?;

                        for transfer in &detected {
                            total_detected += 1;
                            print_transfer(transfer);

                            // Store if database is configured
                            if let Some(ref storage) = storage {
                                storage.save_transfer(transfer)?;
                            }
                        }
                    }

                    // Progress indicator every 100 blocks
                    if height % 100 == 0 {
                        info!(height, "scanning progress...");
                    }
                }

                println!("\nScan complete. Detected {} transfers.", total_detected);

                if let Some(ref storage) = storage {
                    storage.update_sync_height(end)?;
                    println!(
                        "Results saved to database. Total stored: {}",
                        storage.transfer_count()?
                    );
                }

                Ok(())
            }
            _ => anyhow::bail!("exec_scan called on non-scan command"),
        }
    }

    /// Create the transaction plan for this compliance command.
    pub async fn plan(
        &self,
        app: &mut crate::App,
        gas_prices: penumbra_sdk_fee::GasPrices,
    ) -> Result<TransactionPlan> {
        match self {
            ComplianceCmd::RegisterAsset {
                asset_id,
                regulated,
                unregulated,
                fee_tier,
            } => {
                // Determine regulation status
                let is_regulated = if *regulated {
                    true
                } else if *unregulated {
                    false
                } else {
                    anyhow::bail!("Must specify either --regulated or --unregulated");
                };

                // Parse asset ID
                let asset_id = Self::parse_asset_id(asset_id)?;

                // Create the registration message
                let msg = MsgRegisterAsset {
                    asset_id,
                    is_regulated,
                };

                // Build transaction plan
                let mut planner = penumbra_sdk_wallet::plan::Planner::new(rand_core::OsRng);
                planner
                    .set_gas_prices(gas_prices)
                    .set_fee_tier((*fee_tier).into());

                planner.action(ActionPlan::from(msg));

                Ok(planner
                    .plan(app.view(), penumbra_sdk_keys::keys::AddressIndex::new(0))
                    .await
                    .context("can't build transaction")?)
            }

            ComplianceCmd::RegisterUser { asset_id, fee_tier } => {
                // Parse asset ID
                let asset_id = Self::parse_asset_id(asset_id)?;

                // Get user's full viewing key
                let fvk = app.config.full_viewing_key.clone();

                // Get default address (account 0, address index 0)
                let address_index = penumbra_sdk_keys::keys::AddressIndex::new(0);
                let (address, _detection_key) = fvk.payment_address(address_index);

                // Derive user-specific MCK from spend key seed
                // This ensures each user has their own unique MCK for per-user isolation
                let user_mck = match &app.config.custody {
                    CustodyConfig::SoftKms(SoftKmsConfig { spend_key, .. }) => {
                        let seed = spend_key.to_bytes().0;
                        let mck = MasterComplianceKey::from_spend_seed(&seed);
                        println!("Derived user-specific MCK from wallet seed");
                        println!("   MCK (hex): {}", hex::encode(mck.to_bytes()));
                        mck
                    }
                    _ => {
                        // Non-SoftKms custody (view-only, threshold, etc.) cannot derive MCK
                        anyhow::bail!(
                            "Cannot derive compliance key: custody type doesn't have spend key. \
                             Compliance registration requires a SoftKms custody configuration."
                        );
                    }
                };

                // Derive ACK for this specific address (maximum privacy - one ACK per address)
                use penumbra_sdk_compliance::ComplianceLeaf;
                let leaf = ComplianceLeaf::new(&user_mck, address, asset_id);

                // Create registration message (signature is empty for now - filled during tx build)
                let msg = MsgRegisterUser {
                    leaf,
                    signature: vec![],
                };

                // Build transaction plan
                let mut planner = penumbra_sdk_wallet::plan::Planner::new(rand_core::OsRng);
                planner
                    .set_gas_prices(gas_prices)
                    .set_fee_tier((*fee_tier).into());

                planner.action(ActionPlan::from(msg));

                Ok(planner
                    .plan(app.view(), address_index)
                    .await
                    .context("can't build transaction")?)
            }

            ComplianceCmd::DeriveDailyKey { .. } => {
                anyhow::bail!("DeriveDailyKey command doesn't create a transaction - use exec_derive_daily_key instead")
            }

            ComplianceCmd::Scan { .. } => {
                anyhow::bail!("Scan command doesn't create a transaction - use exec_scan instead")
            }
        }
    }

    /// Helper to parse asset ID from string.
    /// Accepts either a full asset ID or a unit name like "penumbra" or "upenumbra".
    fn parse_asset_id(asset_str: &str) -> Result<asset::Id> {
        // Try to parse as a full asset ID first
        if let Ok(asset_id) = asset_str.parse() {
            return Ok(asset_id);
        }
        // Fall back to parsing as a unit name from the registry
        Ok(asset::REGISTRY.parse_unit(asset_str).id())
    }
}

/// Parse Master Compliance Key from hex string.
fn parse_mck_from_hex(hex: &str) -> Result<MasterComplianceKey> {
    let bytes = hex::decode(hex).context("invalid hex string for MCK")?;
    if bytes.len() != 32 {
        anyhow::bail!(
            "MCK must be exactly 32 bytes (64 hex chars), got {}",
            bytes.len()
        );
    }
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&bytes);
    let fr = decaf377::Fr::from_le_bytes_mod_order(&arr);
    Ok(MasterComplianceKey::new(fr))
}

/// Parse DailyKeySet from hex string (96 bytes = 3 keys × 32 bytes each).
fn parse_daily_keys_from_hex(hex: &str) -> Result<DailyKeySet> {
    use penumbra_sdk_keys::keys::DailyMasterKey;

    let bytes = hex::decode(hex).context("invalid hex string for daily key set")?;
    if bytes.len() != 96 {
        anyhow::bail!(
            "Daily key set must be exactly 96 bytes (192 hex chars = 3 keys × 32 bytes), got {} bytes",
            bytes.len()
        );
    }

    let mut detection_arr = [0u8; 32];
    let mut core_arr = [0u8; 32];
    let mut extension_arr = [0u8; 32];

    detection_arr.copy_from_slice(&bytes[0..32]);
    core_arr.copy_from_slice(&bytes[32..64]);
    extension_arr.copy_from_slice(&bytes[64..96]);

    Ok(DailyKeySet {
        detection: DailyMasterKey::from_bytes(&detection_arr, KeyType::Detection),
        core: DailyMasterKey::from_bytes(&core_arr, KeyType::Core),
        extension: DailyMasterKey::from_bytes(&extension_arr, KeyType::Extension),
    })
}

/// Connect to a Penumbra node directly (no wallet required).
async fn connect_to_node(node_url: &Url) -> Result<Channel> {
    let endpoint = tonic::transport::Endpoint::from_shared(node_url.to_string())
        .context("invalid node URL")?
        .timeout(std::time::Duration::from_secs(30));

    endpoint
        .connect()
        .await
        .context(format!("failed to connect to node at {}", node_url))
}

/// Get the latest block height from the chain using TendermintProxy.
async fn get_latest_height(channel: Channel) -> Result<u64> {
    let mut client = TendermintProxyServiceClient::new(channel);

    let response = client
        .get_status(GetStatusRequest {})
        .await
        .context("failed to query node status")?;

    let sync_info = response
        .into_inner()
        .sync_info
        .ok_or_else(|| anyhow::anyhow!("missing sync_info in status response"))?;

    Ok(sync_info.latest_block_height)
}

/// Fetch all transactions at a given height.
async fn fetch_transactions(
    channel: Channel,
    height: u64,
) -> Result<Vec<penumbra_sdk_proto::core::transaction::v1::Transaction>> {
    let mut client = AppQueryServiceClient::new(channel);

    let request = TransactionsByHeightRequest {
        block_height: height,
    };

    let response = client
        .transactions_by_height(request)
        .await
        .context("failed to fetch transactions")?;

    // TransactionsByHeightResponse contains Transaction proto messages directly
    Ok(response.into_inner().transactions)
}

/// Print a detected transfer to stdout.
fn print_transfer(transfer: &DetectedTransfer) {
    println!("─────────────────────────────────────────────────────────");
    println!("📋 Detected Transfer at height {}", transfer.height);
    println!("   Action index: {}", transfer.action_index);
    println!("   Asset: {}", transfer.asset_id);
    println!("   Amount: {}", transfer.amount);
    println!("   Self: {}", transfer.self_address);
    println!("   Counterparty: {}", transfer.counterparty_address);
}
