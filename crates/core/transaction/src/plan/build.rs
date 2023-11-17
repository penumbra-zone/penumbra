use anyhow::{anyhow, Context, Result};
use ark_ff::Zero;
use decaf377::Fr;
use decaf377_rdsa as rdsa;
use penumbra_keys::{symmetric::PayloadKey, FullViewingKey};
use rand_core::OsRng;
use rand_core::{CryptoRng, RngCore};
use std::fmt::Debug;
use wasm_bindgen_test::console_log;

use super::TransactionPlan;
use crate::plan::ActionPlan;
use crate::{
    action::Action,
    memo::MemoCiphertext,
    transaction::{DetectionData, TransactionParameters},
    AuthorizationData, AuthorizingData, Transaction, TransactionBody, WitnessData,
};

impl TransactionPlan {
    /// Builds a [`TransactionPlan`] by slotting in the
    /// provided prebuilt actions instead of using the
    /// [`ActionPlan`]s in the TransactionPlan.
    pub fn build_unauth_with_actions(
        self,
        actions: Vec<Action>,
        fvk: FullViewingKey,
        witness_data: WitnessData,
    ) -> Result<Transaction> {
        // Add the memo.
        let mut memo: Option<MemoCiphertext> = None;
        let mut _memo_key: Option<PayloadKey> = None;
        if self.memo_plan.is_some() {
            let memo_plan = self
                .memo_plan
                .clone()
                .ok_or_else(|| anyhow!("missing memo_plan in TransactionPlan"))?;
            memo = memo_plan.memo().ok();
            _memo_key = Some(memo_plan.key);
        }

        // Add detection data when there are outputs.
        let detection_data: Option<DetectionData> = if self.num_outputs() == 0 {
            None
        } else {
            let mut fmd_clues = Vec::new();
            for clue_plan in self.clue_plans() {
                fmd_clues.push(clue_plan.clue());
            }
            Some(DetectionData { fmd_clues })
        };

        let transaction_body = TransactionBody {
            actions,
            transaction_parameters: TransactionParameters {
                expiry_height: self.expiry_height,
                chain_id: self.chain_id,
            },
            fee: self.fee,
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
    pub fn authorize_with_auth<R: CryptoRng + RngCore + Debug>(
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
        witness_data: WitnessData,
        auth_data: &AuthorizationData,
    ) -> Result<Transaction> {
        // Add the memo.
        let mut _memo: Option<MemoCiphertext> = None;
        let mut memo_key: Option<PayloadKey> = None;
        if self.memo_plan.is_some() {
            let memo_plan = self
                .memo_plan
                .clone()
                .ok_or_else(|| anyhow!("missing memo_plan in TransactionPlan"))?;
            _memo = memo_plan.memo().ok();
            memo_key = Some(memo_plan.key);
        }

        let mut actions: Vec<Action> = Vec::new();

        // 1. Build each action.
        for spend_plan in self.spend_plans() {
            let spend = ActionPlan::Spend(spend_plan.to_owned());
            let action = spend
                .build_unauth(full_viewing_key, &witness_data, memo_key.clone())
                .expect("Build spend action failed!");
            actions.push(action);
        }
        for output_plan in self.output_plans() {
            let output = ActionPlan::Output(output_plan.to_owned());
            let action = output
                .build_unauth(full_viewing_key, &witness_data, memo_key.clone())
                .expect("Build output action failed!");
            actions.push(action);
        }

        // TODO: Handle other actions (swaps, swap claims, etc.).

        // 2. Pass in the prebuilt actions to the build method.
        let transaction = self.clone().build_unauth_with_actions(
            actions,
            full_viewing_key.to_owned(),
            witness_data,
        )?;

        // 3. Slot in the authorization data with TransactionPlan::authorize_with_aut,
        // and return the completed transaction.
        let tx = self.authorize_with_auth(&mut OsRng, auth_data, transaction)?;

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
        // Add the memo.
        let mut _memo: Option<MemoCiphertext> = None;
        let mut memo_key: Option<PayloadKey> = None;
        if self.memo_plan.is_some() {
            let memo_plan = self
                .memo_plan
                .clone()
                .ok_or_else(|| anyhow!("missing memo_plan in TransactionPlan"))?;
            _memo = memo_plan.memo().ok();
            memo_key = Some(memo_plan.key);
        }

        let mut in_progress_spend_actions = Vec::new();
        let mut in_progress_output_actions = Vec::new();

        // 1. Call ActionPlan::build_unauth on each action.
        for spend_plan in self.spend_plans() {
            let fvk = full_viewing_key.clone();
            let witness_data_: WitnessData = witness_data.clone();
            let spend = ActionPlan::Spend(spend_plan.to_owned());
            in_progress_spend_actions.push(tokio::spawn(async move {
                spend.build_unauth(&fvk, &witness_data_, memo_key.clone())
            }));
        }
        for output_plan in self.output_plans() {
            let fvk = full_viewing_key.clone();
            let witness_data_: WitnessData = witness_data.clone();
            let output = ActionPlan::Output(output_plan.to_owned());
            in_progress_output_actions.push(tokio::spawn(async move {
                output.build_unauth(&fvk, &witness_data_, memo_key.clone())
            }));
        }

        // TODO: Handle other actions (swaps, swap claims, etc.).

        // Actions with ZK proofs are slow to build and were done concurrently,
        // so we resolve the corresponding `JoinHandle`s in the order the tasks were started.
        let mut actions = Vec::new();

        // Collect the spend actions.
        for action in in_progress_spend_actions {
            actions.push(action.await?.expect("can form spend action"));
        }
        // Collect the output actions.
        for action in in_progress_output_actions {
            actions.push(action.await?.expect("can form output action"));
        }

        // TODO: Collect other actions (swaps, swap claims, etc.).

        // 2. Pass prebuilt actions to the build method.
        let transaction = self.clone().build_unauth_with_actions(
            actions,
            full_viewing_key.to_owned(),
            witness_data.to_owned(),
        )?;

        // 3. Slot in the authorization data with TransactionPlan::authorize_with_aut,
        // and return the completed transaction.
        let tx = self.authorize_with_auth(&mut OsRng, auth_data, transaction)?;

        Ok(tx)
    }
}
