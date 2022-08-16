use ark_ff::UniformRand;
use penumbra_crypto::{
    dex::{BatchSwapOutputData, TradingPair},
    ka,
    keys::IncomingViewingKey,
    memo::MemoPlaintext,
    note,
    proofs::transparent::SwapClaimProof,
    transaction::Fee,
    Address, FieldExt, Fq, Fr, FullViewingKey, Note, NotePayload, Nullifier, Value,
};
use penumbra_proto::{transaction as pb, Protobuf};
use penumbra_tct as tct;
use rand_core::{CryptoRng, RngCore};
use serde::{Deserialize, Serialize};

use crate::action::{swap_claim, SwapClaim};

// TODO: copied directly from `OutputPlan` right now, needs fields updated
// for `SwapClaim`
/// A planned [`SwapClaim`](SwapClaim).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(try_from = "pb::SwapClaimPlan", into = "pb::SwapClaimPlan")]
pub struct SwapClaimPlan {
    pub nullifier: Nullifier,
    pub fee: Fee,
    pub output_data: BatchSwapOutputData,
    pub anchor: tct::Root,
    pub trading_pair: TradingPair,
    pub claim_address: Address,
    pub output_1_blinding: Fq,
    pub output_2_blinding: Fq,
    pub esk_1: ka::Secret,
    pub esk_2: ka::Secret,
    pub ovk_wrapped_key_1: [u8; note::OVK_WRAPPED_LEN_BYTES],
    pub ovk_wrapped_key_2: [u8; note::OVK_WRAPPED_LEN_BYTES],
}

impl SwapClaimPlan {
    /// Create a new [`SwapClaimPlan`] that redeems output notes to `claim_address` using
    /// the associated swap NFT.
    pub fn new<R: RngCore + CryptoRng>(
        rng: &mut R,
        value: Value,
        claim_address: Address,
    ) -> SwapClaimPlan {
        let output_1_blinding = Fq::rand(rng);
        let output_2_blinding = Fq::rand(rng);
        let value_blinding = Fr::rand(rng);
        let esk_1 = ka::Secret::new(rng);
        let esk_2 = ka::Secret::new(rng);
        Self {
            value,
            claim_address,
            note_blinding,
            value_blinding,
            esk_1,
            esk_2,
            output_1_blinding,
            output_2_blinding,
        }
    }

    /// Convenience method to construct the [`SwapClaim`] described by this
    /// [`SwapClaimPlan`].
    pub fn swap_claim(&self, fvk: &FullViewingKey) -> SwapClaim {
        SwapClaim {
            body: self.swap_claim_body(fvk),
            zkproof: self.swap_claim_proof(),
        }
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
    pub fn swap_claim_body(&self, fvk: &FullViewingKey) -> swap_claim::Body {
        let diversifier = self.claim_address.diversifier().clone();
        let transmission_key = self.claim_address.transmission_key().clone();

        let output_1_note = Note::from_parts(
            diversifier,
            transmission_key,
            Value {
                amount: self.output_data.lambda_1,
                asset_id: self.trading_pair.asset_1(),
            },
            self.output_1_blinding,
        )
        .expect("transmission key in address is always valid");
        let output_2_note = Note::from_parts(
            diversifier,
            transmission_key,
            Value {
                amount: self.output_data.lambda_2,
                asset_id: self.trading_pair.asset_2(),
            },
            self.output_2_blinding,
        )
        .expect("transmission key in address is always valid");

        let output_1 = NotePayload {
            note_commitment: output_1_note.commit(),
            ephemeral_key: self.esk_1.public(),
            encrypted_note: output_1_note.encrypt(&self.esk_1),
        };
        let output_2 = NotePayload {
            note_commitment: output_2_note.commit(),
            ephemeral_key: self.esk_2.public(),
            encrypted_note: output_2_note.encrypt(&self.esk_2),
        };

        swap_claim::Body {
            nullifier: self.nullifier,
            fee: self.fee,
            output_1,
            output_2,
            output_data: self.output_data,
            anchor: self.anchor,
            trading_pair: self.trading_pair,
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
