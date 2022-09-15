use anyhow::Result;
use comfy_table::{presets, Table};
use penumbra_crypto::FullViewingKey;
use penumbra_transaction::{
    view::action_view::{OutputView, SpendView, SwapClaimView, SwapView},
    Transaction,
};
use penumbra_view::ViewClient;

/// Queries the chain for a transaction by hash.
#[derive(Debug, clap::Args)]
pub struct TxCmd {
    /// The hex-formatted transaction hash to query.
    hash: String,
}

impl TxCmd {
    pub fn needs_sync(&self) -> bool {
        true
    }
    pub async fn exec<V: ViewClient>(&self, fvk: &FullViewingKey, view: &mut V) -> Result<()> {
        // Initialize the table
        let mut table = Table::new();
        table.load_preset(presets::NOTHING);
        table.set_header(vec!["Action Type", "Net Change"]);

        // Retrieve Transaction
        let tx = view
            .transaction_by_hash(self.hash.as_bytes())
            .await?
            .unwrap_or(Transaction {
                transaction_body: todo!(),
                binding_sig: todo!(),
                anchor: todo!(),
            });

        // Generate full TxP using FvK

        let txp = view
            .generate_full_perspective(self.hash.as_bytes(), fvk.to_owned())
            .await?;

        // Generate TxV using TxP

        let txv = tx.decrypt_with_perspective(&txp);

        // Iterate over the ActionViews in the TxV & display as appropriate

        table.add_row(vec![
            format!("{}", "Action Type"),
            format!("{:?}", "Net Change"),
        ]);

        for av in txv.actions {
            table.add_row(vec![match av {
                penumbra_transaction::ActionView::Swap(swap_view) => match swap_view {
                    SwapView::Visible {
                        swap,
                        swap_nft,
                        swap_plaintext,
                    } => format!("{:?}", "Swap"),
                    SwapView::Opaque { swap } => format!("{:?}", "Swap"),
                },
                penumbra_transaction::ActionView::SwapClaim(swap_claim_view) => {
                    match swap_claim_view {
                        SwapClaimView::Visible {
                            swap_claim,
                            decrypted_note_1,
                            decrypted_note_2,
                        } => format!("{:?}", "SwapClaim"),
                        SwapClaimView::Opaque { swap_claim } => format!("{:?}", "SwapClaim"),
                    }
                }
                penumbra_transaction::ActionView::Output(output_view) => match output_view {
                    OutputView::Visible {
                        output,
                        decrypted_note,
                        decrypted_memo_key,
                    } => format!("{:?}", "Output"),
                    OutputView::Opaque { output } => format!("{:?}", "Output"),
                },
                penumbra_transaction::ActionView::Spend(spend_view) => match spend_view {
                    SpendView::Visible { spend, note } => format!("{:?}", "Spend"),
                    SpendView::Opaque { spend } => format!("{:?}", "Spend"),
                },
                _ => String::from(""),
            }]);
        }

        // Print table of actions and their change to the balance
        println!("{}", table);

        // Print total change for entire tx

        Ok(())
    }
}
