use anyhow::{anyhow, Result};
use ark_ff::UniformRand;
use decaf377::Fq;
use penumbra_crypto::dex::swap::SwapPlaintext;
use penumbra_crypto::{
    proofs::transparent::SwapProof, FieldExt, Fr, FullViewingKey, Note, NotePayload, Value,
};
use penumbra_proto::{transaction as pb, Protobuf};
use rand_core::{CryptoRng, RngCore};
use serde::{Deserialize, Serialize};

use crate::action::{swap, Swap};

/// A planned [`Swap`](Swap).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(try_from = "pb::SwapPlan", into = "pb::SwapPlan")]
pub struct SwapPlan {
    // No commitments for the values, as they're plaintext
    // until flow encryption is available
    // pub asset_1_commitment: value::Commitment,
    // pub asset_2_commitment: value::Commitment,
    pub swap_plaintext: SwapPlaintext,
    pub fee_blinding: Fr,
    pub note_blinding: Fq,
    pub esk: decaf377_ka::Secret,
}

impl SwapPlan {
    /// Create a new [`SwapPlan`] that requests a swap between the given assets and input amounts.
    pub fn new<R: CryptoRng + RngCore>(rng: &mut R, swap_plaintext: SwapPlaintext) -> SwapPlan {
        let note_blinding = Fq::rand(rng);
        let fee_blinding = Fr::rand(rng);
        let esk = decaf377_ka::Secret::new(rng);
        SwapPlan {
            fee_blinding,
            note_blinding,
            esk,
            swap_plaintext,
        }
    }

    /// Convenience method to construct the [`Swap`] described by this [`SwapPlan`].
    pub fn swap(&self, fvk: &FullViewingKey) -> Swap {
        Swap {
            body: self.swap_body(fvk),
            proof: self.swap_proof(),
        }
    }

    /// Construct the [`swap::Body`] described by this [`SwapPlan`].
    pub fn swap_body(&self, _fvk: &FullViewingKey) -> swap::Body {
        let fee_commitment = self
            .swap_plaintext
            .claim_fee
            .value()
            .commit(self.fee_blinding);

        let swap_nft_asset_id = self.swap_plaintext.asset_id();

        let swap_nft_value = Value {
            amount: 1,
            asset_id: swap_nft_asset_id,
        };

        let swap_nft_note = Note::from_parts(
            self.swap_plaintext.claim_address,
            swap_nft_value,
            self.note_blinding,
        )
        .expect("unable to create swap nft note");
        let note_commitment = swap_nft_note.commit();

        let encrypted_note = swap_nft_note.encrypt(&self.esk);
        let diversified_generator = swap_nft_note.diversified_generator();
        let swap_nft = NotePayload {
            note_commitment,
            ephemeral_key: self.esk.diversified_public(&diversified_generator),
            encrypted_note,
        };

        let swap_ciphertext = self.swap_plaintext.encrypt(&self.esk);

        swap::Body {
            trading_pair: self.swap_plaintext.trading_pair,
            delta_1: self.swap_plaintext.delta_1,
            delta_2: self.swap_plaintext.delta_2,
            fee_commitment,
            swap_nft,
            swap_ciphertext,
        }
    }

    /// Construct the [`SwapProof`] required by the [`swap::Body`] described by this [`SwapPlan`].
    pub fn swap_proof(&self) -> SwapProof {
        let swap_nft_asset_id = self.swap_plaintext.asset_id();

        SwapProof {
            claim_address: self.swap_plaintext.claim_address,
            note_blinding: self.note_blinding,
            fee_delta: self.swap_plaintext.claim_fee.clone(),
            fee_blinding: self.fee_blinding,
            value_t1: Value {
                amount: self.swap_plaintext.delta_1,
                asset_id: self.swap_plaintext.trading_pair.asset_1(),
            },
            value_t2: Value {
                amount: self.swap_plaintext.delta_2,
                asset_id: self.swap_plaintext.trading_pair.asset_2(),
            },
            swap_nft_asset_id,
            esk: self.esk.clone(),
            // TODO: no blinding factors for deltas yet, they're plaintext
            // until flow encryption is available
            // delta_1_blinding: self.delta_1_blinding(),
            // delta_2_blinding: self.delta_2_blinding(),
        }
    }
}

impl Protobuf<pb::SwapPlan> for SwapPlan {}

impl From<SwapPlan> for pb::SwapPlan {
    fn from(msg: SwapPlan) -> Self {
        Self {
            swap_plaintext: Some(msg.swap_plaintext.into()),
            fee_blinding: msg.fee_blinding.to_bytes().to_vec().into(),
            note_blinding: msg.note_blinding.to_bytes().to_vec().into(),
            esk: msg.esk.to_bytes().to_vec().into(),
        }
    }
}

impl TryFrom<pb::SwapPlan> for SwapPlan {
    type Error = anyhow::Error;
    fn try_from(msg: pb::SwapPlan) -> Result<Self, Self::Error> {
        let fee_blinding_bytes: [u8; 32] = msg.fee_blinding[..]
            .try_into()
            .map_err(|_| anyhow!("proto malformed"))?;
        Ok(Self {
            fee_blinding: Fr::from_bytes(fee_blinding_bytes)
                .map_err(|_| anyhow!("proto malformed"))?,
            swap_plaintext: msg
                .swap_plaintext
                .ok_or_else(|| anyhow!("missing swap_plaintext"))?
                .try_into()?,
            note_blinding: Fq::from_bytes(msg.note_blinding[..].try_into()?)?,
            esk: msg.esk.as_ref().try_into()?,
        })
    }
}
