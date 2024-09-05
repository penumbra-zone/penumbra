use crate::App;
use anyhow::{Context, Result};
use penumbra_keys::FullViewingKey;
use penumbra_proto::view::v1::GasPricesRequest;
use penumbra_view::ViewClient;
use penumbra_wallet::plan::Planner;
use rand_core::OsRng;
use std::{io::Write, str::FromStr};
use termion::input::TermRead;

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

        print!("Enter FVK: ");
        std::io::stdout().flush()?;
        let to: String = std::io::stdin().lock().read_line()?.unwrap_or_default();

        match self {
            MigrateCmd::Balance => {
                let source_fvk = app.config.full_viewing_key.clone();

                let dest_fvk = to.parse::<FullViewingKey>().map_err(|_| {
                    anyhow::anyhow!("The provided string is not a valid FullViewingKey.")
                })?;

                let mut planner = Planner::new(OsRng);

                let (dest_address, _) = FullViewingKey::payment_address(
                    &FullViewingKey::from_str(&to[..])?,
                    Default::default(),
                );

                planner
                    .set_gas_prices(gas_prices)
                    .set_fee_tier(Default::default())
                    .change_address(dest_address);

                // Return all unspent notes from the view service
                let notes = app
                    .view
                    .as_mut()
                    .context("view service must be initialized")?
                    .unspent_notes_by_account_and_asset()
                    .await?;

                for notes in notes.into_values() {
                    for notes in notes.into_values() {
                        for note in notes {
                            planner.spend(note.note, note.position);
                        }
                    }
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
