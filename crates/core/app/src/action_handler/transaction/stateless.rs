use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result};
use penumbra_transaction::Transaction;
use penumbra_txhash::AuthorizingData;

#[tracing::instrument(skip(tx))]
pub(super) fn valid_binding_signature(tx: &Transaction) -> Result<()> {
    let auth_hash = tx.auth_hash();

    tracing::debug!(bvk = ?tx.binding_verification_key(), ?auth_hash);

    // Check binding signature.
    tx.binding_verification_key()
        .verify(auth_hash.as_bytes(), tx.binding_sig())
        .context("binding signature failed to verify")
}

pub(super) fn no_duplicate_votes(tx: &Transaction) -> Result<()> {
    // Disallow multiple `DelegatorVotes`s with the same proposal and the same `Nullifier`.
    let mut nullifiers_by_proposal_id = BTreeMap::new();
    for vote in tx.delegator_votes() {
        // Get existing entries
        let nullifiers_for_proposal = nullifiers_by_proposal_id
            .entry(vote.body.proposal)
            .or_insert(BTreeSet::default());

        // If there was a duplicate nullifier for this proposal, this call will return `false`:
        let not_duplicate = nullifiers_for_proposal.insert(vote.body.nullifier);
        if !not_duplicate {
            return Err(anyhow::anyhow!(
                "Duplicate nullifier in transaction: {}",
                &vote.body.nullifier
            ));
        }
    }

    Ok(())
}

pub(super) fn no_duplicate_spends(tx: &Transaction) -> Result<()> {
    // Disallow multiple `Spend`s or `SwapClaim`s with the same `Nullifier`.
    // This can't be implemented in the (`Spend`)[`crate::action_handler::actions::spend::Spend`] `ActionHandler`
    // because it requires access to the entire transaction, and we don't want to perform the check across the entire
    // transaction for *each* `Spend` within the transaction, only once.
    let mut spent_nullifiers = BTreeSet::new();
    for nf in tx.spent_nullifiers() {
        if let Some(duplicate) = spent_nullifiers.replace(nf) {
            anyhow::bail!("Duplicate nullifier in transaction: {}", duplicate);
        }
    }

    Ok(())
}

pub fn num_clues_equal_to_num_outputs(tx: &Transaction) -> anyhow::Result<()> {
    if tx
        .transaction_body()
        .detection_data
        .unwrap_or_default()
        .fmd_clues
        .len()
        != tx.outputs().count()
    {
        Err(anyhow::anyhow!(
            "consensus rule violated: must have equal number of outputs and FMD clues"
        ))
    } else {
        Ok(())
    }
}

#[allow(clippy::if_same_then_else)]
pub fn check_memo_exists_if_outputs_absent_if_not(tx: &Transaction) -> anyhow::Result<()> {
    let num_outputs = tx.outputs().count();
    if num_outputs > 0 && tx.transaction_body().memo.is_none() {
        Err(anyhow::anyhow!(
            "consensus rule violated: must have memo if outputs present"
        ))
    } else if num_outputs > 0 && tx.transaction_body().memo.is_some() {
        Ok(())
    } else if num_outputs == 0 && tx.transaction_body().memo.is_none() {
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "consensus rule violated: cannot have memo if no outputs present"
        ))
    }
}
