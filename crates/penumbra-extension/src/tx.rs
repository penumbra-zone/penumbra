use anyhow::anyhow;
use penumbra_keys::FullViewingKey;
use penumbra_proto::core::transaction::v1::{TransactionPerspective, TransactionView};
use penumbra_transaction::Action;
use penumbra_transaction::Transaction;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::str::FromStr;

use crate::error::WasmResult;
use crate::storage::IndexedDBStorage;
use crate::storage::IndexedDbConstants;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TxInfoResponse {
    txp: TransactionPerspective,
    txv: TransactionView,
}

impl TxInfoResponse {
    pub fn new(txp: TransactionPerspective, txv: TransactionView) -> TxInfoResponse {
        Self { txp, txv }
    }
}

pub async fn transaction_info_inner(
    full_viewing_key: &str,
    tx: Transaction,
    idb_constants: IndexedDbConstants,
) -> WasmResult<TxInfoResponse> {
    let storage = IndexedDBStorage::new(idb_constants).await?;

    let fvk = FullViewingKey::from_str(full_viewing_key)?;

    // First, create a TxP with the payload keys visible to our FVK and no other data.
    let mut txp = penumbra_transaction::TransactionPerspective {
        payload_keys: tx.payload_keys(&fvk)?,
        ..Default::default()
    };

    // Next, extend the TxP with the openings of commitments known to our view server
    // but not included in the transaction body, for instance spent notes or swap claim outputs.
    for action in tx.actions() {
        match action {
            Action::Spend(spend) => {
                let nullifier = spend.body.nullifier;
                // An error here indicates we don't know the nullifier, so we omit it from the Perspective.
                if let Some(spendable_note_record) =
                    storage.get_note_by_nullifier(&nullifier).await?
                {
                    txp.spend_nullifiers
                        .insert(nullifier, spendable_note_record.note.clone());
                }
            }
            Action::SwapClaim(claim) => {
                let output_1_record = storage
                    .get_note(&claim.body.output_1_commitment)
                    .await?
                    .ok_or(anyhow!(
                        "Error generating TxP: SwapClaim output 1 commitment not found",
                    ))?;

                let output_2_record = storage
                    .get_note(&claim.body.output_2_commitment)
                    .await?
                    .ok_or(anyhow!(
                        "Error generating TxP: SwapClaim output 2 commitment not found"
                    ))?;

                txp.advice_notes
                    .insert(claim.body.output_1_commitment, output_1_record.note.clone());
                txp.advice_notes
                    .insert(claim.body.output_2_commitment, output_2_record.note.clone());
            }
            _ => {}
        }
    }

    // Now, generate a stub TxV from our minimal TxP, and inspect it to see what data we should
    // augment the minimal TxP with to provide additional context (e.g., filling in denoms for
    // visible asset IDs).
    let min_view = tx.view_from_perspective(&txp);
    let mut address_views = BTreeMap::new();
    let mut asset_ids = BTreeSet::new();
    for action_view in min_view.action_views() {
        use penumbra_dex::{swap::SwapView, swap_claim::SwapClaimView};
        use penumbra_transaction::view::action_view::{
            ActionView, DelegatorVoteView, OutputView, SpendView,
        };
        match action_view {
            ActionView::Spend(SpendView::Visible { note, .. }) => {
                let address = note.address();
                address_views.insert(address, fvk.view_address(address));
                asset_ids.insert(note.asset_id());
            }
            ActionView::Output(OutputView::Visible { note, .. }) => {
                let address = note.address();
                address_views.insert(address, fvk.view_address(address));
                asset_ids.insert(note.asset_id());

                // Also add an AddressView for the return address in the memo.
                let memo = tx.decrypt_memo(&fvk)?;
                address_views.insert(memo.return_address(), fvk.view_address(address));
            }
            ActionView::Swap(SwapView::Visible { swap_plaintext, .. }) => {
                let address = swap_plaintext.claim_address;
                address_views.insert(address, fvk.view_address(address));
                asset_ids.insert(swap_plaintext.trading_pair.asset_1());
                asset_ids.insert(swap_plaintext.trading_pair.asset_2());
            }
            ActionView::SwapClaim(SwapClaimView::Visible {
                output_1, output_2, ..
            }) => {
                // Both will be sent to the same address so this only needs to be added once
                let address = output_1.address();
                address_views.insert(address, fvk.view_address(address));
                asset_ids.insert(output_1.asset_id());
                asset_ids.insert(output_2.asset_id());
            }
            ActionView::DelegatorVote(DelegatorVoteView::Visible { note, .. }) => {
                let address = note.address();
                address_views.insert(address, fvk.view_address(address));
                asset_ids.insert(note.asset_id());
            }
            _ => {}
        }
    }

    // Now, extend the TxP with information helpful to understand the data it can view:

    let mut denoms = Vec::new();

    for id in asset_ids {
        if let Some(denom) = storage.get_asset(&id).await? {
            denoms.push(denom.clone());
        }
    }

    txp.denoms.extend(denoms.into_iter());

    txp.address_views = address_views.into_values().collect();

    // Finally, compute the full TxV from the full TxP:
    let txv = tx.view_from_perspective(&txp);

    let txp_proto = TransactionPerspective::try_from(txp)?;
    let txv_proto = TransactionView::try_from(txv)?;

    let response = TxInfoResponse {
        txp: txp_proto,
        txv: txv_proto,
    };
    Ok(response)
}
