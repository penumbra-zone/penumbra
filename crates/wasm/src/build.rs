use anyhow::{anyhow, Context, Result};
use ark_ff::Zero;
use decaf377::Fr;
use decaf377_rdsa as rdsa;
use penumbra_keys::{symmetric::PayloadKey, FullViewingKey};
use penumbra_proto::core::component::shielded_pool::v1alpha1::SpendPlan;
use penumbra_shielded_pool::spend;
use rand_core::{CryptoRng, RngCore};
use penumbra_transaction::plan::TransactionPlan; 
use penumbra_transaction::{
    action::Action,
    memo::MemoCiphertext,
    transaction::{DetectionData, TransactionParameters},
    AuthorizationData, AuthorizingData, Transaction, TransactionBody, WitnessData,
};
use penumbra_transaction::plan::ActionPlan;
use penumbra_transaction::plan::BuildPlan;

use wasm_bindgen_test::console_log;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;
use crate::error::{WasmResult, WasmError};
use crate::utils;
use std::str::FromStr;
use penumbra_proto::core::transaction::v1alpha1 as pb;
use crate::wasm_planner::WasmPlanner;
use penumbra_proto::DomainType;
use rand_core::OsRng;
use penumbra_proto::core::transaction::v1alpha1 as ps;

/// Build transaction in parallel
#[wasm_bindgen]
pub fn build_parallel(
    full_viewing_key: &str,
    transaction_plan: JsValue,
    witness_data: JsValue,
    auth_data: JsValue,
) -> WasmResult<JsValue> {
    utils::set_panic_hook();

    let plan_proto: pb::TransactionPlan = serde_wasm_bindgen::from_value(transaction_plan)?;
    let witness_data_proto: pb::WitnessData = serde_wasm_bindgen::from_value(witness_data)?;
    let witness_data_conv: WitnessData = witness_data_proto.try_into()?;
    
    let auth_data_proto: pb::AuthorizationData = serde_wasm_bindgen::from_value(auth_data)?;
    let auth_data_conv: AuthorizationData = auth_data_proto.try_into().unwrap();

    let fvk: FullViewingKey = FullViewingKey::from_str(full_viewing_key)
        .expect("The provided string is not a valid FullViewingKey");

    let plan: TransactionPlan = plan_proto.try_into()?;

    let tx_plan = build_tx_parallel(fvk, witness_data_conv, plan).unwrap();
    
    let tx = tx_plan.authorize(&mut OsRng, &auth_data_conv).unwrap();

    let value = serde_wasm_bindgen::to_value(&tx.to_proto())?;

    Ok(value)
}

pub fn build_tx_parallel(
    fvk: FullViewingKey,
    witness_data: WitnessData,
    plan: TransactionPlan
) -> Result<UnauthTransaction> {
    console_log!("Entered 'new' build method!");

    let mut actions = Vec::new();
    let mut synthetic_blinding_factor = Fr::zero();

    // Add the memo.
    let mut memo: Option<MemoCiphertext> = None;
    let mut memo_key: Option<PayloadKey> = None;
    if plan.memo_plan.is_some() {
        let memo_plan = plan
            .memo_plan
            .clone()
            .ok_or_else(|| anyhow!("missing memo_plan in TransactionPlan"))?;
        memo = memo_plan.memo().ok();
        memo_key = Some(memo_plan.key);
    }

    // We build the actions sorted by type, with all spends first, then all
    // outputs, etc.  This order has to align with the ordering in
    // TransactionPlan::effect_hash, which computes the auth hash of the
    // transaction we'll build here without actually building it.

    // Build the transaction's spends.
    for spend_plan in plan.spend_plans() {
        let spend = BuildPlan::Spend(spend_plan.clone());
        let spend = spend.build_action(&fvk, witness_data.clone(), memo_key.clone()).unwrap();
        actions.push(spend);
    }

    // need to handle the blinding factor!!

    for output_plan in plan.output_plans() {
        let output = BuildPlan::Output(output_plan.clone());
        let output = output.build_action(&fvk, witness_data.clone(), memo_key.clone()).unwrap();
        actions.push(output);
    }

    // Build the transaction's swaps.
    for swap_plan in plan.swap_plans() {
        synthetic_blinding_factor += swap_plan.fee_blinding;
        actions.push(Action::Swap(swap_plan.swap(&fvk)));
    }

    // Build the transaction's swap claims.
    for swap_claim_plan in plan.swap_claim_plans().cloned() {
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
    let detection_data: Option<DetectionData> = if plan.num_outputs() == 0 {
        None
    } else {
        let mut fmd_clues = Vec::new();
        for clue_plan in plan.clue_plans() {
            fmd_clues.push(clue_plan.clue());
        }
        Some(DetectionData { fmd_clues })
    };

    // All of these actions have "transparent" value balance with no
    // blinding factor, so they don't contribute to the
    // synthetic_blinding_factor used for the binding signature.

    for delegation in plan.delegations().cloned() {
        actions.push(Action::Delegate(delegation))
    }
    for undelegation in plan.undelegations().cloned() {
        actions.push(Action::Undelegate(undelegation))
    }
    for plan in plan.undelegate_claim_plans() {
        synthetic_blinding_factor += plan.balance_blinding;
        let undelegate_claim = plan.undelegate_claim();
        actions.push(Action::UndelegateClaim(undelegate_claim));
    }
    for proposal_submit in plan.proposal_submits().cloned() {
        actions.push(Action::ProposalSubmit(proposal_submit))
    }
    for proposal_withdraw_plan in plan.proposal_withdraws().cloned() {
        actions.push(Action::ProposalWithdraw(proposal_withdraw_plan));
    }
    for validator_vote in plan.validator_votes().cloned() {
        actions.push(Action::ValidatorVote(validator_vote))
    }
    for delegator_vote_plan in plan.delegator_vote_plans() {
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
    for proposal_deposit_claim in plan.proposal_deposit_claims().cloned() {
        actions.push(Action::ProposalDepositClaim(proposal_deposit_claim))
    }
    for vd in plan.validator_definitions().cloned() {
        actions.push(Action::ValidatorDefinition(vd))
    }
    for ibc_action in plan.ibc_actions().cloned() {
        actions.push(Action::IbcAction(ibc_action))
    }
    for dao_spend in plan.dao_spends().cloned() {
        actions.push(Action::DaoSpend(dao_spend))
    }
    for dao_output in plan.dao_outputs().cloned() {
        actions.push(Action::DaoOutput(dao_output))
    }
    for dao_deposit in plan.dao_deposits().cloned() {
        actions.push(Action::DaoDeposit(dao_deposit))
    }
    for position_open in plan.position_openings().cloned() {
        actions.push(Action::PositionOpen(position_open))
    }
    for position_close in plan.position_closings().cloned() {
        actions.push(Action::PositionClose(position_close))
    }
    for position_withdraw in plan.position_withdrawals() {
        actions.push(Action::PositionWithdraw(
            position_withdraw.position_withdraw(),
        ))
    }
    // build the transaction's ICS20 withdrawals
    for ics20_withdrawal in plan.ics20_withdrawals() {
        actions.push(Action::Ics20Withdrawal(ics20_withdrawal.clone()))
    }

    let transaction_body = TransactionBody {
        actions,
        transaction_parameters: TransactionParameters {
            expiry_height: plan.expiry_height,
            chain_id: plan.chain_id,
        },
        fee: plan.fee,
        detection_data,
        memo,
    };

    Ok(UnauthTransaction {
        inner: Transaction {
            transaction_body,
            anchor: witness_data.anchor,
            binding_sig: [0; 64].into(),
        },
        synthetic_blinding_factor,
    })
}

/// A partially-constructed transaction awaiting authorization data.
pub struct UnauthTransaction {
    inner: Transaction,
    synthetic_blinding_factor: Fr, // Not serializable (arkwork library)
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
            anyhow::bail!(
                "expected {} spend auths but got {}",
                spend_count,
                auth_data.spend_auths.len()
            );
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

        for (delegator_vote, auth_sig) in self
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
