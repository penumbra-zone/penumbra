use anyhow::Result;
use penumbra_crypto::Value;
use penumbra_wallet::plan;
use rand_core::OsRng;

use crate::App;

#[derive(Debug, clap::Subcommand)]
pub enum TxCmd {
    /// Send transaction to the node.
    Send {
        /// The destination address to send funds to.
        #[clap(long)]
        to: String,
        /// The amounts to send, written as typed values 1.87penumbra, 12cubes, etc.
        values: Vec<String>,
        /// The transaction fee (paid in upenumbra).
        #[clap(long, default_value = "0")]
        fee: u64,
        /// Optional. Only spend funds originally received by the given address index.
        #[clap(long)]
        source: Option<u64>,
        /// Optional. Set the transaction's memo field to the provided text.
        #[clap(long)]
        memo: Option<String>,
    },
    /// Sweeps small notes of the same denomination into a few larger notes.
    ///
    /// Since Penumbra transactions reveal their arity (how many spends,
    /// outputs, etc), but transactions are unlinkable from each other, it is
    /// slightly preferable to sweep small notes into larger ones in an isolated
    /// "sweep" transaction, rather than at the point that they should be spent.
    ///
    /// Currently, only zero-fee sweep transactions are implemented.
    Sweep,
    /// Submit a new Swap to the chain which will burn input assets and allow a future SwapClaim for the given Swap NFT.
    /// Only the first asset has an input amount specified, as in typical usage, the second asset is always
    /// the asset that the submitter wants to swap the first for.
    ///
    /// Swaps are automatically claimed.
    Swap {
        /// Asset ID of the first input asset.
        asset_1_id: String,
        /// The amount of asset 1 to burn as part of the swap.
        asset_1_input_amount: String,
        /// Asset ID of the second input asset.
        asset_2_id: String,
        /// The transaction fee (paid in upenumbra).
        #[clap(long, default_value = "0")]
        fee: u64,
        /// Optional. Only spend funds originally received by the given address index.
        #[clap(long)]
        source: Option<u64>,
    },
}

impl TxCmd {
    /// Determine if this command requires a network sync before it executes.
    pub fn needs_sync(&self) -> bool {
        match self {
            TxCmd::Send { .. } => true,
            TxCmd::Sweep { .. } => true,
            TxCmd::Swap { .. } => true,
        }
    }

    pub async fn exec(&self, app: &mut App) -> Result<()> {
        match self {
            TxCmd::Send {
                values,
                to,
                fee,
                source: from,
                memo,
            } => {
                // Parse all of the values provided.
                let values = values
                    .iter()
                    .map(|v| v.parse())
                    .collect::<Result<Vec<Value>, _>>()?;
                let to = to
                    .parse()
                    .map_err(|_| anyhow::anyhow!("address is invalid"))?;

                let plan = plan::send(
                    &app.fvk,
                    &mut app.view,
                    OsRng,
                    &values,
                    *fee,
                    to,
                    *from,
                    memo.clone(),
                )
                .await?;
                app.build_and_submit_transaction(plan).await?;
            }
            TxCmd::Sweep => loop {
                let plans = plan::sweep(&app.fvk, &mut app.view, OsRng).await?;
                let num_plans = plans.len();

                for (i, plan) in plans.into_iter().enumerate() {
                    println!("building sweep {} of {}", i, num_plans);
                    let tx = app.build_transaction(plan).await?;
                    app.submit_transaction_unconfirmed(&tx).await?;
                }
                if num_plans == 0 {
                    println!("finished sweeping");
                    break;
                } else {
                    println!("awaiting confirmations...");
                    tokio::time::sleep(std::time::Duration::from_secs(6)).await;
                }
            },
            TxCmd::Swap {
                asset_1_id,
                asset_1_input_amount,
                asset_2_id,
                fee,
                source,
            } => {
                println!("Sorry, this command is not yet implemented");
            }
        }
        Ok(())
    }
}
