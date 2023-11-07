use anyhow::{anyhow, Context, Result, Ok};
use ark_ff::Zero;
use decaf377::Fr;
use penumbra_keys::{symmetric::PayloadKey, FullViewingKey};
use serde::{Deserialize, Serialize};
use penumbra_shielded_pool::{OutputPlan, SpendPlan};
use crate::plan::ActionPlan;
use crate::{action::Action, WitnessData};
use wasm_bindgen_test::console_log;

/// A declaration of a [`Buildplan`](crate::Transaction),
/// for use in building a complete action.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum BuildPlan {
    Spend(SpendPlan),
    Output(OutputPlan),
}

impl BuildPlan {
    pub fn build_action(
        &self,
        fvk: &FullViewingKey,
        witness_data: WitnessData,
        memo_key: Option<PayloadKey>,
    ) -> Result<Action> {
        match self {
            BuildPlan::Spend(spend_plan) => {
                console_log!("Building spend action!");
                // Define empty blinding factor
                let mut synthetic_blinding_factor = Fr::zero();

                let note_commitment = spend_plan.note.commit();
                let auth_path = witness_data
                    .state_commitment_proofs
                    .get(&note_commitment)
                    .context(format!("could not get proof for {note_commitment:?}"))?;

                synthetic_blinding_factor += spend_plan.value_blinding;
                let spend = Action::Spend(spend_plan.spend(
                    &fvk,
                    [0; 64].into(),
                    auth_path.clone(),
                    witness_data.anchor,
                ));

                Ok(spend)
            }
            BuildPlan::Output(output_plan) => {
                console_log!("Building output action!");

                // Define empty blinding factor
                let mut synthetic_blinding_factor = Fr::zero();

                let dummy_payload_key: PayloadKey = [0u8; 32].into();
                synthetic_blinding_factor += output_plan.value_blinding;
                let output = Action::Output(output_plan.output(
                    &fvk.outgoing(),
                    memo_key.as_ref().unwrap_or(&dummy_payload_key),
                ));

                Ok(output)
            }
        }
    }
}
