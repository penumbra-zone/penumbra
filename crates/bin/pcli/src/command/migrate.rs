use crate::App;
use anyhow::{anyhow, Context, Result};
use penumbra_sdk_asset::{asset, Value, STAKING_TOKEN_ASSET_ID};
use penumbra_sdk_keys::FullViewingKey;
use penumbra_sdk_num::Amount;
use penumbra_sdk_proto::view::v1::GasPricesRequest;
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
        }
    }
}
