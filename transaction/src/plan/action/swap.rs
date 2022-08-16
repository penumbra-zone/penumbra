use ark_ff::UniformRand;
use decaf377_rdsa::{Signature, SpendAuth};
use penumbra_crypto::{
    dex::TradingPair,
    proofs::transparent::{SpendProof, SwapProof},
    transaction::Fee,
    value, FieldExt, Fr, FullViewingKey, Note, NotePayload,
};
use penumbra_proto::{transaction as pb, Protobuf};
use penumbra_tct as tct;
use rand_core::{CryptoRng, RngCore};
use serde::{Deserialize, Serialize};

use crate::action::{swap, Swap};

/// A planned [`Swap`](Swap).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(try_from = "pb::SwapPlan", into = "pb::SwapPlan")]
pub struct SwapPlan {
    pub trading_pair: TradingPair,
    // No commitments for the values, as they're plaintext
    // until flow encryption is available
    // pub asset_1_commitment: value::Commitment,
    // pub asset_2_commitment: value::Commitment,
    pub delta_1: u64,
    pub delta_2: u64,
    pub fee_commitment: value::Commitment,
    pub fee: Fee,
    pub swap_nft: NotePayload,
    pub note_1: Note,
    pub note_1_position: tct::Position,
    pub note_1_randomizer: Fr,
    pub note_1_value_blinding: Fr,
    pub note_2: Note,
    pub note_2_position: tct::Position,
    pub note_2_randomizer: Fr,
    pub note_2_value_blinding: Fr,
}

impl SwapPlan {
    /// Create a new [`SwapPlan`] that requests a swap between the given assets and input amounts.
    pub fn new<R: CryptoRng + RngCore>(
        rng: &mut R,
        fee_commitment: value::Commitment,
        fee: Fee,
        position: tct::Position,
    ) -> SwapPlan {
        SwapPlan {
            trading_pair: todo!(),
            delta_1: todo!(),
            delta_2: todo!(),
            fee_commitment,
            swap_nft: todo!(),
            fee,
        }
    }

    /// Convenience method to construct the [`Swap`] described by this [`SwapPlan`].
    pub fn swap(&self, fvk: &FullViewingKey, auth_path: tct::Proof) -> Swap {
        Swap {
            body: self.swap_body(fvk),
            proof: self.swap_proof(fvk, auth_path),
        }
    }

    /// Construct the [`swap::Body`] described by this [`SwapPlan`].
    pub fn swap_body(&self, fvk: &FullViewingKey) -> swap::Body {
        swap::Body {
            trading_pair: todo!(),
            delta_1: todo!(),
            delta_2: todo!(),
            fee_commitment: todo!(),
            swap_nft: todo!(),
            swap_ciphertext: todo!(),
        }
    }

    /// Construct the [`SwapProof`] required by the [`swap::Body`] described by this [`SwapPlan`].
    pub fn swap_proof(&self, fvk: &FullViewingKey, note_commitment_proof: tct::Proof) -> SwapProof {
        SwapProof {
            b_d: self.note.diversified_generator(),
            pk_d: self.note.transmission_key(),
            note_blinding: self.note.note_blinding(),
            fee_delta: self.fee_commitment.value(),
            value_t1: self.delta_1,
            value_t2: self.delta_2,
            swap_nft_asset_id: self.swap_nft.asset_id(),
            esk: fvk.esk(),
            delta_1_blinding: self.delta_1_blinding(),
            delta_2_blinding: self.delta_2_blinding(),
        }
    }
}

impl Protobuf<pb::SwapPlan> for SwapPlan {}

impl From<SwapPlan> for pb::SwapPlan {
    fn from(msg: SwapPlan) -> Self {
        Self {
            note: Some(msg.note.into()),
            position: u64::from(msg.position),
            randomizer: msg.randomizer.to_bytes().to_vec().into(),
            value_blinding: msg.value_blinding.to_bytes().to_vec().into(),
        }
    }
}

impl TryFrom<pb::SwapPlan> for SwapPlan {
    type Error = anyhow::Error;
    fn try_from(msg: pb::SwapPlan) -> Result<Self, Self::Error> {
        Ok(Self {
            trading_pair: msg.trading_pair.try_into()?,
            delta_1: msg.delta_1,
            delta_2: msg.delta_2,
            fee_commitment: msg.fee_commitment.try_into()?,
            swap_nft: msg.swap_nft.try_into()?,
        })
    }
}
