use anyhow::{anyhow, Context, Result};
use ark_ff::Zero;
use decaf377::Fr;
use decaf377_rdsa as rdsa;
use penumbra_keys::{symmetric::PayloadKey, FullViewingKey};
use rand_core::OsRng;
use rand_core::{CryptoRng, RngCore};
use std::fmt::Debug;

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
    /// Arguments:
    ///     self: `TransactionPlan`
    ///     fvk: `FullViewingKey`
    ///     witness_data: `WitnessData`
    /// Returns: `Transaction`
    pub fn build_unauth_with_actions(
        self,
        mut actions: Vec<Action>,
        fvk: FullViewingKey,
        witness_data: WitnessData,
    ) -> Result<Transaction> {
        // Add the memo.
        let mut memo: Option<MemoCiphertext> = None;
        let mut memo_key: Option<PayloadKey> = None;
        if self.memo_plan.is_some() {
            let memo_plan = self
                .memo_plan
                .clone()
                .ok_or_else(|| anyhow!("missing memo_plan in TransactionPlan"))?;
            memo = memo_plan.memo().ok();
            memo_key = Some(memo_plan.key);
        }

        // Build the transaction's swaps.
        for swap_plan in self.swap_plans() {
            actions.push(Action::Swap(swap_plan.swap(&fvk)));
        }

        // Build the transaction's swap claims.
        for swap_claim_plan in self.swap_claim_plans().cloned() {
            let note_commitment = swap_claim_plan.swap_plaintext.swap_commitment();
            let auth_path = witness_data
                .state_commitment_proofs
                .get(&note_commitment)
                .context(format!("could not get proof for {note_commitment:?}"))?;

            actions.push(Action::SwapClaim(
                swap_claim_plan.swap_claim(&fvk, auth_path),
            ));
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

        // All of these actions have "transparent" value balance with no
        // blinding factor, so they don't contribute to the
        // synthetic_blinding_factor used for the binding signature.

        for delegation in self.delegations().cloned() {
            actions.push(Action::Delegate(delegation))
        }
        for undelegation in self.undelegations().cloned() {
            actions.push(Action::Undelegate(undelegation))
        }
        for plan in self.undelegate_claim_plans() {
            let undelegate_claim = plan.undelegate_claim();
            actions.push(Action::UndelegateClaim(undelegate_claim));
        }
        for proposal_submit in self.proposal_submits().cloned() {
            actions.push(Action::ProposalSubmit(proposal_submit))
        }
        for proposal_withdraw_plan in self.proposal_withdraws().cloned() {
            actions.push(Action::ProposalWithdraw(proposal_withdraw_plan));
        }
        for validator_vote in self.validator_votes().cloned() {
            actions.push(Action::ValidatorVote(validator_vote))
        }
        for delegator_vote_plan in self.delegator_vote_plans() {
            let note_commitment = delegator_vote_plan.staked_note.commit();
            let auth_path = witness_data
                .state_commitment_proofs
                .get(&note_commitment)
                .context(format!("could not get proof for {note_commitment:?}"))?;

            actions.push(Action::DelegatorVote(delegator_vote_plan.delegator_vote(
                &fvk,
                [0; 64].into(),
                auth_path.clone(),
            )));
        }
        for proposal_deposit_claim in self.proposal_deposit_claims().cloned() {
            actions.push(Action::ProposalDepositClaim(proposal_deposit_claim))
        }
        for vd in self.validator_definitions().cloned() {
            actions.push(Action::ValidatorDefinition(vd))
        }
        for ibc_action in self.ibc_actions().cloned() {
            actions.push(Action::IbcRelay(ibc_action))
        }
        for dao_spend in self.dao_spends().cloned() {
            actions.push(Action::DaoSpend(dao_spend))
        }
        for dao_output in self.dao_outputs().cloned() {
            actions.push(Action::DaoOutput(dao_output))
        }
        for dao_deposit in self.dao_deposits().cloned() {
            actions.push(Action::DaoDeposit(dao_deposit))
        }
        for position_open in self.position_openings().cloned() {
            actions.push(Action::PositionOpen(position_open))
        }
        for position_close in self.position_closings().cloned() {
            actions.push(Action::PositionClose(position_close))
        }
        for position_withdraw in self.position_withdrawals() {
            actions.push(Action::PositionWithdraw(
                position_withdraw.position_withdraw(),
            ))
        }
        // build the transaction's ICS20 withdrawals
        for ics20_withdrawal in self.ics20_withdrawals() {
            actions.push(Action::Ics20Withdrawal(ics20_withdrawal.clone()))
        }

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
    /// Arguments:
    ///     self: `TransactionPlan`
    ///     rng: `&mut R`
    ///     auth_data: `&AuthorizationData`
    ///     transaction: `Transaction`
    /// Returns: `Transaction`
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

        // Derive the blinding factors from `TransactionPlan``.
        let mut synthetic_blinding_factor = Fr::zero();

        for spend_plan in self.spend_plans() {
            synthetic_blinding_factor += spend_plan.value_blinding;
        }
        for output_plan in self.output_plans() {
            synthetic_blinding_factor += output_plan.value_blinding;
        }
        for swap_plan in self.swap_plans() {
            synthetic_blinding_factor += swap_plan.fee_blinding;
        }
        for plan in self.undelegate_claim_plans() {
            synthetic_blinding_factor += plan.balance_blinding;
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
    /// Arguments
    ///     - `full_viewing_key`, the [`FullViewingKey`] for the source funds;
    ///     - `witness_data`, the [`WitnessData`] used for proving;
    ///     -  `auth_data`, the [`AuthorizationData`] required to sign transaction.
    pub fn build(
        self,
        full_viewing_key: &FullViewingKey,
        witness_data: WitnessData,
        auth_data: &AuthorizationData,
    ) -> Result<Transaction> {
        // Add the memo.
        let mut memo: Option<MemoCiphertext> = None;
        let mut memo_key: Option<PayloadKey> = None;
        if self.memo_plan.is_some() {
            let memo_plan = self
                .memo_plan
                .clone()
                .ok_or_else(|| anyhow!("missing memo_plan in TransactionPlan"))?;
            memo = memo_plan.memo().ok();
            memo_key = Some(memo_plan.key);
        }

        let mut actions: Vec<Action> = Vec::new();

        // We build the actions sorted by type, with all spends first, then all
        // outputs, etc.  This order has to align with the ordering in
        // TransactionPlan::effect_hash, which computes the auth hash of the
        // transaction we'll build here without actually building it.
        
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
        let mut memo: Option<MemoCiphertext> = None;
        let mut memo_key: Option<PayloadKey> = None;
        if self.memo_plan.is_some() {
            let memo_plan = self
                .memo_plan
                .clone()
                .ok_or_else(|| anyhow!("missing memo_plan in TransactionPlan"))?;
            memo = memo_plan.memo().ok();
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
                spend
                    .build_unauth(&fvk, &witness_data_, memo_key.clone())
            }));
        }
        for output_plan in self.output_plans() {
            let fvk = full_viewing_key.clone();
            let witness_data_: WitnessData = witness_data.clone();
            let output = ActionPlan::Output(output_plan.to_owned());
            in_progress_output_actions.push(tokio::spawn(async move {
                output
                    .build_unauth(&fvk, &witness_data_, memo_key.clone())
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
