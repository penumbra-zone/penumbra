use std::str::FromStr;

use ark_groth16::r1cs_to_qap::LibsnarkReduction;
use ark_r1cs_std::uint8::UInt8;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use decaf377::r1cs::ElementVar;
use decaf377::FieldExt;
use decaf377::{Bls12_377, Fq, Fr};
use decaf377_fmd as fmd;
use decaf377_ka as ka;

use ark_ff::ToConstraintField;
use ark_groth16::{Groth16, PreparedVerifyingKey, Proof, ProvingKey, VerifyingKey};
use ark_r1cs_std::prelude::*;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef};
use ark_snark::SNARK;
use penumbra_keys::{keys::Diversifier, Address};
use penumbra_proto::{core::crypto::v1alpha1 as pb, DomainType, TypeUrl};
use penumbra_tct::r1cs::StateCommitmentVar;
use rand_core::OsRng;

use crate::{note, Note, Rseed};
use penumbra_asset::{
    balance,
    balance::{commitment::BalanceCommitmentVar, BalanceVar},
    Value,
};
use penumbra_proof_params::{ParameterSetup, VerifyingKeyExt, GROTH16_PROOF_LENGTH_BYTES};

/// Public:
/// * vcm (value commitment)
/// * ncm (note commitment)
///
/// Witnesses:
/// * g_d (point)
/// * pk_d (point)
/// * v (u64 value plus asset ID (scalar))
/// * vblind (Fr)
/// * nblind (Fq)
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
    pub note_commitment: note::StateCommitment,
}

impl OutputCircuit {
    pub fn new(note: Note, v_blinding: Fr, balance_commitment: balance::Commitment) -> Self {
        Self {
            note: note.clone(),
            v_blinding,
            balance_commitment,
            note_commitment: note.commit(),
        }
    }
}

impl ConstraintSynthesizer<Fq> for OutputCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fq>) -> ark_relations::r1cs::Result<()> {
        // Witnesses
        let note_var = note::NoteVar::new_witness(cs.clone(), || Ok(self.note.clone()))?;
        let v_blinding_arr: [u8; 32] = self.v_blinding.to_bytes();
        let v_blinding_vars = UInt8::new_witness_vec(cs.clone(), &v_blinding_arr)?;

        // Public inputs
        let claimed_note_commitment =
            StateCommitmentVar::new_input(cs.clone(), || Ok(self.note_commitment))?;
        let claimed_balance_commitment =
            BalanceCommitmentVar::new_input(cs.clone(), || Ok(self.balance_commitment))?;

        // Check the diversified base is not identity.
        let identity = ElementVar::new_constant(cs, decaf377::Element::default())?;
        identity
            .conditional_enforce_not_equal(&note_var.diversified_generator(), &Boolean::TRUE)?;

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
        let (pk, vk) =
            Groth16::<Bls12_377, LibsnarkReduction>::circuit_specific_setup(circuit, &mut OsRng)
                .expect("can perform circuit specific setup");
        (pk, vk)
    }
}

#[derive(Clone, Debug)]
pub struct OutputProof([u8; GROTH16_PROOF_LENGTH_BYTES]);

impl OutputProof {
    #![allow(clippy::too_many_arguments)]
    /// Generate an [`OutputProof`] given the proving key, public inputs,
    /// witness data, and two random elements `blinding_r` and `blinding_s`.
    pub fn prove(
        blinding_r: Fq,
        blinding_s: Fq,
        pk: &ProvingKey<Bls12_377>,
        note: Note,
        v_blinding: Fr,
        balance_commitment: balance::Commitment,
        note_commitment: note::StateCommitment,
    ) -> anyhow::Result<Self> {
        let circuit = OutputCircuit {
            note,
            note_commitment,
            v_blinding,
            balance_commitment,
        };
        let proof = Groth16::<Bls12_377, LibsnarkReduction>::create_proof_with_reduction(
            circuit, pk, blinding_r, blinding_s,
        )
        .map_err(|err| anyhow::anyhow!(err))?;
        let mut proof_bytes = [0u8; GROTH16_PROOF_LENGTH_BYTES];
        Proof::serialize_compressed(&proof, &mut proof_bytes[..]).expect("can serialize Proof");
        Ok(Self(proof_bytes))
    }

    /// Called to verify the proof using the provided public inputs.
    ///
    /// The public inputs are:
    /// * balance commitment of the new note,
    /// * note commitment of the new note,
    // For debugging proof verification failures:
    // to check that the proof data and verification keys are consistent.
    #[tracing::instrument(level="debug", skip(self, vk), fields(self = ?base64::encode(&self.clone().encode_to_vec()), vk = ?vk.debug_id()))]
    pub fn verify(
        &self,
        vk: &PreparedVerifyingKey<Bls12_377>,
        balance_commitment: balance::Commitment,
        note_commitment: note::StateCommitment,
    ) -> anyhow::Result<()> {
        let proof =
            Proof::deserialize_compressed_unchecked(&self.0[..]).map_err(|e| anyhow::anyhow!(e))?;

        let mut public_inputs = Vec::new();
        public_inputs.extend(note_commitment.0.to_field_elements().unwrap());
        public_inputs.extend(balance_commitment.0.to_field_elements().unwrap());

        tracing::trace!(?public_inputs);
        let start = std::time::Instant::now();
        let proof_result = Groth16::<Bls12_377, LibsnarkReduction>::verify_with_processed_vk(
            &vk,
            public_inputs.as_slice(),
            &proof,
        )
        .map_err(|err| anyhow::anyhow!(err))?;
        tracing::debug!(?proof_result, elapsed = ?start.elapsed());
        proof_result
            .then_some(())
            .ok_or_else(|| anyhow::anyhow!("output proof did not verify"))
    }
}

impl TypeUrl for OutputProof {
    const TYPE_URL: &'static str = "/penumbra.core.crypto.v1alpha1.ZkOutputProof";
}

impl DomainType for OutputProof {
    type Proto = pb::ZkOutputProof;
}

impl From<OutputProof> for pb::ZkOutputProof {
    fn from(proof: OutputProof) -> Self {
        pb::ZkOutputProof {
            inner: proof.0.to_vec(),
        }
    }
}

impl TryFrom<pb::ZkOutputProof> for OutputProof {
    type Error = anyhow::Error;

    fn try_from(proto: pb::ZkOutputProof) -> Result<Self, Self::Error> {
        Ok(OutputProof(proto.inner[..].try_into()?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_ff::UniformRand;
    use decaf377::{Fq, Fr};
    use penumbra_asset::{asset, Balance, Value};
    use penumbra_keys::keys::{SeedPhrase, SpendKey};
    use proptest::prelude::*;

    use penumbra_proto::core::crypto::v1alpha1 as pb;
    use rand_core::OsRng;

    use crate::{note, Note};

    use ark_ff::PrimeField;

    fn fq_strategy() -> BoxedStrategy<Fq> {
        any::<[u8; 32]>()
            .prop_map(|bytes| Fq::from_le_bytes_mod_order(&bytes[..]))
            .boxed()
    }

    fn fr_strategy() -> BoxedStrategy<Fr> {
        any::<[u8; 32]>()
            .prop_map(|bytes| Fr::from_le_bytes_mod_order(&bytes[..]))
            .boxed()
    }

    proptest! {
    #![proptest_config(ProptestConfig::with_cases(2))]
    #[test]
    fn output_proof_happy_path(seed_phrase_randomness in any::<[u8; 32]>(), v_blinding in fr_strategy(), value_amount in 2..200u64) {
            let (pk, vk) = OutputCircuit::generate_prepared_test_parameters();

            let mut rng = OsRng;

            let seed_phrase = SeedPhrase::from_randomness(seed_phrase_randomness);
            let sk_recipient = SpendKey::from_seed_phrase(seed_phrase, 0);
            let fvk_recipient = sk_recipient.full_viewing_key();
            let ivk_recipient = fvk_recipient.incoming();
            let (dest, _dtk_d) = ivk_recipient.payment_address(0u32.into());

            let value_to_send = Value {
                amount: value_amount.into(),
                asset_id: asset::Cache::with_known_assets().get_unit("upenumbra").unwrap().id(),
            };

            let note = Note::generate(&mut rng, &dest, value_to_send);
            let note_commitment = note.commit();
            let balance_commitment = (-Balance::from(value_to_send)).commit(v_blinding);

            let blinding_r = Fq::rand(&mut OsRng);
            let blinding_s = Fq::rand(&mut OsRng);
            let proof = OutputProof::prove(
                blinding_r,
                blinding_s,
                &pk,
                note,
                v_blinding,
                balance_commitment,
                note_commitment,
            )
            .expect("can create proof");
            let serialized_proof: pb::ZkOutputProof = proof.into();

            let deserialized_proof = OutputProof::try_from(serialized_proof).expect("can deserialize proof");
            let proof_result = deserialized_proof.verify(&vk, balance_commitment, note_commitment);

            assert!(proof_result.is_ok());
        }
    }

    proptest! {
    #![proptest_config(ProptestConfig::with_cases(2))]
    #[test]
    fn output_proof_verification_note_commitment_integrity_failure(seed_phrase_randomness in any::<[u8; 32]>(), v_blinding in fr_strategy(), value_amount in 2..200u64, note_blinding in fq_strategy()) {
        let (pk, vk) = OutputCircuit::generate_prepared_test_parameters();
        let mut rng = OsRng;

        let seed_phrase = SeedPhrase::from_randomness(seed_phrase_randomness);
        let sk_recipient = SpendKey::from_seed_phrase(seed_phrase, 0);
        let fvk_recipient = sk_recipient.full_viewing_key();
        let ivk_recipient = fvk_recipient.incoming();
        let (dest, _dtk_d) = ivk_recipient.payment_address(0u32.into());

        let value_to_send = Value {
            amount: value_amount.into(),
            asset_id: asset::Cache::with_known_assets().get_unit("upenumbra").unwrap().id(),
        };

        let note = Note::generate(&mut rng, &dest, value_to_send);
        let note_commitment = note.commit();
        let balance_commitment = (-Balance::from(value_to_send)).commit(v_blinding);

        let blinding_r = Fq::rand(&mut OsRng);
        let blinding_s = Fq::rand(&mut OsRng);
        let proof = OutputProof::prove(
            blinding_r,
            blinding_s,
            &pk,
            note.clone(),
            v_blinding,
            balance_commitment,
            note_commitment,
        )
        .expect("can create proof");

        let incorrect_note_commitment = note::commitment(
            note_blinding,
            value_to_send,
            note.diversified_generator(),
            note.transmission_key_s(),
            note.clue_key(),
        );

        let proof_result = proof.verify(&vk, balance_commitment, incorrect_note_commitment);

        assert!(proof_result.is_err());
    }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(2))]
        #[test]
    fn output_proof_verification_balance_commitment_integrity_failure(seed_phrase_randomness in any::<[u8; 32]>(), v_blinding in fr_strategy(), value_amount in 2..200u64, incorrect_v_blinding in fr_strategy()) {
        let (pk, vk) = OutputCircuit::generate_prepared_test_parameters();
        let mut rng = OsRng;

        let seed_phrase = SeedPhrase::from_randomness(seed_phrase_randomness);
        let sk_recipient = SpendKey::from_seed_phrase(seed_phrase, 0);
        let fvk_recipient = sk_recipient.full_viewing_key();
        let ivk_recipient = fvk_recipient.incoming();
        let (dest, _dtk_d) = ivk_recipient.payment_address(0u32.into());

        let value_to_send = Value {
            amount: value_amount.into(),
            asset_id: asset::Cache::with_known_assets().get_unit("upenumbra").unwrap().id(),
        };

        let note = Note::generate(&mut rng, &dest, value_to_send);
        let note_commitment = note.commit();
        let balance_commitment = (-Balance::from(value_to_send)).commit(v_blinding);

        let blinding_r = Fq::rand(&mut OsRng);
        let blinding_s = Fq::rand(&mut OsRng);
        let proof = OutputProof::prove(
            blinding_r,
            blinding_s,
            &pk,
            note,
            v_blinding,
            balance_commitment,
            note_commitment,
        )
        .expect("can create proof");

        let incorrect_balance_commitment = (-Balance::from(value_to_send)).commit(incorrect_v_blinding);

        let proof_result = proof.verify(&vk, incorrect_balance_commitment, note_commitment);

        assert!(proof_result.is_err());
    }
        }
}
