use anyhow::Result;
use ark_ff::Zero;
use decaf377::Fr;
use decaf377_rdsa as rdsa;
use penumbra_keys::FullViewingKey;
use penumbra_txhash::AuthorizingData;
use rand_core::OsRng;
use rand_core::{CryptoRng, RngCore};
use std::fmt::Debug;

use super::TransactionPlan;
use crate::ActionPlan;
use crate::{action::Action, AuthorizationData, Transaction, TransactionBody, WitnessData};

impl TransactionPlan {
    /// Builds a [`TransactionPlan`] by slotting in the
    /// provided prebuilt actions instead of using the
    /// [`ActionPlan`]s in the TransactionPlan.
    pub fn build_unauth_with_actions(
        self,
        actions: Vec<Action>,
        witness_data: &WitnessData,
    ) -> Result<Transaction> {
        // Add the memo if it is planned.
        let memo = self
            .memo
            .as_ref()
            .map(|memo_data| memo_data.memo())
            .transpose()?;

        let detection_data = self.detection_data.as_ref().map(|x| x.detection_data());

        let transaction_body = TransactionBody {
            actions,
            transaction_parameters: self.transaction_parameters,
            detection_data,
            memo,
        };

        Ok(Transaction {
            transaction_body,
            anchor: witness_data.anchor,
            binding_sig: [0; 64].into(),
        })
    }

    /// Slot in the [`AuthorizationData`] and derive the synthetic
    /// blinding factors needed to compute the binding signature
    /// and assemble the transaction.
    pub fn apply_auth_data<R: CryptoRng + RngCore + Debug>(
        &self,
        rng: &mut R,
        auth_data: &AuthorizationData,
        mut transaction: Transaction,
    ) -> Result<Transaction> {
        // Do some basic input sanity-checking.
        let spend_count = transaction.spends().count();
        if auth_data.spend_auths.len() != spend_count {
            anyhow::bail!(
                "expected {} spend auths but got {}",
                spend_count,
                auth_data.spend_auths.len()
            );
        }

        // Derive the synthetic blinding factors from `TransactionPlan`.
        let mut synthetic_blinding_factor = Fr::zero();

        // Accumulate the blinding factors.
        for action_plan in &self.actions {
            synthetic_blinding_factor += action_plan.value_blinding();
        }

        // Overwrite the placeholder authorization signatures with the real `AuthorizationData`.
        for (spend, auth_sig) in transaction
            .transaction_body
            .actions
            .iter_mut()
            .filter_map(|action| {
                if let Action::Spend(s) = action {
                    Some(s)
                } else {
                    None
                }
            })
            .zip(auth_data.spend_auths.clone().into_iter())
        {
            spend.auth_sig = auth_sig;
        }

        for (delegator_vote, auth_sig) in transaction
            .transaction_body
            .actions
            .iter_mut()
            .filter_map(|action| {
                if let Action::DelegatorVote(s) = action {
                    Some(s)
                } else {
                    None
                }
            })
            .zip(auth_data.delegator_vote_auths.clone().into_iter())
        {
            delegator_vote.auth_sig = auth_sig;
        }

        // Compute the binding signature and assemble the transaction.
        let binding_signing_key = rdsa::SigningKey::from(synthetic_blinding_factor);
        let auth_hash = transaction.transaction_body.auth_hash();

        let binding_sig = binding_signing_key.sign(rng, auth_hash.as_bytes());
        tracing::debug!(bvk = ?rdsa::VerificationKey::from(&binding_signing_key), ?auth_hash);

        transaction.binding_sig = binding_sig;

        Ok(transaction)
    }

    /// Build the serial transaction this plan describes.
    pub fn build(
        self,
        full_viewing_key: &FullViewingKey,
        witness_data: &WitnessData,
        auth_data: &AuthorizationData,
    ) -> Result<Transaction> {
        // 1. Build each action.
        let actions = self
            .actions
            .iter()
            .map(|action_plan| {
                ActionPlan::build_unauth(
                    action_plan.clone(),
                    full_viewing_key,
                    witness_data,
                    self.memo_key(),
                )
            })
            .collect::<Result<Vec<_>>>()?;

        // 2. Pass in the prebuilt actions to the build method.
        let tx = self
            .clone()
            .build_unauth_with_actions(actions, witness_data)?;

        // 3. Slot in the authorization data with .apply_auth_data,
        let tx = self.apply_auth_data(&mut OsRng, auth_data, tx)?;

        // 4. Return the completed transaction.
        Ok(tx)
    }

    #[cfg(feature = "parallel")]
    /// Build the transaction this plan describes while proving concurrently.
    /// This can be used in environments that support tokio tasks.
    pub async fn build_concurrent(
        self,
        full_viewing_key: &FullViewingKey,
        witness_data: &WitnessData,
        auth_data: &AuthorizationData,
    ) -> Result<Transaction> {
        // Clone the witness data into an Arc so it can be shared between tasks.
        let witness_data = std::sync::Arc::new(witness_data.clone());

        // 1. Build each action (concurrently).
        let action_handles = self
            .actions
            .iter()
            .cloned()
            .map(|action_plan| {
                let fvk2 = full_viewing_key.clone();
                let witness_data2 = witness_data.clone(); // Arc
                let memo_key2 = self.memo_key();
                tokio::task::spawn_blocking(move || {
                    ActionPlan::build_unauth(action_plan, &fvk2, &*witness_data2, memo_key2)
                })
            })
            .collect::<Vec<_>>();

        // 1.5. Collect all of the actions.
        let mut actions = Vec::new();
        for handle in action_handles {
            actions.push(handle.await??);
        }

        // 2. Pass in the prebuilt actions to the build method.
        let tx = self
            .clone()
            .build_unauth_with_actions(actions, &*witness_data)?;

        // 3. Slot in the authorization data with .apply_auth_data,
        let tx = self.apply_auth_data(&mut OsRng, auth_data, tx)?;

        // 4. Return the completed transaction.
        Ok(tx)
    }
}
