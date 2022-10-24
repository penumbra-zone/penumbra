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
    pub fn offline(&self) -> bool {
        true
    }
    pub async fn exec<V: ViewClient>(&self, fvk: &FullViewingKey, view: &mut V) -> Result<()> {
        // Initialize the table
        let mut table = Table::new();
        table.load_preset(presets::NOTHING);
        table.set_header(vec!["Action Type", "Net Change"]);

        // Retrieve Transaction
        let tx = view.transaction_by_hash(self.hash.as_bytes()).await?;

        table.add_row(vec![
            format!("{}", "Action Type"),
            format!("{:?}", "Net Change"),
        ]);

        if let Some(tx) = &tx {
            // Retrieve full TxP

            let txp = view.perspective(self.hash.as_bytes()).await?;

            // Generate TxV using TxP

            let txv = tx.decrypt_with_perspective(&txp);

            // Iterate over the ActionViews in the TxV & display as appropriate

            for av in txv.actions {
                table.add_row(vec![match av {
                    penumbra_transaction::ActionView::Swap(SwapView::Visible {
                        swap,
                        swap_nft,
                        swap_plaintext,
                    }) => format!("{:?} {:?} {:?}", swap, swap_nft, swap_plaintext),
                    penumbra_transaction::ActionView::Swap(SwapView::Opaque { swap }) => {
                        format!("{:?}", swap)
                    }
                    penumbra_transaction::ActionView::SwapClaim(SwapClaimView::Visible {
                        swap_claim,
                        decrypted_note_1,
                        decrypted_note_2,
                    }) => format!(
                        "{:?} {:?} {:?}",
                        swap_claim, decrypted_note_1, decrypted_note_2
                    ),
                    penumbra_transaction::ActionView::SwapClaim(SwapClaimView::Opaque {
                        swap_claim,
                    }) => format!("{:?}", swap_claim),

                    penumbra_transaction::ActionView::Output(OutputView::Visible {
                        output,
                        decrypted_note,
                        decrypted_memo_key,
                    }) => format!("{:?} {:?} {:?}", output, decrypted_note, decrypted_memo_key),
                    penumbra_transaction::ActionView::Output(OutputView::Opaque { output }) => {
                        format!("{:?}", output)
                    }
                    penumbra_transaction::ActionView::Spend(SpendView::Visible { spend, note }) => {
                        format!("{:?} {:?}", spend, note)
                    }
                    penumbra_transaction::ActionView::Spend(SpendView::Opaque { spend }) => {
                        format!("{:?}", spend)
                    }
                    _ => String::from(""),
                }]);
            }
        }

        // Print table of actions and their change to the balance
        println!("{}", table);

        // Print total change for entire tx

        Ok(())
    }
}
