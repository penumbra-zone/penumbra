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
use tct::Position;

use crate::action::{swap_claim, SwapClaim};

// TODO: copied directly from `OutputPlan` right now, needs fields updated
// for `SwapClaim`
/// A planned [`SwapClaim`](SwapClaim).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(try_from = "pb::SwapClaimPlan", into = "pb::SwapClaimPlan")]
pub struct SwapClaimPlan {
    pub swap_nft_note: Note,
    pub swap_nft_position: Position,
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
}

impl SwapClaimPlan {
    /// Create a new [`SwapClaimPlan`] that redeems output notes to `claim_address` using
    /// the associated swap NFT.
    pub fn new<R: RngCore + CryptoRng>(
        rng: &mut R,
        swap_nft_note: Note,
        swap_nft_position: Position,
        claim_address: Address,
        fee: Fee,
        output_data: BatchSwapOutputData,
        anchor: tct::Root,
        trading_pair: TradingPair,
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
            swap_nft_note,
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
            swap_nft_position,
        }
    }

    /// Convenience method to construct the [`SwapClaim`] described by this
    /// [`SwapClaimPlan`].
    pub fn swap_claim(
        &self,
        fvk: &FullViewingKey,
        note_commitment_proof: tct::Proof,
        nk: NullifierKey,
        note_blinding: Fq,
    ) -> SwapClaim {
        SwapClaim {
            body: self.swap_claim_body(fvk),
            zkproof: self.swap_claim_proof(note_commitment_proof, nk, note_blinding),
        }
    }

    /// Construct the [`SwapClaimProof`] required by the [`swap_claim::Body`] described
    /// by this plan.
    pub fn swap_claim_proof(
        &self,
        note_commitment_proof: tct::Proof,
        nk: NullifierKey,
        note_blinding: Fq,
    ) -> SwapClaimProof {
        SwapClaimProof {
            swap_nft_asset_id: self.swap_nft_asset_id,
            b_d: *self.claim_address.diversified_generator(),
            pk_d: *self.claim_address.transmission_key(),
            nk,
            note_commitment_proof,
            trading_pair: self.trading_pair,
            note_blinding,
            delta_1: self.output_data.delta_1,
            delta_2: self.output_data.delta_2,
            lambda_1: self.output_data.lambda_1,
            lambda_2: self.output_data.lambda_2,
            note_blinding_1: self.output_1_blinding,
            note_blinding_2: self.output_2_blinding,
            esk_1: self.esk_1.clone(),
            esk_2: self.esk_2.clone(),
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

        let nullifier = fvk.derive_nullifier(self.swap_nft_position, &self.swap_nft_note.commit());

        swap_claim::Body {
            nullifier,
            fee: self.fee.clone(),
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
            swap_nft_note: Some(msg.swap_nft_note.into()),
            swap_nft_position: msg.swap_nft_position.into(),
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
        }
    }
}

impl TryFrom<pb::SwapClaimPlan> for SwapClaimPlan {
    type Error = anyhow::Error;
    fn try_from(msg: pb::SwapClaimPlan) -> Result<Self, Self::Error> {
        Ok(Self {
            swap_nft_note: msg
                .swap_nft_note
                .ok_or_else(|| anyhow::anyhow!("missing swap_nft_note"))?
                .try_into()?,
            swap_nft_position: msg.swap_nft_position.try_into()?,
            fee: msg
                .fee
                .ok_or_else(|| anyhow::anyhow!("missing fee"))?
                .try_into()?,
            output_data: msg
                .output_data
                .ok_or_else(|| anyhow::anyhow!("missing output_data"))?
                .try_into()?,
            anchor: msg
                .anchor
                .ok_or_else(|| anyhow::anyhow!("missing anchor"))?
                .try_into()?,
            trading_pair: msg
                .trading_pair
                .ok_or_else(|| anyhow::anyhow!("missing trading_pair"))?
                .try_into()?,
            claim_address: msg
                .claim_address
                .ok_or_else(|| anyhow::anyhow!("missing claim_address"))?
                .try_into()?,
            output_1_blinding: Fq::from_bytes(msg.output_1_blinding.as_ref().try_into()?)?,
            output_2_blinding: Fq::from_bytes(msg.output_2_blinding.as_ref().try_into()?)?,
            esk_1: msg.esk_1.as_ref().try_into()?,
            esk_2: msg.esk_2.as_ref().try_into()?,
            swap_nft_asset_id: msg
                .swap_nft_asset_id
                .ok_or_else(|| anyhow::anyhow!("missing swap_nft_asset_id"))?
                .try_into()?,
        })
    }
}
