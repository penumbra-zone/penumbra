use ark_ff::UniformRand;
use decaf377::FieldExt;
use penumbra_crypto::{
    asset,
    dex::{swap::generate_swap_asset_id, BatchSwapOutputData, TradingPair},
    ka,
    keys::{IncomingViewingKey, NullifierKey},
    proofs::transparent::SwapClaimProof,
    transaction::Fee,
    Address, Fq, Fr, FullViewingKey, Note, NotePayload, Nullifier, Value,
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
    pub swap_nft_asset_id: asset::Id,
    pub nk: NullifierKey,
    // Blinding factor used for Swap NFT
    pub note_blinding: Fq,
    // TODO: rename "note inclusion proof"
    pub note_commitment_proof: tct::Proof,
}

impl SwapClaimPlan {
    /// Create a new [`SwapClaimPlan`] that redeems output notes to `claim_address` using
    /// the associated swap NFT.
    pub fn new<R: RngCore + CryptoRng>(
        rng: &mut R,
        nullifier: Nullifier,
        claim_address: Address,
        fee: Fee,
        output_data: BatchSwapOutputData,
        anchor: tct::Root,
        trading_pair: TradingPair,
        nk: NullifierKey,
        note_blinding: Fq,
        note_commitment_proof: tct::Proof,
    ) -> SwapClaimPlan {
        let output_1_blinding = Fq::rand(rng);
        let output_2_blinding = Fq::rand(rng);
        let value_blinding = Fr::rand(rng);
        let esk_1 = ka::Secret::new(rng);
        let esk_2 = ka::Secret::new(rng);
        let swap_nft_asset_id = generate_swap_asset_id(
            output_data.delta_1,
            output_data.delta_2,
            fee.0,
            *claim_address.diversified_generator(),
            *claim_address.transmission_key(),
            trading_pair,
        )
        .expect("bad public key when generating swap asset id");

        Self {
            nullifier,
            claim_address,
            esk_1,
            esk_2,
            output_1_blinding,
            output_2_blinding,
            fee,
            output_data,
            anchor,
            trading_pair,
            swap_nft_asset_id,
            nk,
            note_blinding,
            note_commitment_proof,
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
            swap_nft_asset_id: self.swap_nft_asset_id,
            b_d: *self.claim_address.diversified_generator(),
            pk_d: *self.claim_address.transmission_key(),
            nk: self.nk,
            note_commitment_proof: self.note_commitment_proof,
            trading_pair: self.trading_pair,
            note_blinding: self.note_blinding,
            delta_1: self.output_data.delta_1,
            delta_2: self.output_data.delta_2,
            lambda_1: self.output_data.lambda_1,
            lambda_2: self.output_data.lambda_2,
            note_blinding_1: self.output_1_blinding,
            note_blinding_2: self.output_2_blinding,
            esk_1: self.esk_1,
            esk_2: self.esk_2,
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
        ivk.views_address(&self.claim_address)
    }
}

impl Protobuf<pb::SwapClaimPlan> for SwapClaimPlan {}

impl From<SwapClaimPlan> for pb::SwapClaimPlan {
    fn from(msg: SwapClaimPlan) -> Self {
        Self {
            nullifier: msg.nullifier.to_bytes().to_vec().into(),
            claim_address: Some(msg.claim_address.into()),
            fee: Some(msg.fee.into()),
            output_data: Some(msg.output_data.into()),
            anchor: Some(msg.anchor.into()),
            trading_pair: Some(msg.trading_pair.into()),
            output_1_blinding: msg.output_1_blinding.to_bytes().to_vec().into(),
            output_2_blinding: msg.output_2_blinding.to_bytes().to_vec().into(),
            esk_1: msg.esk_1.to_bytes().to_vec().into(),
            esk_2: msg.esk_2.to_bytes().to_vec().into(),
            swap_nft_asset_id: Some(msg.swap_nft_asset_id.into()),
            nk: msg.nk.0.to_bytes().to_vec().into(),
            note_blinding: msg.note_blinding.to_bytes().to_vec().into(),
            note_commitment_proof: msg.note_commitment_proof.into(),
        }
    }
}

impl TryFrom<pb::SwapClaimPlan> for SwapClaimPlan {
    type Error = anyhow::Error;
    fn try_from(msg: pb::SwapClaimPlan) -> Result<Self, Self::Error> {
        Ok(Self {
            nullifier: msg.nullifier.try_into()?,
            fee: todo!(),
            output_data: todo!(),
            anchor: todo!(),
            trading_pair: todo!(),
            claim_address: todo!(),
            output_1_blinding: todo!(),
            output_2_blinding: todo!(),
            esk_1: todo!(),
            esk_2: todo!(),
            swap_nft_asset_id: todo!(),
            nk: todo!(),
            note_blinding: todo!(),
            note_commitment_proof: todo!(),
        })
    }
}
