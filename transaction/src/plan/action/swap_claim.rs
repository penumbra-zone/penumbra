use ark_ff::UniformRand;
use penumbra_crypto::{
    ka,
    keys::{IncomingViewingKey, OutgoingViewingKey},
    memo::MemoPlaintext,
    proofs::transparent::SwapClaimProof,
    Address, FieldExt, Fq, Fr, Note, NotePayload, Value,
};
use penumbra_proto::{transaction as pb, Protobuf};
use rand_core::{CryptoRng, RngCore};
use serde::{Deserialize, Serialize};

use crate::action::{swap_claim, SwapClaim};

// TODO: copied directly from `OutputPlan` right now, needs fields updated
// for `SwapClaim`
/// A planned [`SwapClaim`](SwapClaim).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(try_from = "pb::SwapClaimPlan", into = "pb::SwapClaimPlan")]
pub struct SwapClaimPlan {
    pub value: Value,
    pub dest_address: Address,
    pub memo: MemoPlaintext,
    pub note_blinding: Fq,
    pub value_blinding: Fr,
    pub esk: ka::Secret,
}

impl SwapClaimPlan {
    /// Create a new [`SwapClaimPlan`] that sends `value` to `dest_address` with
    /// the provided `memo`.
    pub fn new<R: RngCore + CryptoRng>(
        rng: &mut R,
        value: Value,
        dest_address: Address,
        memo: MemoPlaintext,
    ) -> SwapClaimPlan {
        let note_blinding = Fq::rand(rng);
        let value_blinding = Fr::rand(rng);
        let esk = ka::Secret::new(rng);
        Self {
            value,
            dest_address,
            memo,
            note_blinding,
            value_blinding,
            esk,
        }
    }

    /// Convenience method to construct the [`SwapClaim`] described by this
    /// [`SwapClaimPlan`].
    pub fn swap_claim(&self, ovk: &OutgoingViewingKey) -> SwapClaim {
        SwapClaim {
            body: self.swap_claim_body(ovk),
            zkproof: self.swap_claim_proof(),
        }
    }

    pub fn swap_claim_note(&self) -> Note {
        let diversifier = self.dest_address.diversifier().clone();
        let transmission_key = self.dest_address.transmission_key().clone();
        Note::from_parts(
            diversifier,
            transmission_key,
            self.value,
            self.note_blinding,
        )
        .expect("transmission key in address is always valid")
    }

    /// Construct the [`SwapClaimProof`] required by the [`swap_claim::Body`] described
    /// by this plan.
    pub fn swap_claim_proof(&self) -> SwapClaimProof {
        SwapClaimProof {
            swap_nft_asset_id: self.dest_address.asset_id().clone(),
            b_d: self.output_note().diversified_generator(),
            pk_d: self.dest_address.transmission_key().clone(),
            nk: todo!(),
            note_commitment_proof: todo!(),
            trading_pair: todo!(),
            note_blinding: self.note_blinding,
            delta_1: todo!(),
            delta_2: todo!(),
            lambda_1: todo!(),
            lambda_2: todo!(),
            note_blinding_1: todo!(),
            note_blinding_2: todo!(),
            esk_1: todo!(),
            esk_2: todo!(),
        }
    }

    /// Construct the [`swap_claim::Body`] described by this plan.
    pub fn swap_claim_body(&self, ovk: &OutgoingViewingKey) -> swap_claim::Body {
        // Prepare the output note and commitment.
        let note = self.output_note();
        let note_commitment = note.commit();

        // Prepare the value commitment.  Outputs subtract from the transaction
        // value balance, so flip the sign of the commitment.
        let value_commitment = -self.value.commit(self.value_blinding);

        // Encrypt the note to the recipient...
        let diversified_generator = note.diversified_generator();
        let ephemeral_key = self.esk.diversified_public(&diversified_generator);
        let encrypted_note = note.encrypt(&self.esk);
        let encrypted_memo = self.memo.encrypt(&self.esk, &self.dest_address);
        // ... and wrap the encryption key to ourselves.
        let ovk_wrapped_key = note.encrypt_key(&self.esk, ovk, value_commitment);

        swap_claim::Body {
            nullifier: todo!(),
            fee: todo!(),
            output_1: todo!(),
            output_2: todo!(),
            output_data: todo!(),
            anchor: todo!(),
            trading_pair: todo!(),
        }
    }

    /// Checks whether this plan's output is viewed by the given IVK.
    pub fn is_viewed_by(&self, ivk: &IncomingViewingKey) -> bool {
        ivk.views_address(&self.dest_address)
    }
}

impl Protobuf<pb::SwapClaimPlan> for SwapClaimPlan {}

impl From<SwapClaimPlan> for pb::SwapClaimPlan {
    fn from(msg: SwapClaimPlan) -> Self {
        Self {
            value: Some(msg.value.into()),
            dest_address: Some(msg.dest_address.into()),
            memo: msg.memo.0.to_vec().into(),
            note_blinding: msg.note_blinding.to_bytes().to_vec().into(),
            value_blinding: msg.value_blinding.to_bytes().to_vec().into(),
            esk: msg.esk.to_bytes().to_vec().into(),
        }
    }
}

impl TryFrom<pb::SwapClaimPlan> for SwapClaimPlan {
    type Error = anyhow::Error;
    fn try_from(msg: pb::SwapClaimPlan) -> Result<Self, Self::Error> {
        Ok(Self {
            value: msg
                .value
                .ok_or_else(|| anyhow::anyhow!("missing value"))?
                .try_into()?,
            dest_address: msg
                .dest_address
                .ok_or_else(|| anyhow::anyhow!("missing address"))?
                .try_into()?,
            memo: msg.memo.as_ref().try_into()?,
            note_blinding: Fq::from_bytes(msg.note_blinding.as_ref().try_into()?)?,
            value_blinding: Fr::from_bytes(msg.value_blinding.as_ref().try_into()?)?,
            esk: msg.esk.as_ref().try_into()?,
        })
    }
}
