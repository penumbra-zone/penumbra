use ark_ff::UniformRand;
use decaf377_rdsa::{Signature, SpendAuth};
use penumbra_crypto::{
    merkle, merkle::AuthPath, proofs::transparent::SpendProof, FieldExt, Fr, FullViewingKey, Note,
};
use penumbra_proto::{transaction as pb, Protobuf};
use rand_core::{CryptoRng, RngCore};
use serde::{Deserialize, Serialize};

use crate::action::{spend, Spend};

/// A planned [`Spend`](Spend).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(try_from = "pb::SpendPlan", into = "pb::SpendPlan")]
pub struct SpendPlan {
    pub note: Note,
    pub position: merkle::Position,
    pub randomizer: Fr,
    pub value_blinding: Fr,
}

impl SpendPlan {
    /// Create a new [`SpendPlan`] that spends the given `position`ed `note`.
    pub fn new<R: CryptoRng + RngCore>(
        rng: &mut R,
        note: Note,
        position: merkle::Position,
    ) -> SpendPlan {
        SpendPlan {
            note,
            position,
            randomizer: Fr::rand(rng),
            value_blinding: Fr::rand(rng),
        }
    }

    /// Convenience method to construct the [`Spend`] described by this [`SpendPlan`].
    pub fn spend(
        &self,
        fvk: &FullViewingKey,
        auth_sig: Signature<SpendAuth>,
        auth_path: AuthPath,
    ) -> Spend {
        Spend {
            body: self.spend_body(fvk),
            auth_sig,
            proof: self.spend_proof(fvk, auth_path),
        }
    }

    /// Construct the [`spend::Body`] described by this [`SpendPlan`].
    pub fn spend_body(&self, fvk: &FullViewingKey) -> spend::Body {
        spend::Body {
            value_commitment: self.note.value().commit(self.value_blinding),
            nullifier: fvk.derive_nullifier(self.position, &self.note.commit()),
            rk: fvk.spend_verification_key().randomize(&self.randomizer),
        }
    }

    /// Construct the [`SpendProof`] required by the [`spend::Body`] described by this [`SpendPlan`].
    pub fn spend_proof(&self, fvk: &FullViewingKey, auth_path: AuthPath) -> SpendProof {
        // XXX: the position field duplicates data from the merkle path
        // probably not worth fixing before we just make them snarks...
        // ... just patch up types and aim to replace by TCT
        let position = auth_path.position.clone();
        let merkle_path = (auth_path.position, auth_path.path);
        SpendProof {
            position,
            merkle_path,
            g_d: self.note.diversified_generator(),
            pk_d: self.note.transmission_key(),
            value: self.note.value(),
            v_blinding: self.value_blinding,
            note_commitment: self.note.commit(),
            note_blinding: self.note.note_blinding(),
            spend_auth_randomizer: self.randomizer,
            ak: *fvk.spend_verification_key(),
            nk: *fvk.nullifier_key(),
        }
    }
}

impl Protobuf<pb::SpendPlan> for SpendPlan {}

impl From<SpendPlan> for pb::SpendPlan {
    fn from(msg: SpendPlan) -> Self {
        Self {
            note: Some(msg.note.into()),
            // TODO replace platform-dep code with TCT
            position: u64::from(msg.position) as u32,
            randomizer: msg.randomizer.to_bytes().to_vec().into(),
            value_blinding: msg.value_blinding.to_bytes().to_vec().into(),
        }
    }
}

impl TryFrom<pb::SpendPlan> for SpendPlan {
    type Error = anyhow::Error;
    fn try_from(msg: pb::SpendPlan) -> Result<Self, Self::Error> {
        Ok(Self {
            note: msg
                .note
                .ok_or_else(|| anyhow::anyhow!("missing note"))?
                .try_into()?,
            position: (msg.position as usize).into(),
            randomizer: Fr::from_bytes(msg.randomizer.as_ref().try_into()?)?,
            value_blinding: Fr::from_bytes(msg.value_blinding.as_ref().try_into()?)?,
        })
    }
}
