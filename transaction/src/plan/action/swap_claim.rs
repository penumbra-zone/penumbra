use penumbra_crypto::{
    dex::{swap::SwapPlaintext, BatchSwapOutputData},
    keys::{IncomingViewingKey, NullifierKey},
    proofs::transparent::SwapClaimProof,
    FullViewingKey, Value,
};
use penumbra_proto::{core::dex::v1alpha1 as pb, DomainType, TypeUrl};
use penumbra_tct as tct;

use serde::{Deserialize, Serialize};
use tct::Position;

use crate::action::{swap_claim, SwapClaim};

/// A planned [`SwapClaim`](SwapClaim).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(try_from = "pb::SwapClaimPlan", into = "pb::SwapClaimPlan")]
pub struct SwapClaimPlan {
    pub swap_plaintext: SwapPlaintext,
    pub position: Position,
    pub output_data: BatchSwapOutputData,
    pub epoch_duration: u64,
}

impl SwapClaimPlan {
    /// Convenience method to construct the [`SwapClaim`] described by this
    /// [`SwapClaimPlan`].
    pub fn swap_claim(
        &self,
        fvk: &FullViewingKey,
        state_commitment_proof: &tct::Proof,
    ) -> SwapClaim {
        SwapClaim {
            body: self.swap_claim_body(fvk),
            proof: self.swap_claim_proof(state_commitment_proof, fvk.nullifier_key()),
            epoch_duration: self.epoch_duration,
        }
    }

    /// Construct the [`SwapClaimProof`] required by the [`swap_claim::Body`] described
    /// by this plan.
    pub fn swap_claim_proof(
        &self,
        state_commitment_proof: &tct::Proof,
        nk: &NullifierKey,
    ) -> SwapClaimProof {
        let (lambda_1_i, lambda_2_i) = self.output_data.pro_rata_outputs((
            self.swap_plaintext.delta_1_i.into(),
            self.swap_plaintext.delta_2_i.into(),
        ));

        SwapClaimProof {
            swap_plaintext: self.swap_plaintext.clone(),
            lambda_1_i,
            lambda_2_i,
            nk: nk.clone(),
            swap_commitment_proof: state_commitment_proof.clone(),
        }
    }

    /// Construct the [`swap_claim::Body`] described by this plan.
    pub fn swap_claim_body(&self, fvk: &FullViewingKey) -> swap_claim::Body {
        let (output_1_note, output_2_note) = self.swap_plaintext.output_notes(&self.output_data);
        tracing::debug!(?output_1_note, ?output_2_note);

        // We need to get the correct diversified generator to use with DH:
        let output_1_commitment = output_1_note.commit();
        let output_2_commitment = output_2_note.commit();

        let nullifier = fvk.derive_nullifier(self.position, &self.swap_plaintext.swap_commitment());

        swap_claim::Body {
            nullifier,
            fee: self.swap_plaintext.claim_fee.clone(),
            output_1_commitment,
            output_2_commitment,
            output_data: self.output_data,
        }
    }

    /// Checks whether this plan's output is viewed by the given IVK.
    pub fn is_viewed_by(&self, ivk: &IncomingViewingKey) -> bool {
        ivk.views_address(&self.swap_plaintext.claim_address)
    }

    pub fn balance(&self) -> penumbra_crypto::Balance {
        // Only the pre-paid fee is contributed to the value balance
        // The rest is handled internally to the SwapClaim action.
        let value_fee = Value {
            amount: self.swap_plaintext.claim_fee.amount(),
            asset_id: self.swap_plaintext.claim_fee.asset_id(),
        };

        value_fee.into()
    }
}

impl TypeUrl for SwapClaimPlan {
    const TYPE_URL: &'static str = "/penumbra.core.dex.v1alpha1.SwapClaimPlan";
}

impl DomainType for SwapClaimPlan {
    type Proto = pb::SwapClaimPlan;
}

impl From<SwapClaimPlan> for pb::SwapClaimPlan {
    fn from(msg: SwapClaimPlan) -> Self {
        Self {
            swap_plaintext: Some(msg.swap_plaintext.into()),
            position: msg.position.into(),
            output_data: Some(msg.output_data.into()),
            epoch_duration: msg.epoch_duration,
        }
    }
}

impl TryFrom<pb::SwapClaimPlan> for SwapClaimPlan {
    type Error = anyhow::Error;
    fn try_from(msg: pb::SwapClaimPlan) -> Result<Self, Self::Error> {
        Ok(Self {
            swap_plaintext: msg
                .swap_plaintext
                .ok_or_else(|| anyhow::anyhow!("missing swap_plaintext"))?
                .try_into()?,
            position: msg.position.into(),
            output_data: msg
                .output_data
                .ok_or_else(|| anyhow::anyhow!("missing output_data"))?
                .try_into()?,
            epoch_duration: msg.epoch_duration,
        })
    }
}
