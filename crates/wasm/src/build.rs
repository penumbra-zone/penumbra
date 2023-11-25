use anyhow;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;
use crate::error::WasmResult;
use std::str::FromStr;

use crate::utils;
use penumbra_transaction::{action::Action, plan::ActionPlan, plan::TransactionPlan, WitnessData};
use penumbra_keys::FullViewingKey;
use penumbra_proto::core::transaction::v1alpha1 as pb;

/// Builds a planned [`Action`] specified by
/// the [`ActionPlan`] in a [`TransactionPlan`].
/// Arguments:
///     transaction_plan: `TransactionPlan`
///     action_plan: `ActionPlan`
///     full_viewing_key: `bech32m String`,
///     witness_data: `WitnessData``
/// Returns: `Action`
#[wasm_bindgen]
pub fn build_action(
    transaction_plan: JsValue,
    action_plan: JsValue,
    full_viewing_key: &str,
    witness_data: JsValue,
) -> WasmResult<JsValue> {
    utils::set_panic_hook();

    let transaction_plan: TransactionPlan =
        serde_wasm_bindgen::from_value(transaction_plan.clone())?;

    let witness_data_proto: pb::WitnessData = serde_wasm_bindgen::from_value(witness_data)?;
    let witness_data: WitnessData = witness_data_proto.try_into()?;

    let action_plan: ActionPlan = serde_wasm_bindgen::from_value(action_plan)?;

    let full_viewing_key: FullViewingKey = FullViewingKey::from_str(full_viewing_key)?;

    let memo_key = transaction_plan.memo_plan.map(|memo_plan| memo_plan.key);

    let action = match action_plan {
        ActionPlan::Spend(spend_plan) => {
            let spend = ActionPlan::Spend(spend_plan);
            Some(spend.build_unauth(&full_viewing_key, &witness_data, memo_key)?)
        }
        ActionPlan::Output(output_plan) => {
            let output = ActionPlan::Output(output_plan);
            Some(output.build_unauth(&full_viewing_key, &witness_data, memo_key)?)
        }

        // TODO: Other action variants besides 'Spend' and 'Output' still require testing.
        ActionPlan::Swap(swap_plan) => {
            let swap = ActionPlan::Swap(swap_plan);
            Some(swap.build_unauth(&full_viewing_key, &witness_data, memo_key)?)
        }
        ActionPlan::SwapClaim(swap_claim_plan) => {
            let swap_claim = ActionPlan::SwapClaim(swap_claim_plan);
            Some(swap_claim.build_unauth(&full_viewing_key, &witness_data, memo_key)?)
        }
        ActionPlan::Delegate(delegation) => Some(Action::Delegate(delegation)),
        ActionPlan::Undelegate(undelegation) => Some(Action::Undelegate(undelegation)),
        ActionPlan::UndelegateClaim(undelegate_claim) => {
            let undelegate_claim = undelegate_claim.undelegate_claim();
            Some(Action::UndelegateClaim(undelegate_claim))
        }
        ActionPlan::ProposalSubmit(proposal_submit) => {
            Some(Action::ProposalSubmit(proposal_submit))
        }
        ActionPlan::ProposalWithdraw(proposal_withdraw) => {
            Some(Action::ProposalWithdraw(proposal_withdraw))
        }
        ActionPlan::ValidatorVote(validator_vote) => {
            Some(Action::ValidatorVote(validator_vote))
        }
        ActionPlan::DelegatorVote(delegator_vote) => {
            let note_commitment = delegator_vote.staked_note.commit();
            let auth_path = witness_data
                .state_commitment_proofs
                .get(&note_commitment)
                .context(format!("could not get proof for {note_commitment:?}"))?;

            Some(Action::DelegatorVote(delegator_vote.delegator_vote(
                &full_viewing_key,
                [0; 64].into(),
                auth_path.clone(),
            )))
        }
        ActionPlan::ProposalDepositClaim(proposal_deposit_claim) => {
            Some(Action::ProposalDepositClaim(proposal_deposit_claim))
        }
        ActionPlan::ValidatorDefinition(validator_definition) => {
            Some(Action::ValidatorDefinition(validator_definition))
        }
        ActionPlan::IbcAction(ibc_action) => Some(Action::IbcRelay(ibc_action)),
        ActionPlan::DaoSpend(dao_spend) => Some(Action::DaoSpend(dao_spend)),
        ActionPlan::DaoOutput(dao_output) => Some(Action::DaoOutput(dao_output)),
        ActionPlan::DaoDeposit(dao_deposit) => Some(Action::DaoDeposit(dao_deposit)),
        ActionPlan::PositionOpen(position_open) => Some(Action::PositionOpen(position_open)),
        ActionPlan::PositionClose(position_close) => {
            Some(Action::PositionClose(position_close))
        }
        ActionPlan::PositionWithdraw(position_withdrawn) => Some(Action::PositionWithdraw(
            position_withdrawn.position_withdraw(),
        )),
        ActionPlan::Withdrawal(ics20_withdrawal) => {
            Some(Action::Ics20Withdrawal(ics20_withdrawal))
        }
        _ => return Err(anyhow::anyhow!("Unsupported action type").into()),
    };

    let action_result_proto = serde_wasm_bindgen::to_value(&Some(action))?;
    Ok(action_result_proto)
}