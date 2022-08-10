use ark_ff::UniformRand;
use decaf377_rdsa::{Signature, SpendAuth};
use penumbra_crypto::{proofs::transparent::SpendProof, FieldExt, Fr, FullViewingKey, Note};
use penumbra_proto::{transaction as pb, Protobuf};
use penumbra_tct as tct;
use rand_core::{CryptoRng, RngCore};
use serde::{Deserialize, Serialize};

use crate::action::{swap, Swap};

// TODO: copied directly from `SpendPlan` right now, needs to be updated
// for `Swap`
/// A planned [`Swap`](Swap).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(try_from = "pb::SwapPlan", into = "pb::SwapPlan")]
pub struct SwapPlan {
    pub note: Note,
    pub position: tct::Position,
    pub randomizer: Fr,
    pub value_blinding: Fr,
}

impl SwapPlan {
    /// Create a new [`SwapPlan`] that spends the given `position`ed `note`.
    pub fn new<R: CryptoRng + RngCore>(
        rng: &mut R,
        note: Note,
        position: tct::Position,
    ) -> SwapPlan {
        SwapPlan {
            note,
            position,
            randomizer: Fr::rand(rng),
            value_blinding: Fr::rand(rng),
        }
    }

    /// Convenience method to construct the [`Swap`] described by this [`SwapPlan`].
    pub fn swap(
        &self,
        fvk: &FullViewingKey,
        auth_sig: Signature<SpendAuth>,
        auth_path: tct::Proof,
    ) -> Swap {
        Swap {
            body: self.swap_body(fvk),
            auth_sig,
            proof: self.swap_proof(fvk, auth_path),
        }
    }

    /// Construct the [`swap::Body`] described by this [`SwapPlan`].
    pub fn swap_body(&self, fvk: &FullViewingKey) -> swap::Body {
        swap::Body {
            value_commitment: self.note.value().commit(self.value_blinding),
            nullifier: fvk.derive_nullifier(self.position, &self.note.commit()),
            rk: fvk.spend_verification_key().randomize(&self.randomizer),
        }
    }

    /// Construct the [`SwapProof`] required by the [`swap::Body`] described by this [`SwapPlan`].
    pub fn swap_proof(&self, fvk: &FullViewingKey, note_commitment_proof: tct::Proof) -> SwapProof {
        SwapProof {
            note_commitment_proof,
            g_d: self.note.diversified_generator(),
            pk_d: self.note.transmission_key(),
            value: self.note.value(),
            v_blinding: self.value_blinding,
            note_blinding: self.note.note_blinding(),
            spend_auth_randomizer: self.randomizer,
            ak: *fvk.spend_verification_key(),
            nk: *fvk.nullifier_key(),
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
            note: msg
                .note
                .ok_or_else(|| anyhow::anyhow!("missing note"))?
                .try_into()?,
            position: msg.position.into(),
            randomizer: Fr::from_bytes(msg.randomizer.as_ref().try_into()?)?,
            value_blinding: Fr::from_bytes(msg.value_blinding.as_ref().try_into()?)?,
        })
    }
}
