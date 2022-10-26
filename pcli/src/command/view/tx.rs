use anyhow::Result;
use comfy_table::{presets, Table};
use penumbra_crypto::{asset::Cache, dex::swap::SwapPlaintext, FullViewingKey, Note, Value};
use penumbra_transaction::{
    action::{Swap, SwapClaim},
    view::action_view::{OutputView, SpendView, SwapClaimView, SwapView},
};
use penumbra_view::ViewClient;

/// Queries the chain for a transaction by hash.
#[derive(Debug, clap::Args)]
pub struct TxCmd {
    /// The hex-formatted transaction hash to query.
    hash: String,
}

fn format_visible_swap_row(asset_cache: &Cache, swap: &SwapPlaintext) -> String {
    // Typical swaps are one asset for another, but we can't know that for sure.

    // For the non-pathological case:
    let (from_asset, from_value, to_asset) =
        if swap.delta_1_i.inner == 0 && swap.delta_2_i.inner > 0 {
            (
                swap.trading_pair.asset_2(),
                swap.delta_2_i,
                swap.trading_pair.asset_1(),
            )
        } else if swap.delta_2_i.inner == 0 && swap.delta_1_i.inner > 0 {
            (
                swap.trading_pair.asset_1(),
                swap.delta_1_i,
                swap.trading_pair.asset_2(),
            )
        } else {
            // The pathological case (both assets have input values).
            let value_1 = Value {
                amount: swap.delta_1_i,
                asset_id: swap.trading_pair.asset_1(),
            }
            .format(asset_cache);
            let value_2 = Value {
                amount: swap.delta_1_i,
                asset_id: swap.trading_pair.asset_1(),
            }
            .format(asset_cache);
            let value_fee = Value {
                amount: swap.claim_fee.amount(),
                asset_id: swap.claim_fee.asset_id(),
            }
            .format(asset_cache);

            return format!(
                "{} for {} and paid claim fee {}",
                value_1, value_2, value_fee,
            );
        };

    let from = Value {
        amount: from_value,
        asset_id: from_asset,
    }
    .format(asset_cache);
    let to = asset_cache.get(&to_asset).map_or_else(
        || format!("{}", to_asset),
        |to_denom| format!("{}", to_denom),
    );
    let value_fee = Value {
        amount: swap.claim_fee.amount(),
        asset_id: swap.claim_fee.asset_id(),
    }
    .format(asset_cache);

    format!("{} for {} and paid claim fee {}", from, to, value_fee)
}

fn format_opaque_swap_row(swap: &Swap) -> String {
    // An opaque swap has no plaintext amount information for us to display, how sad.
    format!(
        "Opaque swap for trading pair: {} <=> {}",
        swap.body.trading_pair.asset_1(),
        swap.body.trading_pair.asset_2()
    )
}

fn format_opaque_swap_claim_row(asset_cache: &Cache, swap: &SwapClaim) -> String {
    // An opaque swap claim has no plaintext amount information for us to display, how sad.
    let value_fee = Value {
        amount: swap.body.fee.amount(),
        asset_id: swap.body.fee.asset_id(),
    }
    .format(asset_cache);
    format!(
        "Opaque swap claim for trading pair: {} <=> {} with fee {}",
        swap.body.output_data.trading_pair.asset_1(),
        swap.body.output_data.trading_pair.asset_2(),
        value_fee,
    )
}

fn format_visible_swap_claim_row(
    asset_cache: &Cache,
    swap: &SwapClaim,
    note_1: &Note,
    note_2: &Note,
) -> String {
    // Typical swap claims only have a single output note with value, but we can't know that for sure.

    let value_fee = Value {
        amount: swap.body.fee.amount(),
        asset_id: swap.body.fee.asset_id(),
    }
    .format(asset_cache);

    // For the non-pathological case:
    let claimed_value = if note_1.amount().inner == 0 && note_2.amount().inner > 0 {
        note_2.value()
    } else if note_2.amount().inner == 0 && note_1.amount().inner > 0 {
        note_1.value()
    } else {
        // The pathological case (both assets have output values).
        return format!(
            "Claimed {} and {} with fee {}",
            note_1.value().format(asset_cache),
            note_2.value().format(asset_cache),
            value_fee,
        );
    };

    format!(
        "Claimed {} with fee {}",
        claimed_value.format(asset_cache),
        value_fee
    )
}

fn format_visible_output_row(asset_cache: &Cache, decrypted_note: &Note) -> String {
    format!(
        "{} to address {}",
        decrypted_note.value().format(asset_cache),
        decrypted_note.address(),
    )
}

fn format_visible_spend_row(asset_cache: &Cache, decrypted_note: &Note) -> String {
    format!(
        "address {} spent {}",
        decrypted_note.address(),
        decrypted_note.value().format(asset_cache),
    )
}

impl TxCmd {
    pub fn offline(&self) -> bool {
        false
    }
    pub async fn exec<V: ViewClient>(&self, _fvk: &FullViewingKey, view: &mut V) -> Result<()> {
        // Initialize the tables
        let mut actions_table = Table::new();
        actions_table.load_preset(presets::NOTHING);
        actions_table.set_header(vec!["Action Type", "Description"]);

        let mut metadata_table = Table::new();
        metadata_table.load_preset(presets::NOTHING);
        metadata_table.set_header(vec!["", ""]);

        // Retrieve Transaction
        let tx = view.transaction_by_hash(self.hash.parse()?).await?;

        if let Some(tx) = &tx {
            // Retrieve full TxP
            let txp = view.transaction_perspective(self.hash.parse()?).await?;

            // Generate TxV using TxP

            let txv = tx.decrypt_with_perspective(&txp);

            let asset_cache = view.assets().await?;
            // Iterate over the ActionViews in the TxV & display as appropriate

            for av in txv.actions {
                actions_table.add_row(match av {
                    penumbra_transaction::ActionView::Swap(SwapView::Visible {
                        swap: _,
                        swap_nft: _,
                        swap_plaintext,
                    }) => [
                        "Swap".to_string(),
                        format_visible_swap_row(&asset_cache, &swap_plaintext),
                    ],
                    penumbra_transaction::ActionView::Swap(SwapView::Opaque { swap }) => {
                        ["Swap".to_string(), format_opaque_swap_row(&swap)]
                    }
                    penumbra_transaction::ActionView::SwapClaim(SwapClaimView::Visible {
                        swap_claim,
                        decrypted_note_1,
                        decrypted_note_2,
                    }) => [
                        "Swap Claim".to_string(),
                        format_visible_swap_claim_row(
                            &asset_cache,
                            &swap_claim,
                            &decrypted_note_1,
                            &decrypted_note_2,
                        ),
                    ],
                    penumbra_transaction::ActionView::SwapClaim(SwapClaimView::Opaque {
                        swap_claim,
                    }) => [
                        "Swap Claim".to_string(),
                        format_opaque_swap_claim_row(&asset_cache, &swap_claim),
                    ],

                    penumbra_transaction::ActionView::Output(OutputView::Visible {
                        output: _,
                        decrypted_note,
                        decrypted_memo_key: _,
                    }) => [
                        "Output".to_string(),
                        format_visible_output_row(&asset_cache, &decrypted_note),
                    ],
                    penumbra_transaction::ActionView::Output(OutputView::Opaque { output: _ }) => {
                        ["Output".to_string(), "Opaque output".to_string()]
                    }
                    penumbra_transaction::ActionView::Spend(SpendView::Visible {
                        spend: _,
                        note,
                    }) => [
                        "Spend".to_string(),
                        format_visible_spend_row(&asset_cache, &note),
                    ],
                    penumbra_transaction::ActionView::Spend(SpendView::Opaque { spend: _ }) => {
                        ["Spend".to_string(), "Opaque spend".to_string()]
                    }
                    penumbra_transaction::ActionView::Delegate(_) => {
                        ["Delegation".to_string(), "".to_string()]
                    }
                    penumbra_transaction::ActionView::Undelegate(_) => {
                        ["Undelegation".to_string(), "".to_string()]
                    }
                    penumbra_transaction::ActionView::ValidatorDefinition(_) => {
                        ["Upload Validator Definition".to_string(), "".to_string()]
                    }
                    penumbra_transaction::ActionView::IBCAction(_) => {
                        ["IBC Action".to_string(), "".to_string()]
                    }
                    penumbra_transaction::ActionView::ProposalSubmit(_) => {
                        ["Submit Governance Proposal".to_string(), "".to_string()]
                    }
                    penumbra_transaction::ActionView::ProposalWithdraw(_) => {
                        ["Governance Withdrawal Proposal".to_string(), "".to_string()]
                    }
                    penumbra_transaction::ActionView::ValidatorVote(_) => {
                        ["Validator Vote".to_string(), "".to_string()]
                    }
                    penumbra_transaction::ActionView::PositionOpen(_) => {
                        ["Open Liquidity Position".to_string(), "".to_string()]
                    }
                    penumbra_transaction::ActionView::PositionClose(_) => {
                        ["Close Liquidity Position".to_string(), "".to_string()]
                    }
                    penumbra_transaction::ActionView::PositionWithdraw(_) => {
                        ["Withdraw Liquidity Position".to_string(), "".to_string()]
                    }
                    penumbra_transaction::ActionView::PositionRewardClaim(_) => [
                        "Claim Liquidity Position Reward".to_string(),
                        "".to_string(),
                    ],
                    penumbra_transaction::ActionView::ICS20Withdrawal(_) => {
                        ["ICS20 Withdrawal".to_string(), "".to_string()]
                    }
                });
            }

            metadata_table.add_row(vec![
                "Transaction Fee",
                &txv.fee.value().format(&asset_cache),
            ]);
            if let Some(memo) = txv.memo {
                metadata_table.add_row(vec!["Transaction Memo", &memo]);
            }
            metadata_table.add_row(vec![
                "Transaction Expiration Height",
                &format!("{}", txv.expiry_height),
            ]);
        }

        // Print table of actions and their descriptions
        println!("{}", actions_table);

        // Print transaction metadata
        println!("{}", metadata_table);

        Ok(())
    }
}
