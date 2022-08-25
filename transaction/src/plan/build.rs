use anyhow::Result;
use penumbra_crypto::{rdsa, Fr, FullViewingKey, Zero};
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
        if witness_data.note_commitment_proofs.len() != spend_count {
            return Err(anyhow::anyhow!(
                "expected {} auth paths but got {}",
                spend_count,
                witness_data.note_commitment_proofs.len()
            ));
        }

        let mut actions = Vec::new();
        let mut fmd_clues = Vec::new();
        let mut synthetic_blinding_factor = Fr::zero();

        // We build the actions sorted by type, with all spends first, then all
        // outputs, etc.  This order has to align with the ordering in
        // TransactionPlan::auth_hash, which computes the auth hash of the
        // transaction we'll build here without actually building it.

        // Build the transaction's spends.
        for ((spend_plan, auth_sig), auth_path) in self
            .spend_plans()
            .zip(auth_data.spend_auths.into_iter())
            .zip(witness_data.note_commitment_proofs.into_iter())
        {
            // Spends add to the transaction's value balance.
            synthetic_blinding_factor += spend_plan.value_blinding;
            actions.push(Action::Spend(spend_plan.spend(fvk, auth_sig, auth_path)));
        }

        // Build the transaction's outputs.
        for output_plan in self.output_plans() {
            // Outputs subtract from the transaction's value balance.
            synthetic_blinding_factor -= output_plan.value_blinding;
            actions.push(Action::Output(output_plan.output(fvk.outgoing())));
        }

        // Build the transaction's swaps.
        // TODO: figure this out
        //         the swap plans shouldn't require pulling in spend auth signatures, because the Swap just draws from the transaction's value balance. but we'll need to figure out how we handle the authentication paths (note commitment proofs) -- in the existing WitnessData struct, there's just one for each spend, in order, because (now-broken assumption) the only place they were needed was for spends.

        // wondering if we should change the WitnessData domain type to store a BTreeMap<note::Commitment, tct::Proof>, and when deserializing from the proto, instead of just deserializing a list of proofs, call .commitment() on each one and use it as the key for the BTreeMap.

        // then instead of having to do zips or whatever, or figure out how to maintain a brittle order dependency, we can just query for exactly the proof we want.
        // for ((swap_plan, auth_sig), auth_path) in self
        //     .swap_plans()
        //     .zip(auth_data.spend_auths.into_iter())
        //     .zip(witness_data.note_commitment_proofs.into_iter())
        // {
        //     actions.push(Action::Swap(swap_plan.swap(fvk, witness_data.anchor)));
        // }

        // Build the transaction's swap claims.
        // for swap_claim_plan in self.swap_claim_plans().cloned() {
        //     actions.push(Action::SwapClaim(swap_claim_plan.swap_claim(
        //         fvk,
        //         note_commitment_proof,
        //         nk,
        //         note_blinding,
        //     )));
        // }

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
        for vd in self.validator_definitions().cloned() {
            actions.push(Action::ValidatorDefinition(vd))
        }
        for ibc_action in self.ibc_actions().cloned() {
            actions.push(Action::IBCAction(ibc_action))
        }

        // Finally, compute the binding signature and assemble the transaction.
        let binding_signing_key = rdsa::SigningKey::from(synthetic_blinding_factor);
        let binding_sig = binding_signing_key.sign(rng, auth_data.auth_hash.as_ref());

        // TODO: add consistency checks?

        Ok(Transaction {
            transaction_body: TransactionBody {
                actions,
                expiry_height: self.expiry_height,
                chain_id: self.chain_id,
                fee: self.fee,
                fmd_clues,
            },
            anchor: witness_data.anchor,
            binding_sig,
        })
    }
}
