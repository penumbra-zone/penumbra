use ark_ff::UniformRand;
use decaf377_rdsa::{Signature, SpendAuth};
use penumbra_crypto::{
    proofs::transparent::SpendProof, Address, FieldExt, Fr, FullViewingKey, Note, Rseed, Value,
    STAKING_TOKEN_ASSET_ID,
};
use penumbra_proto::{core::transaction::v1alpha1 as pb, DomainType};
use penumbra_tct as tct;
use rand_core::{CryptoRng, RngCore};
use serde::{Deserialize, Serialize};

use crate::action::{spend, Spend};

/// A planned [`Spend`](Spend).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(try_from = "pb::SpendPlan", into = "pb::SpendPlan")]
pub struct SpendPlan {
    pub note: Note,
    pub position: tct::Position,
    pub randomizer: Fr,
    pub value_blinding: Fr,
}

impl SpendPlan {
    /// Create a new [`SpendPlan`] that spends the given `position`ed `note`.
    pub fn new<R: CryptoRng + RngCore>(
        rng: &mut R,
        note: Note,
        position: tct::Position,
    ) -> SpendPlan {
        SpendPlan {
            note,
            position,
            randomizer: Fr::rand(rng),
            value_blinding: Fr::rand(rng),
        }
    }

    /// Create a dummy [`SpendPlan`].
    pub fn dummy<R: CryptoRng + RngCore>(rng: &mut R) -> SpendPlan {
        let dummy_address = Address::dummy(rng);
        let rseed = Rseed::generate(rng);
        let dummy_note = Note::from_parts(
            dummy_address,
            Value {
                amount: 0u64.into(),
                asset_id: *STAKING_TOKEN_ASSET_ID,
            },
            rseed,
        )
        .expect("dummy note is valid");

        Self::new(rng, dummy_note, 0u64.into())
    }

    /// Convenience method to construct the [`Spend`] described by this [`SpendPlan`].
    pub fn spend(
        &self,
        fvk: &FullViewingKey,
        auth_sig: Signature<SpendAuth>,
        auth_path: tct::Proof,
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
            balance_commitment: self.balance().commit(self.value_blinding),
            nullifier: fvk.derive_nullifier(self.position, &self.note.commit()),
            rk: fvk.spend_verification_key().randomize(&self.randomizer),
        }
    }

    /// Construct the [`SpendProof`] required by the [`spend::Body`] described by this [`SpendPlan`].
    pub fn spend_proof(
        &self,
        fvk: &FullViewingKey,
        state_commitment_proof: tct::Proof,
    ) -> SpendProof {
        SpendProof {
            state_commitment_proof,
            note: self.note.clone(),
            v_blinding: self.value_blinding,
            spend_auth_randomizer: self.randomizer,
            ak: *fvk.spend_verification_key(),
            nk: *fvk.nullifier_key(),
        }
    }

    pub fn balance(&self) -> penumbra_crypto::Balance {
        penumbra_crypto::Value {
            amount: self.note.value().amount,
            asset_id: self.note.value().asset_id,
        }
        .into()
    }
}

impl DomainType for SpendPlan {
    type Proto = pb::SpendPlan;
}

impl From<SpendPlan> for pb::SpendPlan {
    fn from(msg: SpendPlan) -> Self {
        Self {
            note: Some(msg.note.into()),
            position: u64::from(msg.position),
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
            position: msg.position.into(),
            randomizer: Fr::from_bytes(msg.randomizer.as_ref().try_into()?)?,
            value_blinding: Fr::from_bytes(msg.value_blinding.as_ref().try_into()?)?,
        })
    }
}
