use anyhow::{anyhow, Result};
use ark_ff::UniformRand;
use decaf377::Fq;
use decaf377_rdsa::{Signature, SpendAuth};
use penumbra_crypto::dex::swap::{generate_swap_asset_id, SwapPlaintext};
use penumbra_crypto::Address;
use penumbra_crypto::{
    dex::TradingPair,
    proofs::transparent::{SpendProof, SwapProof},
    transaction::Fee,
    value, FieldExt, Fr, FullViewingKey, Note, NotePayload, Value,
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
    pub fee: Fee,
    pub fee_blinding: Fr,
    pub claim_address: Address,
    pub note_blinding: Fq,
    pub esk: decaf377_ka::Secret,
}

impl SwapPlan {
    /// Create a new [`SwapPlan`] that requests a swap between the given assets and input amounts.
    pub fn new<R: CryptoRng + RngCore>(
        rng: &mut R,
        trading_pair: TradingPair,
        delta_1: u64,
        delta_2: u64,
        fee: Fee,
        claim_address: Address,
    ) -> SwapPlan {
        let note_blinding = Fq::rand(rng);
        let fee_blinding = Fr::rand(rng);
        let esk = decaf377_ka::Secret::new(rng);
        SwapPlan {
            trading_pair,
            delta_1,
            delta_2,
            fee_blinding,
            fee,
            claim_address,
            note_blinding,
            esk,
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
        let fee_commitment = self.fee.value().commit(self.fee_blinding);

        let swap_nft_asset_id = generate_swap_asset_id(
            self.delta_1,
            self.delta_2,
            self.fee.0,
            *self.claim_address.diversified_generator(),
            *self.claim_address.transmission_key(),
            self.trading_pair,
        )
        .expect("bad public key when generating swap asset id");

        let swap_nft_value = Value {
            amount: 1,
            asset_id: swap_nft_asset_id,
        };

        let swap_nft_note = Note::from_parts(
            self.claim_address.diversifier().clone(),
            self.claim_address.transmission_key().clone(),
            swap_nft_value,
            self.note_blinding,
        )
        .expect("unable to create swap nft note");
        let note_commitment = swap_nft_note.commit();

        let encrypted_note = swap_nft_note.encrypt(&self.esk);
        let swap_nft = NotePayload {
            note_commitment,
            ephemeral_key: self.esk.public(),
            encrypted_note,
        };

        let swap_ciphertext = SwapPlaintext::from_parts(
            self.trading_pair,
            self.delta_1,
            self.delta_2,
            self.fee.clone(),
            self.claim_address,
        )
        .expect("unable to create swap plaintext")
        .encrypt(&self.esk);

        swap::Body {
            trading_pair: self.trading_pair,
            delta_1: self.delta_1,
            delta_2: self.delta_2,
            fee_commitment,
            swap_nft,
            swap_ciphertext,
        }
    }

    /// Construct the [`SwapProof`] required by the [`swap::Body`] described by this [`SwapPlan`].
    pub fn swap_proof(&self, fvk: &FullViewingKey, note_commitment_proof: tct::Proof) -> SwapProof {
        let swap_nft_asset_id = generate_swap_asset_id(
            self.delta_1,
            self.delta_2,
            self.fee.0,
            *self.claim_address.diversified_generator(),
            *self.claim_address.transmission_key(),
            self.trading_pair,
        )
        .expect("bad public key when generating swap asset id");

        SwapProof {
            b_d: self.claim_address.diversified_generator().clone(),
            pk_d: self.claim_address.transmission_key().clone(),
            note_blinding: self.note_blinding,
            fee_delta: self.fee.0,
            value_t1: Value {
                amount: self.delta_1,
                asset_id: self.trading_pair.asset_1(),
            },
            value_t2: Value {
                amount: self.delta_2,
                asset_id: self.trading_pair.asset_2(),
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
            trading_pair: Some(msg.trading_pair.into()),
            delta_1: msg.delta_1,
            delta_2: msg.delta_2,
            fee: Some(msg.fee.into()),
            fee_blinding: msg.fee_blinding.to_bytes().to_vec().into(),
            claim_address: Some(msg.claim_address.into()),
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
            trading_pair: msg
                .trading_pair
                .ok_or_else(|| anyhow!("missing trading pair"))?
                .try_into()?,
            delta_1: msg.delta_1,
            delta_2: msg.delta_2,
            fee: msg.fee.ok_or_else(|| anyhow!("missing fee"))?.try_into()?,
            fee_blinding: Fr::from_bytes(fee_blinding_bytes)
                .map_err(|_| anyhow!("proto malformed"))?,
            claim_address: msg
                .claim_address
                .ok_or_else(|| anyhow!("missing claim address"))?
                .try_into()?,
            note_blinding: Fq::from_bytes(msg.note_blinding[..].try_into()?)?,
            esk: msg.esk.as_ref().try_into()?,
        })
    }
}
