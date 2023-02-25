use std::str::FromStr;

use ark_r1cs_std::uint8::UInt8;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use decaf377::FieldExt;
use decaf377::{Bls12_377, Fq, Fr};
use decaf377_fmd as fmd;
use decaf377_ka as ka;

use ark_ff::ToConstraintField;
use ark_groth16::{Groth16, Proof, ProvingKey, VerifyingKey};
use ark_r1cs_std::prelude::*;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef};
use ark_snark::SNARK;
use penumbra_proto::{core::crypto::v1alpha1 as pb, DomainType};
use rand::{CryptoRng, Rng};
use rand_core::OsRng;

use crate::balance::BalanceVar;
use crate::proofs::groth16::{gadgets, ParameterSetup, GROTH16_PROOF_LENGTH_BYTES};
use crate::{
    balance, balance::commitment::BalanceCommitmentVar, keys::Diversifier, note, Address, Note,
    Rseed, Value,
};

use super::VerifyingKeyExt;

// Public:
// * vcm (value commitment)
// * ncm (note commitment)
//
// Witnesses:
// * g_d (point)
// * pk_d (point)
// * v (u64 value plus asset ID (scalar))
// * vblind (Fr)
// * nblind (Fq)
#[derive(Clone, Debug)]
pub struct OutputCircuit {
    // Witnesses
    /// The note being created.
    note: Note,
    /// The blinding factor used for generating the balance commitment.
    v_blinding: Fr,

    // Public inputs
    /// balance commitment of the new note,
    pub balance_commitment: balance::Commitment,
    /// note commitment of the new note,
    pub note_commitment: note::Commitment,
}

impl ConstraintSynthesizer<Fq> for OutputCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fq>) -> ark_relations::r1cs::Result<()> {
        // Witnesses
        let note_var = note::NoteVar::new_witness(cs.clone(), || Ok(self.note.clone()))?;
        let v_blinding_arr: [u8; 32] = self.v_blinding.to_bytes();
        let v_blinding_vars = UInt8::new_witness_vec(cs.clone(), &v_blinding_arr)?;

        // Public inputs
        let claimed_note_commitment =
            note::NoteCommitmentVar::new_input(cs.clone(), || Ok(self.note_commitment))?;
        let claimed_balance_commitment =
            BalanceCommitmentVar::new_input(cs.clone(), || Ok(self.balance_commitment))?;

        gadgets::element_not_identity(
            cs.clone(),
            &Boolean::TRUE,
            note_var.diversified_generator(),
        )?;
        // Check integrity of balance commitment.
        let balance_commitment =
            BalanceVar::from_negative_value_var(note_var.value()).commit(v_blinding_vars)?;
        balance_commitment.enforce_equal(&claimed_balance_commitment)?;

        // Note commitment integrity
        let note_commitment = note_var.commit()?;
        note_commitment.enforce_equal(&claimed_note_commitment)?;

        Ok(())
    }
}

impl ParameterSetup for OutputCircuit {
    fn generate_test_parameters() -> (ProvingKey<Bls12_377>, VerifyingKey<Bls12_377>) {
        let diversifier_bytes = [1u8; 16];
        let pk_d_bytes = decaf377::basepoint().vartime_compress().0;
        let clue_key_bytes = [1; 32];
        let diversifier = Diversifier(diversifier_bytes);
        let address = Address::from_components(
            diversifier,
            ka::Public(pk_d_bytes),
            fmd::ClueKey(clue_key_bytes),
        )
        .expect("generated 1 address");
        let note = Note::from_parts(
            address,
            Value::from_str("1upenumbra").expect("valid value"),
            Rseed([1u8; 32]),
        )
        .expect("can make a note");
        let v_blinding = Fr::from(1);
        let circuit = OutputCircuit {
            note: note.clone(),
            note_commitment: note.commit(),
            v_blinding,
            balance_commitment: balance::Commitment(decaf377::basepoint()),
        };
        let (pk, vk) = Groth16::circuit_specific_setup(circuit, &mut OsRng)
            .expect("can perform circuit specific setup");
        (pk, vk)
    }
}

#[derive(Clone, Debug)]
pub struct OutputProof(Proof<Bls12_377>);

impl OutputProof {
    #![allow(clippy::too_many_arguments)]
    pub fn prove<R: CryptoRng + Rng>(
        rng: &mut R,
        pk: &ProvingKey<Bls12_377>,
        note: Note,
        v_blinding: Fr,
        balance_commitment: balance::Commitment,
        note_commitment: note::Commitment,
    ) -> anyhow::Result<Self> {
        let circuit = OutputCircuit {
            note,
            note_commitment,
            v_blinding,
            balance_commitment,
        };
        let proof = Groth16::prove(pk, circuit, rng).map_err(|err| anyhow::anyhow!(err))?;
        Ok(Self(proof))
    }

    /// Called to verify the proof using the provided public inputs.
    ///
    /// The public inputs are:
    /// * balance commitment of the new note,
    /// * note commitment of the new note,
    #[tracing::instrument(skip(self, vk), fields(self = ?base64::encode(&self.clone().encode_to_vec()), vk = ?vk.debug_id()))]
    pub fn verify(
        &self,
        vk: &VerifyingKey<Bls12_377>,
        balance_commitment: balance::Commitment,
        note_commitment: note::Commitment,
    ) -> anyhow::Result<()> {
        let processed_pvk = Groth16::process_vk(vk).map_err(|err| anyhow::anyhow!(err))?;
        let mut public_inputs = Vec::new();
        public_inputs.extend(note_commitment.0.to_field_elements().unwrap());
        public_inputs.extend(balance_commitment.0.to_field_elements().unwrap());

        tracing::debug!(?public_inputs);
        let proof_result =
            Groth16::verify_with_processed_vk(&processed_pvk, public_inputs.as_slice(), &self.0)
                .map_err(|err| anyhow::anyhow!(err))?;
        tracing::debug!(?proof_result);
        proof_result
            .then_some(())
            .ok_or_else(|| anyhow::anyhow!("proof did not verify"))
    }
}

impl DomainType for OutputProof {
    type Proto = pb::ZkOutputProof;
}

impl From<OutputProof> for pb::ZkOutputProof {
    fn from(proof: OutputProof) -> Self {
        let mut proof_bytes = [0u8; GROTH16_PROOF_LENGTH_BYTES];
        Proof::serialize(&proof.0, &mut proof_bytes[..]).expect("can serialize Proof");
        pb::ZkOutputProof {
            inner: proof_bytes.to_vec(),
        }
    }
}

impl TryFrom<pb::ZkOutputProof> for OutputProof {
    type Error = anyhow::Error;

    fn try_from(proto: pb::ZkOutputProof) -> Result<Self, Self::Error> {
        Ok(OutputProof(
            Proof::deserialize(&proto.inner[..]).map_err(|e| anyhow::anyhow!(e))?,
        ))
    }
}
