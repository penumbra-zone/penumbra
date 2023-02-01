use anyhow::{Context, Result};
use penumbra_crypto::{
    memo::MemoCiphertext, rdsa, symmetric::PayloadKey, Fr, FullViewingKey, Zero,
};
use rand_core::{CryptoRng, RngCore};

use super::TransactionPlan;
use crate::{action::Action, AuthorizationData, Transaction, TransactionBody, WitnessData};

impl TransactionPlan {
    /// Build the transaction this plan describes.
    ///
    /// To turn a transaction plan into a transaction, we need:
    ///
    /// - `fvk`, the [`FullViewingKey`] for the source funds;
    /// - `auth_data`, the [`AuthorizationData`] authorizing the transaction;
    /// - `witness_data`, the [`WitnessData`] used for proving;
    ///
    pub fn build<R: CryptoRng + RngCore>(
        self,
        rng: &mut R,
        fvk: &FullViewingKey,
        auth_data: AuthorizationData,
        witness_data: WitnessData,
    ) -> Result<Transaction> {
        // Do some basic input sanity-checking.
        let spend_count = self.spend_plans().count();
        if auth_data.spend_auths.len() != spend_count {
            return Err(anyhow::anyhow!(
                "expected {} spend auths but got {}",
                spend_count,
                auth_data.spend_auths.len()
            ));
        }

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
        for (spend_plan, auth_sig) in self.spend_plans().zip(auth_data.spend_auths.into_iter()) {
            let note_commitment = spend_plan.note.commit();
            let auth_path = witness_data
                .state_commitment_proofs
                .get(&note_commitment)
                .context(format!("could not get proof for {:?}", note_commitment))?;

            synthetic_blinding_factor += spend_plan.value_blinding;
            actions.push(Action::Spend(spend_plan.spend(
                fvk,
                auth_sig,
                auth_path.clone(),
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
                .context(format!("could not get proof for {:?}", note_commitment))?;

            actions.push(Action::SwapClaim(
                swap_claim_plan.swap_claim(fvk, auth_path),
            ));
        }

        // Build the clue plans.
        for clue_plan in self.clue_plans() {
            fmd_clues.push(clue_plan.clue());
        }

        // We don't have anything more to build, but iterate through the rest of
        // the action plans by type so that the transaction will have them in a
        // defined order.

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
        // TODO: delegator vote
        for proposal_deposit_claim in self.proposal_deposit_claims().cloned() {
            actions.push(Action::ProposalDepositClaim(proposal_deposit_claim))
        }
        for vd in self.validator_definitions().cloned() {
            actions.push(Action::ValidatorDefinition(vd))
        }
        for ibc_action in self.ibc_actions().cloned() {
            actions.push(Action::IBCAction(ibc_action))
        }

        // Finally, compute the binding signature and assemble the transaction.
        let binding_signing_key = rdsa::SigningKey::from(synthetic_blinding_factor);
        let binding_sig = binding_signing_key.sign(rng, auth_data.effect_hash.as_ref());
        tracing::debug!(bvk = ?rdsa::VerificationKey::from(&binding_signing_key), effect_hash = ?auth_data.effect_hash);

        // TODO: add consistency checks?

        Ok(Transaction {
            transaction_body: TransactionBody {
                actions,
                expiry_height: self.expiry_height,
                chain_id: self.chain_id,
                fee: self.fee,
                fmd_clues,
                memo,
            },
            anchor: witness_data.anchor,
            binding_sig,
        })
    }
}
