use std::collections::BTreeSet;

use anyhow::{Context, Result};
use penumbra_transaction::Transaction;

pub(super) fn valid_binding_signature(tx: &Transaction) -> Result<()> {
    let auth_hash = tx.transaction_body().auth_hash();

    // Check binding signature.
    tx.binding_verification_key()
        .verify(auth_hash.as_ref(), tx.binding_sig())
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

pub(super) fn at_most_one_undelegate(tx: &Transaction) -> Result<()> {
    // 3. Check that the transaction undelegates from at most one validator.
    let undelegation_identities = tx
        .undelegations()
        .map(|u| u.validator_identity.clone())
        .collect::<BTreeSet<_>>();

    if undelegation_identities.len() > 1 {
        return Err(anyhow::anyhow!(
            "transaction contains undelegations from multiple validators: {:?}",
            undelegation_identities
        ));
    }

    // We prohibit actions other than `Spend`, `Delegate`, `Output` and `Undelegate` in
    // transactions that contain `Undelegate`, to avoid having to quarantine them.
    if undelegation_identities.len() == 1 {
        use penumbra_transaction::Action::*;
        for action in tx.transaction_body().actions {
            if !matches!(action, Undelegate(_) | Delegate(_) | Spend(_) | Output(_)) {
                return Err(anyhow::anyhow!("transaction contains an undelegation, but also contains an action other than Spend, Delegate, Output or Undelegate"));
            }
        }
    }

    Ok(())
}
