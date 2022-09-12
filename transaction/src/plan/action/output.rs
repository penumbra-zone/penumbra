use ark_ff::UniformRand;
use penumbra_crypto::{
    ka,
    keys::{IncomingViewingKey, OutgoingViewingKey},
    proofs::transparent::OutputProof,
    symmetric::WrappedMemoKey,
    Address, FieldExt, Fq, Fr, Note, NotePayload, PayloadKey, Value, STAKING_TOKEN_ASSET_ID,
};
use penumbra_proto::{transaction as pb, Protobuf};
use rand_core::{CryptoRng, RngCore};
use serde::{Deserialize, Serialize};

use crate::action::{output, Output};

/// A planned [`Output`](Output).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(try_from = "pb::OutputPlan", into = "pb::OutputPlan")]
pub struct OutputPlan {
    pub value: Value,
    pub dest_address: Address,
    pub note_blinding: Fq,
    pub value_blinding: Fr,
    pub esk: ka::Secret,
}

impl OutputPlan {
    /// Create a new [`OutputPlan`] that sends `value` to `dest_address`.
    pub fn new<R: RngCore + CryptoRng>(
        rng: &mut R,
        value: Value,
        dest_address: Address,
    ) -> OutputPlan {
        let note_blinding = Fq::rand(rng);
        let value_blinding = Fr::rand(rng);
        let esk = ka::Secret::new(rng);
        Self {
            value,
            dest_address,
            note_blinding,
            value_blinding,
            esk,
        }
    }

    /// Create a dummy [`OutputPlan`].
    pub fn dummy<R: CryptoRng + RngCore>(rng: &mut R) -> OutputPlan {
        let dummy_address = Address::dummy(rng);
        Self::new(
            rng,
            Value {
                amount: 0,
                asset_id: *STAKING_TOKEN_ASSET_ID,
            },
            dummy_address,
        )
    }

    /// Convenience method to construct the [`Output`] described by this
    /// [`OutputPlan`].
    pub fn output(&self, ovk: &OutgoingViewingKey, memo_key: &PayloadKey) -> Output {
        Output {
            body: self.output_body(ovk, memo_key),
            proof: self.output_proof(),
        }
    }

    pub fn output_note(&self) -> Note {
        Note::from_parts(self.dest_address, self.value, self.note_blinding)
            .expect("transmission key in address is always valid")
    }

    /// Construct the [`OutputProof`] required by the [`output::Body`] described
    /// by this plan.
    pub fn output_proof(&self) -> OutputProof {
        OutputProof {
            g_d: self.output_note().diversified_generator(),
            pk_d: self.dest_address.transmission_key().clone(),
            ck_d: self.dest_address.clue_key().clone(),
            value: self.value,
            v_blinding: self.value_blinding,
            note_blinding: self.note_blinding,
            esk: self.esk.clone(),
        }
    }

    /// Construct the [`output::Body`] described by this plan.
    pub fn output_body(&self, ovk: &OutgoingViewingKey, memo_key: &PayloadKey) -> output::Body {
        // Prepare the output note and commitment.
        let note = self.output_note();
        let note_commitment = note.commit();

        // Prepare the value commitment.  Outputs subtract from the transaction
        // value balance, so flip the sign of the commitment.
        let value_commitment = -self.value.commit(self.value_blinding);

        // Encrypt the note to the recipient...
        let diversified_generator = note.diversified_generator();
        let ephemeral_key = self.esk.diversified_public(&diversified_generator);
        let encrypted_note = note.encrypt(&self.esk);
        // ... and wrap the encryption key to ourselves.
        let ovk_wrapped_key = note.encrypt_key(&self.esk, ovk, value_commitment);

        let wrapped_memo_key = WrappedMemoKey::encrypt(
            &memo_key,
            self.esk.clone(),
            note.transmission_key(),
            &note.diversified_generator(),
        );

        output::Body {
            note_payload: NotePayload {
                note_commitment,
                ephemeral_key,
                encrypted_note,
            },
            value_commitment,
            ovk_wrapped_key,
            wrapped_memo_key,
        }
    }

    /// Checks whether this plan's output is viewed by the given IVK.
    pub fn is_viewed_by(&self, ivk: &IncomingViewingKey) -> bool {
        ivk.views_address(&self.dest_address)
    }
}

impl Protobuf<pb::OutputPlan> for OutputPlan {}

impl From<OutputPlan> for pb::OutputPlan {
    fn from(msg: OutputPlan) -> Self {
        Self {
            value: Some(msg.value.into()),
            dest_address: Some(msg.dest_address.into()),
            note_blinding: msg.note_blinding.to_bytes().to_vec().into(),
            value_blinding: msg.value_blinding.to_bytes().to_vec().into(),
            esk: msg.esk.to_bytes().to_vec().into(),
        }
    }
}

impl TryFrom<pb::OutputPlan> for OutputPlan {
    type Error = anyhow::Error;
    fn try_from(msg: pb::OutputPlan) -> Result<Self, Self::Error> {
        Ok(Self {
            value: msg
                .value
                .ok_or_else(|| anyhow::anyhow!("missing value"))?
                .try_into()?,
            dest_address: msg
                .dest_address
                .ok_or_else(|| anyhow::anyhow!("missing address"))?
                .try_into()?,
            note_blinding: Fq::from_bytes(msg.note_blinding.as_ref().try_into()?)?,
            value_blinding: Fr::from_bytes(msg.value_blinding.as_ref().try_into()?)?,
            esk: msg.esk.as_ref().try_into()?,
        })
    }
}
