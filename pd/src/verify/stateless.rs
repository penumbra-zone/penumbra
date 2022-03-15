use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Error};
use penumbra_crypto::{note, Nullifier};
use penumbra_stake::{Delegate, Undelegate, ValidatorDefinition};
use penumbra_transaction::{Action, Transaction};

use super::{NoteData, PendingTransaction};

/// An extension trait that performs stateless transaction verification
/// (verifying signatures and proofs, but not checking consistency with the
/// chain state).
///
/// This is defined as an extension trait since the [`Transaction`] is defined
/// in another crate.
pub trait StatelessTransactionExt {
    fn verify_stateless(&self) -> Result<PendingTransaction, Error>;
}

impl StatelessTransactionExt for Transaction {
    // TODO: use tokio's blocking code when we do work here -- internally to verify_stateless?
    fn verify_stateless(&self) -> Result<PendingTransaction, Error> {
        let id = self.id();

        let sighash = self.transaction_body().sighash();

        // 1. Check binding signature.
        self.binding_verification_key()
            .verify(&sighash, self.binding_sig())
            .context("binding signature failed to verify")?;

        // 2. Check all spend auth signatures using provided spend auth keys
        // and check all proofs verify. If any action does not verify, the entire
        // transaction has failed.
        let mut spent_nullifiers = BTreeSet::<Nullifier>::new();
        let mut new_notes = BTreeMap::<note::Commitment, NoteData>::new();
        let mut delegations = Vec::<Delegate>::new();
        let mut undelegation = None::<Undelegate>;
        let mut validator_definitions = Vec::<ValidatorDefinition>::new();

        for action in self.transaction_body().actions {
            match action {
                Action::Output(output) => {
                    if output
                        .body
                        .proof
                        .verify(
                            output.body.value_commitment,
                            output.body.note_commitment,
                            output.body.ephemeral_key,
                        )
                        .is_err()
                    {
                        // TODO should the verification error be bubbled up here?
                        return Err(anyhow::anyhow!("An output proof did not verify"));
                    }

                    new_notes.insert(
                        output.body.note_commitment,
                        NoteData {
                            ephemeral_key: output.body.ephemeral_key,
                            encrypted_note: output.body.encrypted_note,
                            transaction_id: id,
                        },
                    );
                }
                Action::Spend(spend) => {
                    spend
                        .body
                        .rk
                        .verify(&sighash, &spend.auth_sig)
                        .context("spend auth signature failed to verify")?;

                    if spend
                        .body
                        .proof
                        .verify(
                            self.transaction_body().merkle_root,
                            spend.body.value_commitment,
                            spend.body.nullifier.clone(),
                            spend.body.rk,
                        )
                        .is_err()
                    {
                        // TODO should the verification error be bubbled up here?
                        return Err(anyhow::anyhow!("A spend proof did not verify"));
                    }

                    // Check nullifier has not been revealed already in this transaction.
                    if spent_nullifiers.contains(&spend.body.nullifier.clone()) {
                        return Err(anyhow::anyhow!("Double spend"));
                    }

                    spent_nullifiers.insert(spend.body.nullifier.clone());
                }
                Action::Delegate(delegate) => {
                    // There are currently no stateless verification checks than the ones implied by
                    // the binding signature.
                    delegations.push(delegate);
                }
                Action::Undelegate(undelegate) => {
                    if undelegation.is_none() {
                        undelegation = Some(undelegate);
                    } else {
                        return Err(anyhow::anyhow!("Multiple undelegations in one transaction"));
                    }
                }
                Action::ValidatorDefinition(validator) => {
                    // Perform stateless checks that the validator definition is valid.

                    // Validate that the transaction signature is valid and signed by the
                    // validator's identity key.
                    validator
                        .validator
                        .identity_key
                        .0
                        .verify(&sighash, &validator.auth_sig)
                        .context("validator definition signature failed to verify")?;

                    // Validate that the definition's funding streams do not exceed 100% (10000bps)
                    let total_funding_bps = validator
                        .validator
                        .funding_streams
                        // TODO: possible to remove this clone?
                        .clone()
                        .into_iter()
                        .map(|stream| stream.rate_bps as u64)
                        .sum::<u64>();

                    if total_funding_bps > 10000 {
                        return Err(anyhow::anyhow!(
                            "Total validator definition funding streams exceeds 100%"
                        ));
                    }

                    // TODO: Any other stateless checks to apply to validator definitions?

                    validator_definitions.push(validator);
                }
                #[allow(unreachable_patterns)]
                _ => {
                    return Err(anyhow::anyhow!("unsupported action"));
                }
            }
        }

        // We prohibit actions other than `Spend`, `Delegate`, `Output` and `Undelegate` in
        // transactions that contain `Undelegate`, to avoid having to quarantine them.
        if undelegation.is_some() {
            use Action::*;
            for action in self.transaction_body().actions {
                if !matches!(action, Undelegate(_) | Delegate(_) | Spend(_) | Output(_)) {
                    return Err(anyhow::anyhow!("transaction contains an undelegation, but also contains an action other than Spend, Delegate, Output or Undelegate"));
                }
            }
        }

        Ok(PendingTransaction {
            id,
            root: self.transaction_body().merkle_root,
            new_notes,
            spent_nullifiers,
            delegations,
            undelegation,
            validator_definitions,
        })
    }
}
