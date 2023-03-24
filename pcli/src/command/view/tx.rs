use anyhow::{Context, Result};
use comfy_table::{presets, Table};
use penumbra_crypto::{
    asset::Cache, dex::swap::SwapPlaintext, keys::IncomingViewingKey, Address, Note, NoteView,
    Value,
};
use penumbra_proto::{client::v1alpha1::GetTxRequest, DomainType};
use penumbra_transaction::{
    action::{Swap, SwapClaim},
    view::action_view::{OutputView, SpendView, SwapClaimView, SwapView},
    Transaction,
};
use penumbra_view::ViewClient;

use crate::App;

/// Queries the chain for a transaction by hash.
#[derive(Debug, clap::Args)]
pub struct TxCmd {
    /// The hex-formatted transaction hash to query.
    hash: String,
    /// If set, print the raw transaction view rather than a formatted table.
    #[clap(long)]
    raw: bool,
}

fn format_visible_swap_row(asset_cache: &Cache, swap: &SwapPlaintext) -> String {
    // Typical swaps are one asset for another, but we can't know that for sure.

    // For the non-pathological case:
    let (from_asset, from_value, to_asset) =
        if swap.delta_1_i.value() == 0 && swap.delta_2_i.value() > 0 {
            (
                swap.trading_pair.asset_2(),
                swap.delta_2_i,
                swap.trading_pair.asset_1(),
            )
        } else if swap.delta_2_i.value() == 0 && swap.delta_1_i.value() > 0 {
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

            return format!("{value_1} for {value_2} and paid claim fee {value_fee}",);
        };

    let from = Value {
        amount: from_value,
        asset_id: from_asset,
    }
    .format(asset_cache);
    let to = asset_cache
        .get(&to_asset)
        .map_or_else(|| format!("{to_asset}"), |to_denom| format!("{to_denom}"));
    let value_fee = Value {
        amount: swap.claim_fee.amount(),
        asset_id: swap.claim_fee.asset_id(),
    }
    .format(asset_cache);

    format!("{from} for {to} and paid claim fee {value_fee}")
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
    let claimed_value = if note_1.amount().value() == 0 && note_2.amount().value() > 0 {
        note_2.value()
    } else if note_2.amount().value() == 0 && note_1.amount().value() > 0 {
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

fn format_visible_output_row(
    asset_cache: &Cache,
    ivk: &IncomingViewingKey,
    decrypted_note: &NoteView,
) -> String {
    format!(
        "{} to {}",
        decrypted_note.value.value().format(asset_cache),
        format_address(ivk, &decrypted_note.address.address()),
    )
}

fn format_visible_spend_row(
    asset_cache: &Cache,
    ivk: &IncomingViewingKey,
    decrypted_note: &NoteView,
) -> String {
    format!(
        "{} spent {}",
        format_address(ivk, &decrypted_note.address.address()),
        decrypted_note.value.value().format(asset_cache),
    )
}

// Turns an `Address` into a `String` representation; either a short-form for addresses
// not associated with the `ivk`, or in the form of `[account: {account}]` for
// addresses associated with the `ivk`.
fn format_address(ivk: &IncomingViewingKey, address: &Address) -> String {
    if ivk.views_address(address) {
        let account = ivk.index_for_diversifier(address.diversifier()).account;

        format!("[account {account:?}]")
    } else {
        address.display_short_form()
    }
}

impl TxCmd {
    pub fn offline(&self) -> bool {
        false
    }
    pub async fn exec(&self, app: &mut App) -> Result<()> {
        let fvk = app.fvk.clone();
        let hash = self
            .hash
            // We have to convert to uppercase because `tendermint::Hash` only accepts uppercase :(
            .to_uppercase()
            .parse()
            .context("invalid transaction hash")?;

        // Retrieve Transaction from the view service first, or else the fullnode
        let tx = if let Some(tx) = app.view().transaction_by_hash(hash).await? {
            tx
        } else {
            println!("Transaction not found in view service, fetching from fullnode...");
            // Fall back to fetching from fullnode
            let mut client = app.tendermint_proxy_client().await?;
            let rsp = client
                .get_tx(GetTxRequest {
                    hash: hex::decode(self.hash.clone())?,
                    prove: false,
                })
                .await?;

            let rsp = rsp.into_inner();
            let tx = Transaction::decode(rsp.tx.as_slice())?;
            tx
        };
        // Retrieve full TxP
        let txp = app.view().transaction_perspective(hash).await?;
        // Generate TxV using TxP
        let txv = tx.view_from_perspective(&txp);

        if self.raw {
            use colored_json::prelude::*;
            println!(
                "{}",
                serde_json::to_string_pretty(&txv)?.to_colored_json_auto()?
            );
        } else {
            // Initialize the tables
            let mut actions_table = Table::new();
            actions_table.load_preset(presets::NOTHING);
            actions_table.set_header(vec!["Action Type", "Description"]);

            let mut metadata_table = Table::new();
            metadata_table.load_preset(presets::NOTHING);
            metadata_table.set_header(vec!["", ""]);

            let asset_cache = app.view().assets().await?;
            // Iterate over the ActionViews in the TxV & display as appropriate

            for av in txv.body_view.action_views {
                actions_table.add_row(match av {
                    penumbra_transaction::ActionView::Swap(SwapView::Visible {
                        swap: _,
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
                        output_1,
                        output_2,
                    }) => [
                        "Swap Claim".to_string(),
                        format_visible_swap_claim_row(
                            &asset_cache,
                            &swap_claim,
                            &output_1.note()?,
                            &output_2.note()?,
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
                        note,
                        payload_key: _,
                    }) => [
                        "Output".to_string(),
                        format_visible_output_row(&asset_cache, fvk.incoming(), &note),
                    ],
                    penumbra_transaction::ActionView::Output(OutputView::Opaque { output: _ }) => {
                        ["Output".to_string(), "[?] to [?]".to_string()]
                    }
                    penumbra_transaction::ActionView::Spend(SpendView::Visible {
                        spend: _,
                        note,
                    }) => [
                        "Spend".to_string(),
                        format_visible_spend_row(&asset_cache, fvk.incoming(), &note),
                    ],
                    penumbra_transaction::ActionView::Spend(SpendView::Opaque { spend: _ }) => {
                        ["Spend".to_string(), "[?] spent [?]".to_string()]
                    }
                    penumbra_transaction::ActionView::Delegate(_) => {
                        ["Delegation".to_string(), "".to_string()]
                    }
                    penumbra_transaction::ActionView::Undelegate(_) => {
                        ["Undelegation".to_string(), "".to_string()]
                    }
                    penumbra_transaction::ActionView::UndelegateClaim(_) => {
                        ["Undelegation Claim".to_string(), "".to_string()]
                    }
                    penumbra_transaction::ActionView::ValidatorDefinition(_) => {
                        ["Upload Validator Definition".to_string(), "".to_string()]
                    }
                    penumbra_transaction::ActionView::IBCAction(_) => {
                        ["IBC Action".to_string(), "".to_string()]
                    }
                    penumbra_transaction::ActionView::ProposalSubmit(prop_submit) => [
                        format!("Submit Governance Proposal #{}", prop_submit.proposal.id),
                        "".to_string(),
                    ],
                    penumbra_transaction::ActionView::ProposalWithdraw(prop_withdraw) => [
                        format!("Withdraw Governance Proposal #{}", prop_withdraw.proposal),
                        "".to_string(),
                    ],
                    penumbra_transaction::ActionView::ProposalDepositClaim(prop_deposit_claim) => [
                        format!(
                            "Claim Deposit for Governance Proposal #{}",
                            prop_deposit_claim.proposal
                        ),
                        "".to_string(),
                    ],
                    penumbra_transaction::ActionView::ValidatorVote(_) => {
                        ["Validator Vote".to_string(), "".to_string()]
                    }
                    penumbra_transaction::ActionView::DelegatorVote(_) => {
                        ["Delegator Vote".to_string(), "".to_string()]
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
                    penumbra_transaction::ActionView::Ics20Withdrawal(_) => {
                        ["Ics20 Withdrawal".to_string(), "".to_string()]
                    }
                    penumbra_transaction::ActionView::DaoDeposit(_) => {
                        ["Dao Deposit".to_string(), "".to_string()]
                    }
                    penumbra_transaction::ActionView::DaoSpend(_) => {
                        ["Dao Spend".to_string(), "".to_string()]
                    }
                    penumbra_transaction::ActionView::DaoOutput(_) => {
                        ["Dao Output".to_string(), "".to_string()]
                    }
                });
            }

            metadata_table.add_row(vec![
                "Transaction Fee",
                &txv.body_view.fee.value().format(&asset_cache),
            ]);

            let memo_view = txv.body_view.memo_view;

            if let Some(memo_view) = memo_view {
                match memo_view {
                    penumbra_transaction::MemoView::Visible {
                        plaintext,
                        ciphertext: _,
                    } => {
                        metadata_table.add_row(vec![
                            "Transaction Memo Sender",
                            &plaintext.sender.to_string(),
                        ]);
                        metadata_table.add_row(vec!["Transaction Memo Text", &plaintext.text]);
                    }
                    penumbra_transaction::MemoView::Opaque { ciphertext: _ } => (),
                }
            }

            metadata_table.add_row(vec![
                "Transaction Expiration Height",
                &format!("{}", txv.body_view.expiry_height),
            ]);

            // Print table of actions and their descriptions
            println!("{actions_table}");

            // Print transaction metadata
            println!("{metadata_table}");
        }

        Ok(())
    }
}
