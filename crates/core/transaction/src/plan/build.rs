use anyhow::{Context, Result};
use penumbra_crypto::{rdsa, symmetric::PayloadKey, Fr, Zero};
use penumbra_keys::FullViewingKey;
use rand_core::{CryptoRng, RngCore};

use super::TransactionPlan;
use crate::{
    action::Action, memo::MemoCiphertext, AuthorizationData, AuthorizingData, Transaction,
    TransactionBody, WitnessData,
};

impl TransactionPlan {
    /// Build the transaction this plan describes.
    ///
    /// To turn a transaction plan into an unauthorized transaction, we need:
    ///
    /// - `fvk`, the [`FullViewingKey`] for the source funds;
    /// - `witness_data`, the [`WitnessData`] used for proving;
    ///
    pub fn build(
        self,
        fvk: &FullViewingKey,
        witness_data: WitnessData,
    ) -> Result<UnauthTransaction> {
        let mut actions = Vec::new();
        let mut fmd_clues = Vec::new();
        let mut synthetic_blinding_factor = Fr::zero();

        // Add the memo.
        let mut memo: Option<MemoCiphertext> = None;
        let mut memo_key: Option<PayloadKey> = None;
        if self.memo_plan.is_some() {
            let memo_plan = self.memo_plan.clone().unwrap();
            memo = memo_plan.memo().ok();
            memo_key = Some(memo_plan.key);
        }

        // We build the actions sorted by type, with all spends first, then all
        // outputs, etc.  This order has to align with the ordering in
        // TransactionPlan::effect_hash, which computes the auth hash of the
        // transaction we'll build here without actually building it.

        // Build the transaction's spends.
        for spend_plan in self.spend_plans() {
            let note_commitment = spend_plan.note.commit();
            let auth_path = witness_data
                .state_commitment_proofs
                .get(&note_commitment)
                .context(format!("could not get proof for {note_commitment:?}"))?;

            synthetic_blinding_factor += spend_plan.value_blinding;
            actions.push(Action::Spend(spend_plan.spend(
                fvk,
                [0; 64].into(),
                auth_path.clone(),
                witness_data.anchor,
            )));
        }

        // Build the transaction's outputs.
        let dummy_payload_key: PayloadKey = [0u8; 32].into();
        // If the memo_key is None, then there is no memo, and we populate the memo key
        // field with a dummy key.
        for output_plan in self.output_plans() {
            // Outputs subtract from the transaction's value balance.
            synthetic_blinding_factor += output_plan.value_blinding;
            actions.push(Action::Output(output_plan.output(
                fvk.outgoing(),
                memo_key.as_ref().unwrap_or(&dummy_payload_key),
            )));
        }

        // Build the transaction's swaps.
        for swap_plan in self.swap_plans() {
            synthetic_blinding_factor += swap_plan.fee_blinding;
            actions.push(Action::Swap(swap_plan.swap(fvk)));
        }

        // Build the transaction's swap claims.
        for swap_claim_plan in self.swap_claim_plans().cloned() {
            let note_commitment = swap_claim_plan.swap_plaintext.swap_commitment();
            let auth_path = witness_data
                .state_commitment_proofs
                .get(&note_commitment)
                .context(format!("could not get proof for {note_commitment:?}"))?;

            actions.push(Action::SwapClaim(
                swap_claim_plan.swap_claim(fvk, auth_path),
            ));
        }

        // Build the clue plans.
        for clue_plan in self.clue_plans() {
            fmd_clues.push(clue_plan.clue());
        }

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
            synthetic_blinding_factor += plan.balance_blinding;
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
                fvk,
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
            actions.push(Action::IbcAction(ibc_action))
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
        for position_withdraw in self.position_withdrawals().cloned() {
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
            expiry_height: self.expiry_height,
            chain_id: self.chain_id,
            fee: self.fee,
            fmd_clues,
            memo,
        };

        // TODO: add consistency checks?

        Ok(UnauthTransaction {
            inner: Transaction {
                transaction_body,
                anchor: witness_data.anchor,
                binding_sig: [0; 64].into(),
            },
            synthetic_blinding_factor,
        })
    }

    #[cfg(feature = "parallel")]
    /// Build the transaction this plan describes while proving concurrently.
    /// This can be used in environments that support tokio tasks.
    pub async fn build_concurrent<R: CryptoRng + RngCore>(
        self,
        rng: R,
        fvk: &FullViewingKey,
        witness_data: WitnessData,
    ) -> Result<UnauthTransaction> {
        let mut fmd_clues = Vec::new();
        let mut synthetic_blinding_factor = Fr::zero();

        // Add the memo.
        let mut memo: Option<MemoCiphertext> = None;
        let mut memo_key: Option<PayloadKey> = None;
        if self.memo_plan.is_some() {
            let memo_plan = self.memo_plan.clone().unwrap();
            memo = memo_plan.memo().ok();
            memo_key = Some(memo_plan.key);
        }

        // We build the actions sorted by type, with all spends first, then all
        // outputs, etc.  This order has to align with the ordering in
        // TransactionPlan::effect_hash, which computes the auth hash of the
        // transaction we'll build here without actually building it.

        // Start building the transaction's spends.
        let mut in_progress_spend_actions = Vec::new();
        for spend_plan in self.spend_plans().cloned() {
            let note_commitment = spend_plan.note.commit();
            let auth_path = witness_data
                .state_commitment_proofs
                .get(&note_commitment)
                .context(format!("could not get proof for {note_commitment:?}"))?
                .clone();

            synthetic_blinding_factor += spend_plan.value_blinding;
            let fvk_ = fvk.clone();
            in_progress_spend_actions.push(tokio::spawn(async move {
                //Add dummy auth sig for UnauthTransaction
                let auth_sig = [0; 64].into();
                spend_plan.spend(&fvk_, auth_sig, auth_path, witness_data.anchor)
            }));
        }

        // Start building the transaction's outputs.
        let mut in_progress_output_actions = Vec::new();
        let dummy_payload_key: PayloadKey = [0u8; 32].into();
        // If the memo_key is None, then there is no memo, and we populate the memo key
        // field with a dummy key.
        for output_plan in self.output_plans().cloned() {
            // Outputs subtract from the transaction's value balance.
            synthetic_blinding_factor += output_plan.value_blinding;
            let ovk = fvk.outgoing().clone();
            let memo_key = memo_key.as_ref().unwrap_or(&dummy_payload_key).clone();
            in_progress_output_actions.push(tokio::spawn(async move {
                output_plan.output(&ovk, &memo_key)
            }));
        }

        // Start building the transaction's swaps.
        let mut in_progress_swap_actions = Vec::new();
        for swap_plan in self.swap_plans().cloned() {
            synthetic_blinding_factor += swap_plan.fee_blinding;
            let fvk_ = fvk.clone();
            in_progress_swap_actions.push(tokio::spawn(async move { swap_plan.swap(&fvk_) }));
        }

        // Start building the transaction's swap claims.
        let mut in_progress_swap_claim_actions = Vec::new();
        for swap_claim_plan in self.swap_claim_plans().cloned() {
            let note_commitment = swap_claim_plan.swap_plaintext.swap_commitment();
            let auth_path = witness_data
                .state_commitment_proofs
                .get(&note_commitment)
                .context(format!("could not get proof for {note_commitment:?}"))?
                .clone();
            let fvk_ = fvk.clone();

            in_progress_swap_claim_actions.push(tokio::spawn(async move {
                swap_claim_plan.swap_claim(&fvk_, &auth_path)
            }));
        }

        // Start building the transaction's delegator votes.
        let mut in_progress_delegator_vote_actions = Vec::new();
        for delegator_vote_plan in self.delegator_vote_plans().cloned() {
            let note_commitment = delegator_vote_plan.staked_note.commit();
            let auth_path = witness_data
                .state_commitment_proofs
                .get(&note_commitment)
                .context(format!("could not get proof for {note_commitment:?}"))?
                .clone();
            let fvk_ = fvk.clone();

            in_progress_delegator_vote_actions.push(tokio::spawn(async move {
                //Add dummy auth sig for UnauthTransaction
                let auth_sig = [0; 64].into();
                delegator_vote_plan.delegator_vote(&fvk_, auth_sig, auth_path.clone())
            }));
        }

        // Build the clue plans.
        for clue_plan in self.clue_plans() {
            fmd_clues.push(clue_plan.clue());
        }

        // Actions with ZK proofs are slow to build and were done concurrently,
        // so we resolve the corresponding `JoinHandle`s in the order the tasks were started.
        let mut actions = Vec::new();
        // Collect the spend actions.
        for action in in_progress_spend_actions {
            actions.push(Action::Spend(action.await.expect("can form spend action")));
        }
        // Collect the output actions.
        for action in in_progress_output_actions {
            actions.push(Action::Output(
                action.await.expect("can form output action"),
            ));
        }
        // Collect the swap actions.
        for action in in_progress_swap_actions {
            actions.push(Action::Swap(action.await.expect("can form swap action")));
        }
        // Collect the swap claim actions.
        for action in in_progress_swap_claim_actions {
            actions.push(Action::SwapClaim(
                action.await.expect("can form swap claim action"),
            ));
        }

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
            synthetic_blinding_factor += plan.balance_blinding;
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
        for delegator_vote in in_progress_delegator_vote_actions {
            actions.push(Action::DelegatorVote(
                delegator_vote
                    .await
                    .expect("can form delegator vote action"),
            ));
        }
        for proposal_deposit_claim in self.proposal_deposit_claims().cloned() {
            actions.push(Action::ProposalDepositClaim(proposal_deposit_claim))
        }
        for vd in self.validator_definitions().cloned() {
            actions.push(Action::ValidatorDefinition(vd))
        }
        for ibc_action in self.ibc_actions().cloned() {
            actions.push(Action::IbcAction(ibc_action))
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
        for position_withdraw in self.position_withdrawals().cloned() {
            actions.push(Action::PositionWithdraw(
                position_withdraw.position_withdraw(),
            ))
        }

        let transaction_body = TransactionBody {
            actions,
            expiry_height: self.expiry_height,
            chain_id: self.chain_id,
            fee: self.fee,
            fmd_clues,
            memo,
        };

        // Finally, compute the binding signature and assemble the transaction.
        let binding_signing_key = rdsa::SigningKey::from(synthetic_blinding_factor);
        let auth_hash = transaction_body.auth_hash();
        let binding_sig = binding_signing_key.sign(rng, auth_hash.as_bytes());
        tracing::debug!(bvk = ?rdsa::VerificationKey::from(&binding_signing_key), ?auth_hash);

        Ok(UnauthTransaction {
            inner: Transaction {
                transaction_body,
                binding_sig,
                anchor: witness_data.anchor,
            },
            synthetic_blinding_factor,
        })
    }
}

/// A partially-constructed transaction awaiting authorization data.
pub struct UnauthTransaction {
    inner: Transaction,
    synthetic_blinding_factor: Fr,
}

impl UnauthTransaction {
    pub fn authorize<R: CryptoRng + RngCore>(
        mut self,
        rng: &mut R,
        auth_data: &AuthorizationData,
    ) -> Result<Transaction> {
        // Do some basic input sanity-checking.
        let spend_count = self.inner.spends().count();
        if auth_data.spend_auths.len() != spend_count {
            return Err(anyhow::anyhow!(
                "expected {} spend auths but got {}",
                spend_count,
                auth_data.spend_auths.len()
            ));
        }
        // Overwrite the placeholder auth sigs with the real ones from `auth_data`

        for (spend, auth_sig) in self
            .inner
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

        for (mut delegator_vote, auth_sig) in self
            .inner
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
        let binding_signing_key = rdsa::SigningKey::from(self.synthetic_blinding_factor);
        let auth_hash = self.inner.transaction_body.auth_hash();
        let binding_sig = binding_signing_key.sign(rng, auth_hash.as_bytes());
        tracing::debug!(bvk = ?rdsa::VerificationKey::from(&binding_signing_key), ?auth_hash);

        self.inner.binding_sig = binding_sig;

        Ok(self.inner)
    }
}
