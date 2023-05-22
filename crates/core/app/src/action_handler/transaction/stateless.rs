use std::collections::BTreeSet;

use anyhow::{Context, Result};
use penumbra_transaction::{AuthorizingData, Transaction};

#[tracing::instrument(skip(tx))]
pub(super) fn valid_binding_signature(tx: &Transaction) -> Result<()> {
    let auth_hash = tx.auth_hash();

    tracing::debug!(bvk = ?tx.binding_verification_key(), ?auth_hash);

    // Check binding signature.
    tx.binding_verification_key()
        .verify(auth_hash.as_bytes(), tx.binding_sig())
        .context("binding signature failed to verify")
}

pub(super) fn no_duplicate_nullifiers(tx: &Transaction) -> Result<()> {
    // Disallow multiple `Spend`s with the same `Nullifier`.
    // This can't be implemented in the (`Spend`)[`crate::action_handler::actions::spend::Spend`] `ActionHandler`
    // because it requires access to the entire transaction, and we don't want to perform the check across the entire
    // transaction for *each* `Spend` within the transaction, only once.
    let mut spent_nullifiers = BTreeSet::new();
    for nf in tx.spent_nullifiers() {
        if let Some(duplicate) = spent_nullifiers.replace(nf) {
            return Err(anyhow::anyhow!(
                "Duplicate nullifier in transaction: {}",
                duplicate
            ));
        }
    }

    Ok(())
}

pub fn num_clues_equal_to_num_outputs(tx: &Transaction) -> anyhow::Result<()> {
    if tx.transaction_body().fmd_clues.len() != tx.outputs().count() {
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
