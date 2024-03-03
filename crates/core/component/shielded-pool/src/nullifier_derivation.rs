use base64::prelude::*;
use std::str::FromStr;

use anyhow::Result;
use ark_r1cs_std::prelude::*;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use decaf377::{Bls12_377, Fq};

use ark_ff::ToConstraintField;
use ark_groth16::{
    r1cs_to_qap::LibsnarkReduction, Groth16, PreparedVerifyingKey, Proof, ProvingKey,
};
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef};
use ark_snark::SNARK;
use penumbra_proto::{penumbra::core::component::shielded_pool::v1 as pb, DomainType};
use penumbra_tct as tct;
use rand::{CryptoRng, Rng};
use tct::StateCommitment;

use crate::{Note, Rseed};
use penumbra_asset::Value;
use penumbra_keys::keys::{Bip44Path, NullifierKey, NullifierKeyVar, SeedPhrase, SpendKey};
use penumbra_proof_params::{DummyWitness, VerifyingKeyExt, GROTH16_PROOF_LENGTH_BYTES};
use penumbra_sct::{Nullifier, NullifierVar};

/// The public input for a ['NullifierDerivationProof'].
#[derive(Clone, Debug)]
pub struct NullifierDerivationProofPublic {
    /// The position of the spent note.
    pub position: tct::Position,
    /// A commitment to the spent note.
    pub note_commitment: StateCommitment,
    /// nullifier of the spent note.
    pub nullifier: Nullifier,
}

/// The private input for a ['NullifierDerivationProof'].
#[derive(Clone, Debug)]
pub struct NullifierDerivationProofPrivate {
    /// The nullifier deriving key.
    pub nk: NullifierKey,
}

#[cfg(test)]
fn check_satisfaction(
    public: &NullifierDerivationProofPublic,
    private: &NullifierDerivationProofPrivate,
) -> Result<()> {
    let nullifier = Nullifier::derive(&private.nk, public.position, &public.note_commitment);
    if nullifier != public.nullifier {
        anyhow::bail!("nullifier did not match public input");
    }
    Ok(())
}

#[cfg(test)]
fn check_circuit_satisfaction(
    public: NullifierDerivationProofPublic,
    private: NullifierDerivationProofPrivate,
) -> Result<()> {
    use ark_relations::r1cs::{self, ConstraintSystem};

    let cs = ConstraintSystem::new_ref();
    let circuit = NullifierDerivationCircuit { public, private };
    cs.set_optimization_goal(r1cs::OptimizationGoal::Constraints);
    circuit
        .generate_constraints(cs.clone())
        .expect("can generate constraints from circuit");
    cs.finalize();
    if !cs.is_satisfied()? {
        anyhow::bail!("constraints are not satisfied");
    }
    Ok(())
}

/// Groth16 proof for correct nullifier derivation.
#[derive(Clone, Debug)]
pub struct NullifierDerivationCircuit {
    public: NullifierDerivationProofPublic,
    private: NullifierDerivationProofPrivate,
}

impl ConstraintSynthesizer<Fq> for NullifierDerivationCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fq>) -> ark_relations::r1cs::Result<()> {
        // Witnesses
        let nk_var = NullifierKeyVar::new_witness(cs.clone(), || Ok(self.private.nk))?;

        // Public inputs
        let claimed_nullifier_var =
            NullifierVar::new_input(cs.clone(), || Ok(self.public.nullifier))?;
        let note_commitment_var = tct::r1cs::StateCommitmentVar::new_input(cs.clone(), || {
            Ok(self.public.note_commitment)
        })?;
        let position_var = tct::r1cs::PositionVar::new_input(cs, || Ok(self.public.position))?;

        // Nullifier integrity.
        let nullifier_var = NullifierVar::derive(&nk_var, &position_var, &note_commitment_var)?;
        nullifier_var.conditional_enforce_equal(&claimed_nullifier_var, &Boolean::TRUE)?;

        Ok(())
    }
}

impl DummyWitness for NullifierDerivationCircuit {
    fn with_dummy_witness() -> Self {
        let seed_phrase = SeedPhrase::from_randomness(&[b'f'; 32]);
        let sk_sender = SpendKey::from_seed_phrase_bip44(seed_phrase, &Bip44Path::new(0));
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
        let nullifier = Nullifier(Fq::from(1u64));
        let mut sct = tct::Tree::new();
        let note_commitment = note.commit();
        sct.insert(tct::Witness::Keep, note_commitment)
            .expect("able to insert note commitment into SCT");
        let state_commitment_proof = sct
            .witness(note_commitment)
            .expect("able to witness just-inserted note commitment");
        let position = state_commitment_proof.position();

        let public = NullifierDerivationProofPublic {
            position,
            note_commitment,
            nullifier,
        };
        let private = NullifierDerivationProofPrivate { nk };

        Self { public, private }
    }
}

#[derive(Clone, Debug)]
pub struct NullifierDerivationProof([u8; GROTH16_PROOF_LENGTH_BYTES]);

impl NullifierDerivationProof {
    pub fn prove<R: CryptoRng + Rng>(
        rng: &mut R,
        pk: &ProvingKey<Bls12_377>,
        public: NullifierDerivationProofPublic,
        private: NullifierDerivationProofPrivate,
    ) -> anyhow::Result<Self> {
        let circuit = NullifierDerivationCircuit { public, private };
        let proof = Groth16::<Bls12_377, LibsnarkReduction>::prove(pk, circuit, rng)
            .map_err(|err| anyhow::anyhow!(err))?;
        let mut proof_bytes = [0u8; GROTH16_PROOF_LENGTH_BYTES];
        Proof::serialize_compressed(&proof, &mut proof_bytes[..]).expect("can serialize Proof");
        Ok(Self(proof_bytes))
    }

    /// Called to verify the proof using the provided public inputs.
    #[tracing::instrument(level="debug", skip(self, vk), fields(self = ?BASE64_STANDARD.encode(&self.0), vk = ?vk.debug_id()))]
    pub fn verify(
        &self,
        vk: &PreparedVerifyingKey<Bls12_377>,
        public: NullifierDerivationProofPublic,
    ) -> anyhow::Result<()> {
        let proof =
            Proof::deserialize_compressed_unchecked(&self.0[..]).map_err(|e| anyhow::anyhow!(e))?;

        let mut public_inputs = Vec::new();
        public_inputs.extend(
            public
                .nullifier
                .0
                .to_field_elements()
                .ok_or_else(|| anyhow::anyhow!("could not convert nullifier to field elements"))?,
        );
        public_inputs.extend(
            public
                .note_commitment
                .0
                .to_field_elements()
                .ok_or_else(|| {
                    anyhow::anyhow!("could not convert note commitment to field elements")
                })?,
        );
        public_inputs.extend(
            public
                .position
                .to_field_elements()
                .ok_or_else(|| anyhow::anyhow!("could not convert position to field elements"))?,
        );

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
            .ok_or_else(|| anyhow::anyhow!("nullifier derivation proof did not verify"))
    }
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
    use penumbra_asset::{asset, Value};
    use penumbra_keys::keys::{SeedPhrase, SpendKey};
    use penumbra_num::Amount;
    use penumbra_sct::Nullifier;
    use proptest::prelude::*;

    use crate::Note;

    prop_compose! {
        fn arb_valid_nullifier_derivation_statement()(amount in any::<u64>(), address_index in any::<u32>(), position in any::<(u16, u16, u16)>(), asset_id64 in any::<u64>(), seed_phrase_randomness in any::<[u8; 32]>(), rseed_randomness in any::<[u8; 32]>()) -> (NullifierDerivationProofPublic, NullifierDerivationProofPrivate) {
            let seed_phrase = SeedPhrase::from_randomness(&seed_phrase_randomness);
            let sk_sender = SpendKey::from_seed_phrase_bip44(seed_phrase, &Bip44Path::new(0));
            let fvk_sender = sk_sender.full_viewing_key();
            let ivk_sender = fvk_sender.incoming();
            let (sender, _dtk_d) = ivk_sender.payment_address(address_index.into());
            let nk = *sk_sender.nullifier_key();
            let note = Note::from_parts(
                sender,
                Value {
                    amount: Amount::from(amount),
                    asset_id: asset::Id(Fq::from(asset_id64)),
                },
                Rseed(rseed_randomness),
            ).expect("should be able to create note");
            let nullifier = Nullifier::derive(&nk, position.into(), &note.commit());
            let public = NullifierDerivationProofPublic {
                position: position.into(),
                note_commitment: note.commit(),
                nullifier
            };
            let private = NullifierDerivationProofPrivate {
                nk,
            };
            (public, private)
        }
    }

    prop_compose! {
        // An invalid nullifier derivation statement is derived here by
        // adding a random value to the nullifier key. The circuit should
        // be unsatisfiable if the witnessed nullifier key is incorrect, i.e.
        // does not match the nullifier key used to derive the nullifier.
        fn arb_invalid_nullifier_derivation_statement()(amount in any::<u64>(), address_index in any::<u32>(), position in any::<(u16, u16, u16)>(), invalid_nk_randomness in any::<[u8; 32]>(), asset_id64 in any::<u64>(), seed_phrase_randomness in any::<[u8; 32]>(), rseed_randomness in any::<[u8; 32]>()) -> (NullifierDerivationProofPublic, NullifierDerivationProofPrivate) {
            let seed_phrase = SeedPhrase::from_randomness(&seed_phrase_randomness);
            let sk_sender = SpendKey::from_seed_phrase_bip44(seed_phrase, &Bip44Path::new(0));
            let fvk_sender = sk_sender.full_viewing_key();
            let ivk_sender = fvk_sender.incoming();
            let (sender, _dtk_d) = ivk_sender.payment_address(address_index.into());
            let nk = *sk_sender.nullifier_key();
            let incorrect_nk = NullifierKey(nk.0 + Fq::from_le_bytes_mod_order(&invalid_nk_randomness));
            let note = Note::from_parts(
                sender,
                Value {
                    amount: Amount::from(amount),
                    asset_id: asset::Id(Fq::from(asset_id64)),
                },
                Rseed(rseed_randomness),
            ).expect("should be able to create note");
            let nullifier = Nullifier::derive(&nk, position.into(), &note.commit());

            let public = NullifierDerivationProofPublic {
                position: position.into(),
                note_commitment: note.commit(),
                nullifier
            };
            let private = NullifierDerivationProofPrivate {
                nk: incorrect_nk,
            };
            (public, private)
        }
    }

    proptest! {
        #[test]
        fn nullifier_derivation_proof_happy_path((public, private) in arb_valid_nullifier_derivation_statement()) {
            assert!(check_satisfaction(&public, &private).is_ok());
            assert!(check_circuit_satisfaction(public, private).is_ok());
        }
    }

    proptest! {
        #[test]
        fn nullifier_derivation_proof_unhappy_path((public, private) in arb_invalid_nullifier_derivation_statement()) {
            assert!(check_satisfaction(&public, &private).is_err());
            assert!(check_circuit_satisfaction(public, private).is_err());
        }
    }
}
