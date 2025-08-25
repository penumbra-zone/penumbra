use crate::App;
use anyhow::{anyhow, Context, Result};
use futures::TryStreamExt;
use penumbra_sdk_asset::{asset, Value, STAKING_TOKEN_ASSET_ID};
use penumbra_sdk_keys::FullViewingKey;
use penumbra_sdk_num::Amount;
use penumbra_sdk_proto::view::v1::{AssetsRequest, GasPricesRequest};
use penumbra_sdk_view::ViewClient;
use penumbra_sdk_wallet::plan::Planner;
use rand_core::OsRng;
use std::{collections::HashMap, io::Write};
use termion::input::TermRead;

fn read_fvk() -> Result<FullViewingKey> {
    print!("Enter FVK: ");
    std::io::stdout().flush()?;
    let fvk_string: String = std::io::stdin().lock().read_line()?.unwrap_or_default();

    fvk_string
        .parse::<FullViewingKey>()
        .map_err(|_| anyhow::anyhow!("The provided string is not a valid FullViewingKey."))
}

fn parse_range(s: &str) -> Result<std::ops::Range<u32>> {
    let parts: Vec<&str> = s.split("..").collect();
    if parts.len() != 2 {
        return Err(anyhow!("Invalid range format. Expected format: start..end"));
    }

    let start = parts[0]
        .parse::<u32>()
        .context("Invalid start value in range")?;
    let end = parts[1]
        .parse::<u32>()
        .context("Invalid end value in range")?;

    if start >= end {
        return Err(anyhow!("Invalid range: start must be less than end"));
    }

    Ok(start..end)
}

#[derive(Debug, clap::Parser)]
pub enum MigrateCmd {
    /// Migrate your entire balance to another wallet.
    ///
    /// All assets from all accounts in the source wallet will be sent to the destination wallet.
    /// A FullViewingKey must be provided for the destination wallet.
    /// All funds will be deposited in the account 0 of the destination wallet,
    /// minus any gas prices for the migration transaction.
    #[clap(name = "balance")]
    Balance,
    /// Migrate balances from specified subaccounts to a destination wallet.
    ///
    /// All assets from the specified source subaccounts will be sent to the destination wallet.
    /// A FullViewingKey must be provided for the destination wallet.
    /// Gas fees will be paid from the source subaccount with the most fee token.
    #[clap(name = "subaccount-balance")]
    SubaccountBalance {
        /// Range of source subaccount indices to migrate from (e.g., 0..17)
        ///
        /// The range is inclusive of the `start` value, and exclusive of the `end` value,
        /// such that `1..4` will migrate subaccounts 1, 2, & 3, but not 4. Therefore
        /// to migrate only subaccount 4, use `4..5`.
        #[clap(long, required = true, value_parser = parse_range, name = "subaccount_index_range")]
        from_range: std::ops::Range<u32>,
        /// Only print the transaction plan without executing it (for threshold signing)
        #[clap(long)]
        plan_only: bool,
    },
}

impl MigrateCmd {
    #[tracing::instrument(skip(self, app))]
    pub async fn exec(&self, app: &mut App) -> Result<()> {
        let gas_prices = app
            .view
            .as_mut()
            .context("view service must be initialized")?
            .gas_prices(GasPricesRequest {})
            .await?
            .into_inner()
            .gas_prices
            .expect("gas prices must be available")
            .try_into()?;

        match self {
            MigrateCmd::Balance => {
                let source_fvk = app.config.full_viewing_key.clone();

                let dest_fvk = read_fvk()?;

                let mut planner = Planner::new(OsRng);

                planner
                    .set_gas_prices(gas_prices)
                    .set_fee_tier(Default::default());

                // Return all unspent notes from the view service
                let notes = app
                    .view
                    .as_mut()
                    .context("view service must be initialized")?
                    .unspent_notes_by_account_and_asset()
                    .await?;

                let mut account_values: HashMap<(u32, asset::Id), Amount> = HashMap::new();

                for (account, notes) in notes {
                    for notes in notes.into_values() {
                        for note in notes {
                            let position = note.position;
                            let note = note.note;
                            let value = note.value();
                            planner.spend(note, position);
                            *account_values.entry((account, value.asset_id)).or_default() +=
                                value.amount;
                        }
                    }
                }

                // We'll use the account with the most amount of the fee token to pay fees.
                //
                // If this fails, then it won't be possible to migrate.
                let (&(largest_account, _), _) = account_values
                    .iter()
                    .filter(|((_, asset), _)| *asset == *STAKING_TOKEN_ASSET_ID)
                    .max_by_key(|&(_, &amount)| amount)
                    .ok_or(anyhow!("no account with the ability to pay fees exists"))?;

                // Set this account to be the change address.
                planner.change_address(dest_fvk.payment_address(largest_account.into()).0);

                // Create explicit outputs for the other addresses.
                for (&(account, asset_id), &amount) in &account_values {
                    if account == largest_account {
                        continue;
                    }
                    let (address, _) = dest_fvk.payment_address(account.into());
                    planner.output(Value { asset_id, amount }, address);
                }

                let memo = format!("Migrating balance from {} to {}", source_fvk, dest_fvk);
                let plan = planner
                    .memo(memo)
                    .plan(
                        app.view
                            .as_mut()
                            .context("view service must be initialized")?,
                        Default::default(),
                    )
                    .await
                    .context("can't build send transaction")?;

                if plan.actions.is_empty() {
                    anyhow::bail!("migration plan contained zero actions: is the source wallet already empty?");
                }
                app.build_and_submit_transaction(plan).await?;

                Result::Ok(())
            }
            MigrateCmd::SubaccountBalance {
                from_range,
                plan_only,
            } => {
                let source_fvk = app.config.full_viewing_key.clone();

                // Read destination FVK from stdin
                let dest_fvk = read_fvk()?;

                let mut planner = Planner::new(OsRng);
                planner
                    .set_gas_prices(gas_prices)
                    .set_fee_tier(Default::default());

                // Get asset cache for human-readable denominations
                let assets_response = app
                    .view
                    .as_mut()
                    .context("view service must be initialized")?
                    .assets(AssetsRequest {
                        filtered: false,
                        include_specific_denominations: vec![],
                        include_lp_nfts: true,
                        include_delegation_tokens: true,
                        include_unbonding_tokens: true,
                        include_proposal_nfts: false,
                        include_voting_receipt_tokens: false,
                    })
                    .await?;

                // Build asset cache from the response
                let mut asset_cache = penumbra_sdk_asset::asset::Cache::default();
                let assets_stream = assets_response.into_inner();
                let assets = assets_stream
                    .try_collect::<Vec<_>>()
                    .await
                    .context("failed to collect assets")?;
                for asset_response in assets {
                    if let Some(denom) = asset_response.denom_metadata {
                        let metadata =
                            denom.try_into().context("failed to parse asset metadata")?;
                        asset_cache.extend(std::iter::once(metadata));
                    }
                }

                // Return all unspent notes from the view service
                let all_notes = app
                    .view
                    .as_mut()
                    .context("view service must be initialized")?
                    .unspent_notes_by_account_and_asset()
                    .await?;

                // Track values per (subaccount, asset) for fee calculation
                let mut subaccount_values: HashMap<(u32, asset::Id), Amount> = HashMap::new();

                // Filter and spend notes only from subaccounts in the specified range
                for (account, notes_by_asset) in all_notes {
                    if from_range.contains(&account) {
                        for notes in notes_by_asset.into_values() {
                            for note in notes {
                                let position = note.position;
                                let note = note.note;
                                let value = note.value();
                                planner.spend(note, position);
                                *subaccount_values
                                    .entry((account, value.asset_id))
                                    .or_default() += value.amount;
                            }
                        }
                    }
                }

                if subaccount_values.is_empty() {
                    anyhow::bail!("no notes found in the specified subaccount range");
                }

                // Find the subaccount with the most fee token to pay fees
                let (&(fee_account, _), _) = subaccount_values
                    .iter()
                    .filter(|((_, asset), _)| *asset == *STAKING_TOKEN_ASSET_ID)
                    .max_by_key(|&(_, &amount)| amount)
                    .ok_or(anyhow!(
                        "no subaccount in the range has the ability to pay fees"
                    ))?;

                // Set the change address to the destination's corresponding subaccount
                planner.change_address(dest_fvk.payment_address(fee_account.into()).0);

                // Create outputs for all assets to their corresponding destination subaccounts
                for (&(account, asset_id), &amount) in &subaccount_values {
                    // Skip empty values
                    if amount == Amount::zero() {
                        continue;
                    }

                    // For the fee account, the change will handle the remaining balance
                    if account == fee_account && asset_id == *STAKING_TOKEN_ASSET_ID {
                        continue;
                    }

                    let (dest_address, _) = dest_fvk.payment_address(account.into());
                    planner.output(Value { asset_id, amount }, dest_address);
                }

                let memo = format!(
                    "Migrating subaccounts {}..{} from {} to {}",
                    from_range.start, from_range.end, source_fvk, dest_fvk
                );

                let plan = planner
                    .memo(memo)
                    .plan(
                        app.view
                            .as_mut()
                            .context("view service must be initialized")?,
                        Default::default(),
                    )
                    .await
                    .context("can't build migration transaction")?;

                if plan.actions.is_empty() {
                    anyhow::bail!("migration plan contained zero actions: are the source subaccounts already empty?");
                }

                // Print migration summary
                println!("\n=== Migration Summary ===");
                println!("Source wallet: {}", source_fvk);
                println!("Destination wallet: {}", dest_fvk);
                println!(
                    "Subaccounts: {} through {} (inclusive)",
                    from_range.start, from_range.end
                );

                // Calculate total assets across all subaccounts
                let mut asset_summary: HashMap<asset::Id, Amount> = HashMap::new();
                for ((_, asset_id), amount) in &subaccount_values {
                    *asset_summary.entry(*asset_id).or_default() += *amount;
                }

                // Show assets being migrated with human-readable denominations
                println!("\nAssets to migrate:");
                let mut total_outputs = 0;
                for (asset_id, total_amount) in &asset_summary {
                    if *total_amount > Amount::zero() {
                        let value = penumbra_sdk_asset::Value {
                            asset_id: *asset_id,
                            amount: *total_amount,
                        };
                        println!("  • {}", value.format(&asset_cache));
                        total_outputs += 1;
                    }
                }

                println!("Total outputs: {}", total_outputs);

                // Show which subaccounts contain assets
                println!("\nSubaccounts with balances:");
                let mut accounts_with_balance: Vec<u32> = subaccount_values
                    .keys()
                    .map(|(account, _)| *account)
                    .collect::<std::collections::HashSet<_>>()
                    .into_iter()
                    .collect();
                accounts_with_balance.sort();

                for account in &accounts_with_balance {
                    println!("  • Subaccount {}", account);
                }

                println!("\nFee-paying subaccount: {}", fee_account);
                println!("Total distinct assets: {}", asset_summary.len());
                println!("========================\n");

                if *plan_only {
                    println!("{}", serde_json::to_string_pretty(&plan)?);
                } else {
                    // Ask for confirmation
                    print!("Send transaction? (Y/N): ");
                    std::io::stdout().flush()?;

                    let response: String = std::io::stdin().lock().read_line()?.unwrap_or_default();
                    let trimmed = response.trim().to_lowercase();

                    if trimmed == "y" || trimmed == "yes" {
                        app.build_and_submit_transaction(plan).await?;
                    } else {
                        println!("Transaction cancelled.");
                    }
                }

                Result::Ok(())
            }
        }
    }
}
