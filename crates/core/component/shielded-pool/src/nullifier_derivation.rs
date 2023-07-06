use std::str::FromStr;

use ark_r1cs_std::prelude::*;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use decaf377::{Bls12_377, Fq};

use ark_ff::ToConstraintField;
use ark_groth16::{
    r1cs_to_qap::LibsnarkReduction, Groth16, PreparedVerifyingKey, Proof, ProvingKey, VerifyingKey,
};
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef};
use ark_snark::SNARK;
use penumbra_proto::{core::crypto::v1alpha1 as pb, DomainType, TypeUrl};
use penumbra_tct as tct;
use rand::{CryptoRng, Rng};
use rand_core::OsRng;

use crate::{note, Note, Rseed};
use penumbra_asset::Value;
use penumbra_keys::keys::{NullifierKey, NullifierKeyVar, SeedPhrase, SpendKey};
use penumbra_proof_params::{ParameterSetup, VerifyingKeyExt, GROTH16_PROOF_LENGTH_BYTES};
use penumbra_sct::{Nullifier, NullifierVar};

/// Groth16 proof for correct nullifier derivation.
#[derive(Clone, Debug)]
pub struct NullifierDerivationCircuit {
    // Witnesses
    /// The nullifier deriving key.
    nk: NullifierKey,

    // Public inputs
    /// The spent note.
    note: Note,
    /// nullifier of the spent note.
    pub nullifier: Nullifier,
    /// the position of the spent note.
    pub position: tct::Position,
}

impl NullifierDerivationCircuit {
    pub fn new(
        nk: NullifierKey,
        note: Note,
        nullifier: Nullifier,
        position: tct::Position,
    ) -> Self {
        Self {
            nk,
            note,
            nullifier,
            position,
        }
    }
}

impl ConstraintSynthesizer<Fq> for NullifierDerivationCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fq>) -> ark_relations::r1cs::Result<()> {
        // Witnesses
        let nk_var = NullifierKeyVar::new_witness(cs.clone(), || Ok(self.nk))?;

        // Public inputs
        let claimed_nullifier_var = NullifierVar::new_input(cs.clone(), || Ok(self.nullifier))?;
        let note_var = note::NoteVar::new_input(cs.clone(), || Ok(self.note.clone()))?;
        let position_var = tct::r1cs::PositionVar::new_input(cs, || Ok(self.position))?;

        // Nullifier integrity.
        let note_commitment = note_var.commit()?;
        let nullifier_var = NullifierVar::derive(&nk_var, &position_var, &note_commitment)?;
        nullifier_var.conditional_enforce_equal(&claimed_nullifier_var, &Boolean::TRUE)?;

        Ok(())
    }
}

impl ParameterSetup for NullifierDerivationCircuit {
    fn generate_test_parameters() -> (ProvingKey<Bls12_377>, VerifyingKey<Bls12_377>) {
        let seed_phrase = SeedPhrase::from_randomness([b'f'; 32]);
        let sk_sender = SpendKey::from_seed_phrase(seed_phrase, 0);
        let fvk_sender = sk_sender.full_viewing_key();
        let ivk_sender = fvk_sender.incoming();
        let (address, _dtk_d) = ivk_sender.payment_address(0u32.into());

        let nk = *sk_sender.nullifier_key();
        let note = Note::from_parts(
            address,
            Value::from_str("1upenumbra").expect("valid value"),
            Rseed([1u8; 32]),
        )
        .expect("can make a note");
        let nullifier = Nullifier(Fq::from(1));
        let mut sct = tct::Tree::new();
        let note_commitment = note.commit();
        sct.insert(tct::Witness::Keep, note_commitment).unwrap();
        let state_commitment_proof = sct.witness(note_commitment).unwrap();
        let position = state_commitment_proof.position();

        let circuit = NullifierDerivationCircuit {
            note,
            nk,
            nullifier,
            position,
        };
        let (pk, vk) =
            Groth16::<Bls12_377, LibsnarkReduction>::circuit_specific_setup(circuit, &mut OsRng)
                .expect("can perform circuit specific setup");
        (pk, vk)
    }
}

#[derive(Clone, Debug)]
pub struct NullifierDerivationProof([u8; GROTH16_PROOF_LENGTH_BYTES]);

impl NullifierDerivationProof {
    #![allow(clippy::too_many_arguments)]
    pub fn prove<R: CryptoRng + Rng>(
        rng: &mut R,
        pk: &ProvingKey<Bls12_377>,
        position: tct::Position,
        note: Note,
        nk: NullifierKey,
        nullifier: Nullifier,
    ) -> anyhow::Result<Self> {
        let circuit = NullifierDerivationCircuit {
            note,
            position,
            nk,
            nullifier,
        };
        let proof = Groth16::<Bls12_377, LibsnarkReduction>::prove(pk, circuit, rng)
            .map_err(|err| anyhow::anyhow!(err))?;
        let mut proof_bytes = [0u8; GROTH16_PROOF_LENGTH_BYTES];
        Proof::serialize_compressed(&proof, &mut proof_bytes[..]).expect("can serialize Proof");
        Ok(Self(proof_bytes))
    }

    /// Called to verify the proof using the provided public inputs.
    #[tracing::instrument(level="debug", skip(self, vk), fields(self = ?base64::encode(&self.clone().encode_to_vec()), vk = ?vk.debug_id()))]
    pub fn verify(
        &self,
        vk: &PreparedVerifyingKey<Bls12_377>,
        position: tct::Position,
        note: Note,
        nullifier: Nullifier,
    ) -> anyhow::Result<()> {
        let proof =
            Proof::deserialize_compressed_unchecked(&self.0[..]).map_err(|e| anyhow::anyhow!(e))?;

        let mut public_inputs = Vec::new();
        public_inputs.extend(nullifier.0.to_field_elements().unwrap());
        public_inputs.extend(note.to_field_elements().unwrap());
        public_inputs.extend(Fq::from(u64::from(position)).to_field_elements().unwrap());

        tracing::trace!(?public_inputs);
        let start = std::time::Instant::now();
        let proof_result = Groth16::<Bls12_377, LibsnarkReduction>::verify_with_processed_vk(
            vk,
            public_inputs.as_slice(),
            &proof,
        )
        .map_err(|err| anyhow::anyhow!(err))?;
        tracing::debug!(?proof_result, elapsed = ?start.elapsed());
        proof_result
            .then_some(())
            .ok_or_else(|| anyhow::anyhow!("spend proof did not verify"))
    }
}

impl TypeUrl for NullifierDerivationProof {
    const TYPE_URL: &'static str = "penumbra.core.crypto.v1alpha1.ZKNullifierDerivationProof";
}

impl DomainType for NullifierDerivationProof {
    type Proto = pb::ZkNullifierDerivationProof;
}

impl From<NullifierDerivationProof> for pb::ZkNullifierDerivationProof {
    fn from(proof: NullifierDerivationProof) -> Self {
        pb::ZkNullifierDerivationProof {
            inner: proof.0.to_vec(),
        }
    }
}

impl TryFrom<pb::ZkNullifierDerivationProof> for NullifierDerivationProof {
    type Error = anyhow::Error;

    fn try_from(proto: pb::ZkNullifierDerivationProof) -> Result<Self, Self::Error> {
        Ok(NullifierDerivationProof(proto.inner[..].try_into()?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use decaf377::{Fq, Fr};
    use penumbra_asset::{asset, Value};
    use penumbra_keys::keys::{SeedPhrase, SpendKey};
    use penumbra_sct::Nullifier;
    use proptest::prelude::*;

    use penumbra_tct as tct;
    use rand_core::OsRng;

    use crate::Note;

    use ark_ff::PrimeField;

    proptest! {
    #![proptest_config(ProptestConfig::with_cases(2))]
    #[test]
    fn nullifier_derivation_proof_happy_path(seed_phrase_randomness in any::<[u8; 32]>(), value_amount in 2..200u64) {
            let (pk, vk) = NullifierDerivationCircuit::generate_prepared_test_parameters();

            let mut rng = OsRng;

            let seed_phrase = SeedPhrase::from_randomness(seed_phrase_randomness);
            let sk_sender = SpendKey::from_seed_phrase(seed_phrase, 0);
            let fvk_sender = sk_sender.full_viewing_key();
            let ivk_sender = fvk_sender.incoming();
            let (sender, _dtk_d) = ivk_sender.payment_address(0u32.into());

            let value_to_send = Value {
                amount: value_amount.into(),
                asset_id: asset::Cache::with_known_assets().get_unit("upenumbra").unwrap().id(),
            };

            let note = Note::generate(&mut rng, &sender, value_to_send);
            let note_commitment = note.commit();
            let nk = *sk_sender.nullifier_key();
            let mut sct = tct::Tree::new();

            sct.insert(tct::Witness::Keep, note_commitment).unwrap();
            let state_commitment_proof = sct.witness(note_commitment).unwrap();
            let position = state_commitment_proof.position();
            let nullifier = Nullifier::derive(&nk, state_commitment_proof.position(), &note_commitment);

                let proof = NullifierDerivationProof::prove(
                    &mut rng,
                    &pk,
                    position,
                    note.clone(),
                    nk,
                    nullifier,
                )
                .expect("can create proof");

                let proof_result = proof.verify(&vk, position, note, nullifier);

                assert!(proof_result.is_ok());
        }
    }
}
