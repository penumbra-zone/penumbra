use decaf377::Fq;
use penumbra_asset::{Balance, Value};
use penumbra_keys::{
    keys::{IncomingViewingKey, NullifierKey},
    FullViewingKey,
};
use penumbra_proof_params::SWAPCLAIM_PROOF_PROVING_KEY;
use penumbra_proto::{penumbra::core::component::dex::v1 as pb, DomainType};
use penumbra_sct::Nullifier;
use penumbra_tct as tct;

use serde::{Deserialize, Serialize};
use tct::Position;

use crate::{swap::SwapPlaintext, BatchSwapOutputData};

use super::{
    action as swap_claim,
    proof::{SwapClaimProof, SwapClaimProofPrivate, SwapClaimProofPublic},
    SwapClaim,
};

/// A planned [`SwapClaim`](SwapClaim).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(try_from = "pb::SwapClaimPlan", into = "pb::SwapClaimPlan")]
pub struct SwapClaimPlan {
    pub swap_plaintext: SwapPlaintext,
    pub position: Position,
    pub output_data: BatchSwapOutputData,
    pub epoch_duration: u64,
    /// The first blinding factor used for generating the ZK proof.
    pub proof_blinding_r: Fq,
    /// The second blinding factor used for generating the ZK proof.
    pub proof_blinding_s: Fq,
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
        let (lambda_1, lambda_2) = self
            .output_data
            .pro_rata_outputs((self.swap_plaintext.delta_1_i, self.swap_plaintext.delta_2_i));
        let (output_rseed_1, output_rseed_2) = self.swap_plaintext.output_rseeds();
        let note_blinding_1 = output_rseed_1.derive_note_blinding();
        let note_blinding_2 = output_rseed_2.derive_note_blinding();
        let (output_1_note, output_2_note) = self.swap_plaintext.output_notes(&self.output_data);
        let note_commitment_1 = output_1_note.commit();
        let note_commitment_2 = output_2_note.commit();

        let nullifier =
            Nullifier::derive(nk, self.position, &self.swap_plaintext.swap_commitment());
        SwapClaimProof::prove(
            self.proof_blinding_r,
            self.proof_blinding_s,
            &SWAPCLAIM_PROOF_PROVING_KEY,
            SwapClaimProofPublic {
                anchor: state_commitment_proof.root(),
                nullifier,
                claim_fee: self.swap_plaintext.claim_fee.clone(),
                output_data: self.output_data,
                note_commitment_1,
                note_commitment_2,
            },
            SwapClaimProofPrivate {
                swap_plaintext: self.swap_plaintext.clone(),
                state_commitment_proof: state_commitment_proof.clone(),
                nk: *nk,
                lambda_1,
                lambda_2,
                note_blinding_1,
                note_blinding_2,
            },
        )
        .expect("can generate ZKSwapClaimProof")
    }

    /// Construct the [`swap_claim::Body`] described by this plan.
    pub fn swap_claim_body(&self, fvk: &FullViewingKey) -> swap_claim::Body {
        let (output_1_note, output_2_note) = self.swap_plaintext.output_notes(&self.output_data);
        tracing::debug!(?output_1_note, ?output_2_note);

        // We need to get the correct diversified generator to use with DH:
        let output_1_commitment = output_1_note.commit();
        let output_2_commitment = output_2_note.commit();

        let nullifier = Nullifier::derive(
            fvk.nullifier_key(),
            self.position,
            &self.swap_plaintext.swap_commitment(),
        );

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

    pub fn balance(&self) -> Balance {
        // Only the pre-paid fee is contributed to the value balance
        // The rest is handled internally to the SwapClaim action.
        let value_fee = Value {
            amount: self.swap_plaintext.claim_fee.amount(),
            asset_id: self.swap_plaintext.claim_fee.asset_id(),
        };

        value_fee.into()
    }
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
            proof_blinding_r: msg.proof_blinding_r.to_bytes().to_vec(),
            proof_blinding_s: msg.proof_blinding_s.to_bytes().to_vec(),
        }
    }
}

impl TryFrom<pb::SwapClaimPlan> for SwapClaimPlan {
    type Error = anyhow::Error;
    fn try_from(msg: pb::SwapClaimPlan) -> Result<Self, Self::Error> {
        let proof_blinding_r_bytes: [u8; 32] = msg
            .proof_blinding_r
            .try_into()
            .map_err(|_| anyhow::anyhow!("malformed r in `SwapClaimPlan`"))?;
        let proof_blinding_s_bytes: [u8; 32] = msg
            .proof_blinding_s
            .try_into()
            .map_err(|_| anyhow::anyhow!("malformed s in `SwapClaimPlan`"))?;

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
            proof_blinding_r: Fq::from_bytes_checked(&proof_blinding_r_bytes)
                .expect("proof_blinding_r malformed"),
            proof_blinding_s: Fq::from_bytes_checked(&proof_blinding_s_bytes)
                .expect("proof_blinding_s malformed"),
        })
    }
}
