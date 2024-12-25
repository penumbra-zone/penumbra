use anyhow::{Context, Result};
use penumbra_sdk_transaction::Transaction;
use penumbra_sdk_txhash::AuthorizingData;

#[tracing::instrument(skip(tx))]
pub(super) fn valid_binding_signature(tx: &Transaction) -> Result<()> {
    let auth_hash = tx.auth_hash();

    tracing::debug!(bvk = ?tx.binding_verification_key(), ?auth_hash);

    // Check binding signature.
    tx.binding_verification_key()
        .verify(auth_hash.as_bytes(), tx.binding_sig())
        .context("binding signature failed to verify")
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

pub fn check_non_empty_transaction(tx: &Transaction) -> anyhow::Result<()> {
    let num_actions = tx.actions().count();
    if num_actions > 0 {
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "consensus rule violated: transaction must have more than 0 actions"
        ))
    }
}
