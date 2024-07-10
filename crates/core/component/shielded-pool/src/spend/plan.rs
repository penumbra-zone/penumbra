use decaf377::{Fq, Fr};
use decaf377_rdsa::{Signature, SpendAuth};
use penumbra_asset::{Balance, Value, STAKING_TOKEN_ASSET_ID};
use penumbra_keys::{keys::AddressIndex, FullViewingKey};
use penumbra_proto::{core::component::shielded_pool::v1 as pb, DomainType};
use penumbra_sct::Nullifier;
use penumbra_tct as tct;
use rand_core::{CryptoRng, RngCore};
use serde::{Deserialize, Serialize};

use super::{Body, Spend, SpendProof};
use crate::{Note, Rseed, SpendProofPrivate, SpendProofPublic};

/// A planned [`Spend`](Spend).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(try_from = "pb::SpendPlan", into = "pb::SpendPlan")]
pub struct SpendPlan {
    pub note: Note,
    pub position: tct::Position,
    pub randomizer: Fr,
    pub value_blinding: Fr,
    pub proof_blinding_r: Fq,
    pub proof_blinding_s: Fq,
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
            proof_blinding_r: Fq::rand(rng),
            proof_blinding_s: Fq::rand(rng),
        }
    }

    /// Create a dummy [`SpendPlan`].
    pub fn dummy<R: CryptoRng + RngCore>(rng: &mut R, fvk: &FullViewingKey) -> SpendPlan {
        // A valid address we can spend; since the note is hidden, we can just pick the default.
        let dummy_address = fvk.payment_address(AddressIndex::default()).0;
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
        anchor: tct::Root,
    ) -> Spend {
        Spend {
            body: self.spend_body(fvk),
            auth_sig,
            proof: self.spend_proof(fvk, auth_path, anchor),
        }
    }

    /// Construct the [`spend::Body`] described by this [`SpendPlan`].
    pub fn spend_body(&self, fvk: &FullViewingKey) -> Body {
        Body {
            balance_commitment: self.balance().commit(self.value_blinding),
            nullifier: self.nullifier(fvk),
            rk: self.rk(fvk),
        }
    }

    /// Construct the randomized verification key associated with this [`SpendPlan`].
    pub fn rk(&self, fvk: &FullViewingKey) -> decaf377_rdsa::VerificationKey<SpendAuth> {
        fvk.spend_verification_key().randomize(&self.randomizer)
    }

    /// Construct the [`Nullifier`] associated with this [`SpendPlan`].
    pub fn nullifier(&self, fvk: &FullViewingKey) -> Nullifier {
        let nk = fvk.nullifier_key();
        Nullifier::derive(nk, self.position, &self.note.commit())
    }

    /// Construct the [`SpendProof`] required by the [`spend::Body`] described by this [`SpendPlan`].
    pub fn spend_proof(
        &self,
        fvk: &FullViewingKey,
        state_commitment_proof: tct::Proof,
        anchor: tct::Root,
    ) -> SpendProof {
        let public = SpendProofPublic {
            anchor,
            balance_commitment: self.balance().commit(self.value_blinding),
            nullifier: self.nullifier(fvk),
            rk: self.rk(fvk),
        };
        let private = SpendProofPrivate {
            state_commitment_proof,
            note: self.note.clone(),
            v_blinding: self.value_blinding,
            spend_auth_randomizer: self.randomizer,
            ak: *fvk.spend_verification_key(),
            nk: *fvk.nullifier_key(),
        };
        SpendProof::prove(
            self.proof_blinding_r,
            self.proof_blinding_s,
            &penumbra_proof_params::SPEND_PROOF_PROVING_KEY,
            public,
            private,
        )
        .expect("can generate ZKSpendProof")
    }

    pub fn balance(&self) -> Balance {
        Value {
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
            randomizer: msg.randomizer.to_bytes().to_vec(),
            value_blinding: msg.value_blinding.to_bytes().to_vec(),
            proof_blinding_r: msg.proof_blinding_r.to_bytes().to_vec(),
            proof_blinding_s: msg.proof_blinding_s.to_bytes().to_vec(),
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
            randomizer: Fr::from_bytes_checked(msg.randomizer.as_slice().try_into()?)
                .expect("randomizer malformed"),
            value_blinding: Fr::from_bytes_checked(msg.value_blinding.as_slice().try_into()?)
                .expect("value_blinding malformed"),
            proof_blinding_r: Fq::from_bytes_checked(msg.proof_blinding_r.as_slice().try_into()?)
                .expect("proof_blinding_r malformed"),
            proof_blinding_s: Fq::from_bytes_checked(msg.proof_blinding_s.as_slice().try_into()?)
                .expect("proof_blinding_s malformed"),
        })
    }
}
