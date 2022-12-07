use penumbra_crypto::{
    dex::{swap::SwapPlaintext, BatchSwapOutputData},
    ka,
    keys::{IncomingViewingKey, NullifierKey},
    proofs::transparent::SwapClaimProof,
    EncryptedNote, FullViewingKey, Note, Value,
};
use penumbra_proto::{core::transaction::v1alpha1 as pb, Protobuf};
use penumbra_tct as tct;
use rand_core::{CryptoRng, RngCore};
use serde::{Deserialize, Serialize};
use tct::Position;

use crate::action::{swap_claim, SwapClaim};

/// A planned [`SwapClaim`](SwapClaim).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(try_from = "pb::SwapClaimPlan", into = "pb::SwapClaimPlan")]
pub struct SwapClaimPlan {
    pub swap_nft_note: Note,
    pub swap_nft_position: Position,
    pub swap_plaintext: SwapPlaintext,
    pub output_data: BatchSwapOutputData,
    pub esk_1: ka::Secret,
    pub esk_2: ka::Secret,
    pub epoch_duration: u64,
}

impl SwapClaimPlan {
    /// Create a new [`SwapClaimPlan`] that redeems output notes to `claim_address` using
    /// the associated swap NFT.
    #[allow(clippy::too_many_arguments)]
    pub fn new<R: RngCore + CryptoRng>(
        rng: &mut R,
        swap_plaintext: SwapPlaintext,
        swap_nft_note: Note,
        swap_nft_position: Position,
        epoch_duration: u64,
        output_data: BatchSwapOutputData,
    ) -> SwapClaimPlan {
        let esk_1 = ka::Secret::new(rng);
        let esk_2 = ka::Secret::new(rng);

        Self {
            swap_nft_note,
            esk_1,
            esk_2,
            output_data,
            swap_plaintext,
            swap_nft_position,
            epoch_duration,
        }
    }

    /// Convenience method to construct the [`SwapClaim`] described by this
    /// [`SwapClaimPlan`].
    pub fn swap_claim(
        &self,
        fvk: &FullViewingKey,
        note_commitment_proof: &tct::Proof,
    ) -> SwapClaim {
        SwapClaim {
            body: self.swap_claim_body(fvk),
            proof: self.swap_claim_proof(note_commitment_proof, fvk.nullifier_key()),
        }
    }

    /// Construct the [`SwapClaimProof`] required by the [`swap_claim::Body`] described
    /// by this plan.
    pub fn swap_claim_proof(
        &self,
        note_commitment_proof: &tct::Proof,
        nk: &NullifierKey,
    ) -> SwapClaimProof {
        let (lambda_1_i, lambda_2_i) = self.output_data.pro_rata_outputs((
            self.swap_plaintext.delta_1_i.into(),
            self.swap_plaintext.delta_2_i.into(),
        ));

        SwapClaimProof {
            claim_address: self.swap_nft_note.address(),
            note_commitment_proof: note_commitment_proof.clone(),
            trading_pair: self.swap_plaintext.trading_pair,
            note_blinding: self.swap_nft_note.note_blinding(),
            delta_1_i: self.swap_plaintext.delta_1_i.into(),
            delta_2_i: self.swap_plaintext.delta_2_i.into(),
            lambda_1_i,
            lambda_2_i,
            esk_1: self.esk_1.clone(),
            esk_2: self.esk_2.clone(),
            nk: nk.clone(),
            swap_blinding: self.swap_plaintext.swap_blinding,
        }
    }

    /// Construct the [`swap_claim::Body`] described by this plan.
    pub fn swap_claim_body(&self, fvk: &FullViewingKey) -> swap_claim::Body {
        let (lambda_1_i, lambda_2_i) = self.output_data.pro_rata_outputs((
            self.swap_plaintext.delta_1_i.into(),
            self.swap_plaintext.delta_2_i.into(),
        ));

        let (output_blinding_1, output_blinding_2) = self.swap_plaintext.output_blinding_factors();

        let output_1_note = Note::from_parts(
            self.swap_nft_note.address(),
            Value {
                amount: lambda_1_i.into(),
                asset_id: self.swap_plaintext.trading_pair.asset_1(),
            },
            output_blinding_1,
        )
        .expect("transmission key in address is always valid");
        let output_2_note = Note::from_parts(
            self.swap_nft_note.address(),
            Value {
                amount: lambda_2_i.into(),
                asset_id: self.swap_plaintext.trading_pair.asset_2(),
            },
            output_blinding_2,
        )
        .expect("transmission key in address is always valid");
        tracing::debug!(?output_1_note, ?output_2_note);

        // We need to get the correct diversified generator to use with DH:
        let g_d = self.swap_plaintext.claim_address.diversified_generator();
        let output_1 = EncryptedNote {
            note_commitment: output_1_note.commit(),
            ephemeral_key: self.esk_1.diversified_public(g_d),
            encrypted_note: output_1_note.encrypt(&self.esk_1),
        };
        let output_2 = EncryptedNote {
            note_commitment: output_2_note.commit(),
            ephemeral_key: self.esk_2.diversified_public(g_d),
            encrypted_note: output_2_note.encrypt(&self.esk_2),
        };

        let nullifier = fvk.derive_nullifier(self.swap_nft_position, &self.swap_nft_note.commit());

        swap_claim::Body {
            nullifier,
            fee: self.swap_plaintext.claim_fee.clone(),
            output_1,
            output_2,
            output_data: self.output_data,
            epoch_duration: self.epoch_duration,
        }
    }

    /// Checks whether this plan's output is viewed by the given IVK.
    pub fn is_viewed_by(&self, ivk: &IncomingViewingKey) -> bool {
        ivk.views_address(&self.swap_nft_note.address())
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

impl Protobuf<pb::SwapClaimPlan> for SwapClaimPlan {}

impl From<SwapClaimPlan> for pb::SwapClaimPlan {
    fn from(msg: SwapClaimPlan) -> Self {
        Self {
            swap_plaintext: Some(msg.swap_plaintext.into()),
            swap_nft_note: Some(msg.swap_nft_note.into()),
            swap_nft_position: msg.swap_nft_position.into(),
            output_data: Some(msg.output_data.into()),
            esk_1: msg.esk_1.to_bytes().to_vec().into(),
            esk_2: msg.esk_2.to_bytes().to_vec().into(),
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
            swap_nft_note: msg
                .swap_nft_note
                .ok_or_else(|| anyhow::anyhow!("missing swap_nft_note"))?
                .try_into()?,
            swap_nft_position: msg.swap_nft_position.try_into()?,
            output_data: msg
                .output_data
                .ok_or_else(|| anyhow::anyhow!("missing output_data"))?
                .try_into()?,
            esk_1: msg.esk_1.as_ref().try_into()?,
            esk_2: msg.esk_2.as_ref().try_into()?,
            epoch_duration: msg.epoch_duration,
        })
    }
}
