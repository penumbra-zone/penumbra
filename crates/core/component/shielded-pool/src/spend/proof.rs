use std::str::FromStr;

use anyhow::Result;
use ark_r1cs_std::{
    prelude::{EqGadget, FieldVar},
    uint8::UInt8,
    ToBitsGadget,
};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use decaf377::{r1cs::ElementVar, FieldExt};
use decaf377::{r1cs::FqVar, Bls12_377, Fq, Fr};

use ark_ff::ToConstraintField;
use ark_groth16::{
    r1cs_to_qap::LibsnarkReduction, Groth16, PreparedVerifyingKey, Proof, ProvingKey,
};
use ark_r1cs_std::prelude::AllocVar;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef};
use ark_snark::SNARK;
use decaf377_rdsa::{SpendAuth, VerificationKey};
use penumbra_proto::{penumbra::core::component::shielded_pool::v1alpha1 as pb, DomainType};
use penumbra_tct as tct;
use penumbra_tct::r1cs::StateCommitmentVar;

use crate::{note, Note, Rseed};
use penumbra_asset::{balance, balance::commitment::BalanceCommitmentVar, Value};
use penumbra_keys::keys::{
    AuthorizationKeyVar, Bip44Path, IncomingViewingKeyVar, NullifierKey, NullifierKeyVar,
    RandomizedVerificationKey, SeedPhrase, SpendAuthRandomizerVar, SpendKey,
};
use penumbra_proof_params::{DummyWitness, VerifyingKeyExt, GROTH16_PROOF_LENGTH_BYTES};
use penumbra_sct::{Nullifier, NullifierVar};

/// The public input for a [`SpendProof`].
#[derive(Clone, Debug)]
pub struct SpendProofPublic {
    /// the merkle root of the state commitment tree.
    pub anchor: tct::Root,
    /// value commitment of the note to be spent.
    pub balance_commitment: balance::Commitment,
    /// nullifier of the note to be spent.
    pub nullifier: Nullifier,
    /// the randomized verification spend key.
    pub rk: VerificationKey<SpendAuth>,
}

/// The private input for a [`SpendProof`].
#[derive(Clone, Debug)]
pub struct SpendProofPrivate {
    /// Inclusion proof for the note commitment.
    pub state_commitment_proof: tct::Proof,
    /// The note being spent.
    pub note: Note,
    /// The blinding factor used for generating the value commitment.
    pub v_blinding: Fr,
    /// The randomizer used for generating the randomized spend auth key.
    pub spend_auth_randomizer: Fr,
    /// The spend authorization key.
    pub ak: VerificationKey<SpendAuth>,
    /// The nullifier deriving key.
    pub nk: NullifierKey,
}

#[cfg(test)]
fn check_satisfaction(public: &SpendProofPublic, private: &SpendProofPrivate) -> Result<()> {
    use penumbra_keys::keys::FullViewingKey;

    let amount_u128: u128 = private.note.value().amount.into();
    if amount_u128 == 0u128 {
        return Ok(());
    }

    let note_commitment = private.note.commit();
    if note_commitment != private.state_commitment_proof.commitment() {
        anyhow::bail!("note commitment did not match state commitment proof");
    }

    let nullifier = Nullifier::derive(
        &private.nk,
        private.state_commitment_proof.position(),
        &note_commitment,
    );
    if nullifier != public.nullifier {
        anyhow::bail!("nullifier did not match public input");
    }

    private.state_commitment_proof.verify(public.anchor)?;

    let rk = private.ak.randomize(&private.spend_auth_randomizer);
    if rk != public.rk {
        anyhow::bail!("randomized spend auth key did not match public input");
    }

    let fvk = FullViewingKey::from_components(private.ak, private.nk);
    let ivk = fvk.incoming();
    let transmission_key = ivk.diversified_public(&private.note.diversified_generator());
    if transmission_key != *private.note.transmission_key() {
        anyhow::bail!("transmission key did not match note");
    }

    let balance_commitment = private.note.value().commit(private.v_blinding);
    if balance_commitment != public.balance_commitment {
        anyhow::bail!("balance commitment did not match public input");
    }

    if private.note.diversified_generator() == decaf377::Element::default() {
        anyhow::bail!("diversified generator is identity");
    }
    if private.ak.is_identity() {
        anyhow::bail!("ak is identity");
    }

    Ok(())
}

#[cfg(test)]
fn check_circuit_satisfaction(public: SpendProofPublic, private: SpendProofPrivate) -> Result<()> {
    use ark_relations::r1cs::{self, ConstraintSystem};

    let cs = ConstraintSystem::new_ref();
    let circuit = SpendCircuit { public, private };
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

/// Groth16 proof for spending existing notes.
#[derive(Clone, Debug)]
pub struct SpendCircuit {
    public: SpendProofPublic,
    private: SpendProofPrivate,
}

impl ConstraintSynthesizer<Fq> for SpendCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fq>) -> ark_relations::r1cs::Result<()> {
        // Witnesses
        let note_var = note::NoteVar::new_witness(cs.clone(), || Ok(self.private.note.clone()))?;
        let claimed_note_commitment = StateCommitmentVar::new_witness(cs.clone(), || {
            Ok(self.private.state_commitment_proof.commitment())
        })?;

        let position_var = tct::r1cs::PositionVar::new_witness(cs.clone(), || {
            Ok(self.private.state_commitment_proof.position())
        })?;
        let position_bits = position_var.to_bits_le()?;
        let merkle_path_var = tct::r1cs::MerkleAuthPathVar::new_witness(cs.clone(), || {
            Ok(self.private.state_commitment_proof)
        })?;

        let v_blinding_arr: [u8; 32] = self.private.v_blinding.to_bytes();
        let v_blinding_vars = UInt8::new_witness_vec(cs.clone(), &v_blinding_arr)?;

        let spend_auth_randomizer_var = SpendAuthRandomizerVar::new_witness(cs.clone(), || {
            Ok(self.private.spend_auth_randomizer)
        })?;
        let ak_element_var: AuthorizationKeyVar =
            AuthorizationKeyVar::new_witness(cs.clone(), || Ok(self.private.ak))?;
        let nk_var = NullifierKeyVar::new_witness(cs.clone(), || Ok(self.private.nk))?;

        // Public inputs
        let anchor_var = FqVar::new_input(cs.clone(), || Ok(Fq::from(self.public.anchor)))?;
        let claimed_balance_commitment_var =
            BalanceCommitmentVar::new_input(cs.clone(), || Ok(self.public.balance_commitment))?;
        let claimed_nullifier_var =
            NullifierVar::new_input(cs.clone(), || Ok(self.public.nullifier))?;
        let rk_var = RandomizedVerificationKey::new_input(cs.clone(), || Ok(self.public.rk))?;

        // We short circuit to true if value released is 0. That means this is a _dummy_ spend.
        let is_dummy = note_var.amount().is_eq(&FqVar::zero())?;
        // We use a Boolean constraint to enforce the below constraints only if this is not a
        // dummy spend.
        let is_not_dummy = is_dummy.not();

        // Note commitment integrity.
        let note_commitment_var = note_var.commit()?;
        note_commitment_var.conditional_enforce_equal(&claimed_note_commitment, &is_not_dummy)?;

        // Nullifier integrity.
        let nullifier_var = NullifierVar::derive(&nk_var, &position_var, &claimed_note_commitment)?;
        nullifier_var.conditional_enforce_equal(&claimed_nullifier_var, &is_not_dummy)?;

        // Merkle auth path verification against the provided anchor.
        merkle_path_var.verify(
            cs.clone(),
            &is_not_dummy,
            &position_bits,
            anchor_var,
            claimed_note_commitment.inner(),
        )?;

        // Check integrity of randomized verification key.
        let computed_rk_var = ak_element_var.randomize(&spend_auth_randomizer_var)?;
        computed_rk_var.conditional_enforce_equal(&rk_var, &is_not_dummy)?;

        // Check integrity of diversified address.
        let ivk = IncomingViewingKeyVar::derive(&nk_var, &ak_element_var)?;
        let computed_transmission_key =
            ivk.diversified_public(&note_var.diversified_generator())?;
        computed_transmission_key
            .conditional_enforce_equal(&note_var.transmission_key(), &is_not_dummy)?;

        // Check integrity of balance commitment.
        let balance_commitment = note_var.value().commit(v_blinding_vars)?;
        balance_commitment
            .conditional_enforce_equal(&claimed_balance_commitment_var, &is_not_dummy)?;

        // Check the diversified base is not identity.
        let identity = ElementVar::new_constant(cs, decaf377::Element::default())?;
        identity.conditional_enforce_not_equal(&note_var.diversified_generator(), &is_not_dummy)?;
        // Check the ak is not identity.
        identity.conditional_enforce_not_equal(&ak_element_var.inner, &is_not_dummy)?;

        Ok(())
    }
}

impl DummyWitness for SpendCircuit {
    fn with_dummy_witness() -> Self {
        let seed_phrase = SeedPhrase::from_randomness(&[b'f'; 32]);
        let sk_sender = SpendKey::from_seed_phrase_bip44(seed_phrase, &Bip44Path::new(0));
        let fvk_sender = sk_sender.full_viewing_key();
        let ivk_sender = fvk_sender.incoming();
        let (address, _dtk_d) = ivk_sender.payment_address(0u32.into());

        let spend_auth_randomizer = Fr::from(1);
        let rsk = sk_sender.spend_auth_key().randomize(&spend_auth_randomizer);
        let nk = *sk_sender.nullifier_key();
        let ak = sk_sender.spend_auth_key().into();
        let note = Note::from_parts(
            address,
            Value::from_str("1upenumbra").expect("valid value"),
            Rseed([1u8; 32]),
        )
        .expect("can make a note");
        let v_blinding = Fr::from(1);
        let rk: VerificationKey<SpendAuth> = rsk.into();
        let nullifier = Nullifier(Fq::from(1));
        let mut sct = tct::Tree::new();
        let anchor: tct::Root = sct.root();
        let note_commitment = note.commit();
        sct.insert(tct::Witness::Keep, note_commitment)
            .expect("able to insert note commitment into SCT");
        let state_commitment_proof = sct
            .witness(note_commitment)
            .expect("able to witness just-inserted note commitment");

        let public = SpendProofPublic {
            anchor,
            balance_commitment: balance::Commitment(decaf377::basepoint()),
            nullifier,
            rk,
        };
        let private = SpendProofPrivate {
            state_commitment_proof,
            note,
            v_blinding,
            spend_auth_randomizer,
            ak,
            nk,
        };

        Self { public, private }
    }
}

#[derive(Clone, Debug)]
pub struct SpendProof([u8; GROTH16_PROOF_LENGTH_BYTES]);

impl SpendProof {
    /// Generate a `SpendProof` given the proving key, public inputs,
    /// witness data, and two random elements `blinding_r` and `blinding_s`.
    pub fn prove(
        blinding_r: Fq,
        blinding_s: Fq,
        pk: &ProvingKey<Bls12_377>,
        public: SpendProofPublic,
        private: SpendProofPrivate,
    ) -> anyhow::Result<Self> {
        let circuit = SpendCircuit { public, private };
        let proof = Groth16::<Bls12_377, LibsnarkReduction>::create_proof_with_reduction(
            circuit, pk, blinding_r, blinding_s,
        )
        .map_err(|err| anyhow::anyhow!(err))?;
        let mut proof_bytes = [0u8; GROTH16_PROOF_LENGTH_BYTES];
        Proof::serialize_compressed(&proof, &mut proof_bytes[..]).expect("can serialize Proof");
        Ok(Self(proof_bytes))
    }

    /// Called to verify the proof using the provided public inputs.
    // For debugging proof verification failures,
    // to check that the proof data and verification keys are consistent.
    #[tracing::instrument(level="debug", skip(self, vk), fields(self = ?base64::encode(self.clone().encode_to_vec()), vk = ?vk.debug_id()))]
    pub fn verify(
        &self,
        vk: &PreparedVerifyingKey<Bls12_377>,
        public: SpendProofPublic,
    ) -> anyhow::Result<()> {
        let proof =
            Proof::deserialize_compressed_unchecked(&self.0[..]).map_err(|e| anyhow::anyhow!(e))?;

        let mut public_inputs = Vec::new();
        public_inputs.extend([Fq::from(public.anchor.0)]);
        public_inputs.extend(
            public
                .balance_commitment
                .0
                .to_field_elements()
                .ok_or_else(|| anyhow::anyhow!("balance commitment is not a valid element"))?,
        );
        public_inputs.extend(
            public
                .nullifier
                .0
                .to_field_elements()
                .ok_or_else(|| anyhow::anyhow!("nullifier is not a valid element"))?,
        );
        let element_rk = decaf377::Encoding(public.rk.to_bytes())
            .vartime_decompress()
            .expect("expect only valid element points");
        public_inputs.extend(
            element_rk
                .to_field_elements()
                .ok_or_else(|| anyhow::anyhow!("rk is not a valid element"))?,
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
            .ok_or_else(|| anyhow::anyhow!("spend proof did not verify"))
    }
}

impl DomainType for SpendProof {
    type Proto = pb::ZkSpendProof;
}

impl From<SpendProof> for pb::ZkSpendProof {
    fn from(proof: SpendProof) -> Self {
        pb::ZkSpendProof {
            inner: proof.0.to_vec(),
        }
    }
}

impl TryFrom<pb::ZkSpendProof> for SpendProof {
    type Error = anyhow::Error;

    fn try_from(proto: pb::ZkSpendProof) -> Result<Self, Self::Error> {
        Ok(SpendProof(proto.inner[..].try_into()?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_ff::UniformRand;
    use ark_r1cs_std::prelude::Boolean;
    use decaf377::{Fq, Fr};
    use penumbra_asset::{asset, Value};
    use penumbra_keys::{
        keys::{Bip44Path, SeedPhrase, SpendKey},
        Address,
    };
    use penumbra_num::Amount;
    use penumbra_proof_params::generate_prepared_test_parameters;
    use penumbra_sct::Nullifier;
    use penumbra_tct::StateCommitment;
    use proptest::prelude::*;

    use decaf377_rdsa::{SpendAuth, VerificationKey};
    use penumbra_tct as tct;
    use rand_core::OsRng;

    use crate::Note;

    use ark_ff::PrimeField;

    fn fr_strategy() -> BoxedStrategy<Fr> {
        any::<[u8; 32]>()
            .prop_map(|bytes| Fr::from_le_bytes_mod_order(&bytes[..]))
            .boxed()
    }

    prop_compose! {
        fn arb_valid_spend_statement()(v_blinding in fr_strategy(), spend_auth_randomizer in fr_strategy(), asset_id64 in any::<u64>(), address_index in any::<u32>(), amount in any::<u64>(), seed_phrase_randomness in any::<[u8; 32]>(), rseed_randomness in any::<[u8; 32]>(), num_commitments in 0..100) -> (SpendProofPublic, SpendProofPrivate) {
            let seed_phrase = SeedPhrase::from_randomness(&seed_phrase_randomness);
            let sk_sender = SpendKey::from_seed_phrase_bip44(seed_phrase, &Bip44Path::new(0));
            let fvk_sender = sk_sender.full_viewing_key();
            let ivk_sender = fvk_sender.incoming();
            let (sender, _dtk_d) = ivk_sender.payment_address(address_index.into());
            let value_to_send = Value {
                amount: Amount::from(amount),
                asset_id: asset::Id(Fq::from(asset_id64)),
            };
            let note = Note::from_parts(
                sender,
                value_to_send,
                Rseed(rseed_randomness),
            ).expect("should be able to create note");
            let note_commitment = note.commit();
            let rsk = sk_sender.spend_auth_key().randomize(&spend_auth_randomizer);
            let nk = *sk_sender.nullifier_key();
            let ak: VerificationKey<SpendAuth> = sk_sender.spend_auth_key().into();

            let mut sct = tct::Tree::new();

            // Next, we simulate the case where the SCT is not empty by adding `num_commitments`
            // unrelated items in the SCT.
            for i in 0..num_commitments {
                // To avoid duplicate note commitments, we use the `i` counter as the Rseed randomness
                let rseed = Rseed([i as u8; 32]);
                let dummy_note_commitment = Note::from_parts(sender, value_to_send, rseed).expect("can create note").commit();
                sct.insert(tct::Witness::Keep, dummy_note_commitment).expect("should be able to insert note commitments into the SCT");
            }

            sct.insert(tct::Witness::Keep, note_commitment).expect("should be able to insert note commitments into the SCT");
            let anchor = sct.root();
            let state_commitment_proof = sct.witness(note_commitment).expect("can witness note commitment");
            let balance_commitment = value_to_send.commit(v_blinding);
            let rk: VerificationKey<SpendAuth> = rsk.into();
            let nullifier = Nullifier::derive(&nk, state_commitment_proof.position(), &note_commitment);

            let public = SpendProofPublic {
                anchor,
                balance_commitment,
                nullifier,
                rk,
            };
            let private = SpendProofPrivate {
                state_commitment_proof,
                note,
                v_blinding,
                spend_auth_randomizer,
                ak,
                nk,
            };
            (public, private)
        }
    }

    proptest! {
        #[test]
        fn spend_proof_happy_path((public, private) in arb_valid_spend_statement()) {
            assert!(check_satisfaction(&public, &private).is_ok());
            assert!(check_circuit_satisfaction(public, private).is_ok());
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(2))]
    #[test]
    /// Check that the `SpendProof` verification fails when using an incorrect
    /// TCT root (`anchor`).
    fn spend_proof_verification_merkle_path_integrity_failure(seed_phrase_randomness in any::<[u8; 32]>(), spend_auth_randomizer in fr_strategy(), value_amount in 2..200u64, v_blinding in fr_strategy()) {
        let mut rng = OsRng;
        let (pk, vk) = generate_prepared_test_parameters::<SpendCircuit>(&mut rng);

        let seed_phrase = SeedPhrase::from_randomness(&seed_phrase_randomness);
        let sk_sender = SpendKey::from_seed_phrase_bip44(seed_phrase, &Bip44Path::new(0));
        let fvk_sender = sk_sender.full_viewing_key();
        let ivk_sender = fvk_sender.incoming();
        let (sender, _dtk_d) = ivk_sender.payment_address(0u32.into());

        let value_to_send = Value {
            amount: value_amount.into(),
            asset_id: asset::Cache::with_known_assets().get_unit("upenumbra").unwrap().id(),
        };

        let note = Note::generate(&mut rng, &sender, value_to_send);
        let note_commitment = note.commit();
        let rsk = sk_sender.spend_auth_key().randomize(&spend_auth_randomizer);
        let nk = *sk_sender.nullifier_key();
        let ak: VerificationKey<SpendAuth> = sk_sender.spend_auth_key().into();
        let mut sct = tct::Tree::new();
        let incorrect_anchor = sct.root();
        sct.insert(tct::Witness::Keep, note_commitment).unwrap();
        let anchor = sct.root();
        let state_commitment_proof = sct.witness(note_commitment).unwrap();
        let balance_commitment = value_to_send.commit(v_blinding);
        let rk: VerificationKey<SpendAuth> = rsk.into();
        let nf = Nullifier::derive(&nk, 0.into(), &note_commitment);

        let blinding_r = Fq::rand(&mut OsRng);
        let blinding_s = Fq::rand(&mut OsRng);
        let public = SpendProofPublic {
            anchor,
            balance_commitment,
            nullifier: nf,
            rk,
        };
        let private = SpendProofPrivate {
            state_commitment_proof,
            note,
            v_blinding,
            spend_auth_randomizer,
            ak,
            nk,
        };
        let proof = SpendProof::prove(
            blinding_r,
            blinding_s,
            &pk,
            public,
            private
        )
        .expect("can create proof");

        let incorrect_public = SpendProofPublic {
            anchor: incorrect_anchor,
            balance_commitment,
            nullifier: nf,
            rk,
        };
        let proof_result = proof.verify(&vk, incorrect_public);
        assert!(proof_result.is_err());
    }
    }

    proptest! {
            #![proptest_config(ProptestConfig::with_cases(2))]
            #[should_panic]
        #[test]
        /// Check that the `SpendProof` verification fails when the diversified address is wrong.
        fn spend_proof_verification_diversified_address_integrity_failure(seed_phrase_randomness in any::<[u8; 32]>(), incorrect_seed_phrase_randomness in any::<[u8; 32]>(), spend_auth_randomizer in fr_strategy(), value_amount in 2..200u64, v_blinding in fr_strategy()) {
            let mut rng = OsRng;
            let (pk, vk) = generate_prepared_test_parameters::<SpendCircuit>(&mut rng);

            let seed_phrase = SeedPhrase::from_randomness(&seed_phrase_randomness);
            let sk_sender = SpendKey::from_seed_phrase_bip44(seed_phrase, &Bip44Path::new(0));

            let wrong_seed_phrase = SeedPhrase::from_randomness(&incorrect_seed_phrase_randomness);
            let wrong_sk_sender = SpendKey::from_seed_phrase_bip44(wrong_seed_phrase, &Bip44Path::new(0));
            let wrong_fvk_sender = wrong_sk_sender.full_viewing_key();
            let wrong_ivk_sender = wrong_fvk_sender.incoming();
            let (wrong_sender, _dtk_d) = wrong_ivk_sender.payment_address(1u32.into());

            let value_to_send = Value {
                amount: value_amount.into(),
                asset_id: asset::Cache::with_known_assets().get_unit("upenumbra").unwrap().id(),
            };

            let note = Note::generate(&mut rng, &wrong_sender, value_to_send);

            let note_commitment = note.commit();
            let rsk = sk_sender.spend_auth_key().randomize(&spend_auth_randomizer);
            let nk = *sk_sender.nullifier_key();
            let ak = sk_sender.spend_auth_key().into();
            let mut sct = tct::Tree::new();
            sct.insert(tct::Witness::Keep, note_commitment).unwrap();
            let anchor = sct.root();
            let state_commitment_proof = sct.witness(note_commitment).unwrap();
            let balance_commitment = value_to_send.commit(v_blinding);
            let rk: VerificationKey<SpendAuth> = rsk.into();
            let nf = Nullifier::derive(&nk, 0.into(), &note_commitment);

            // Note that this will blow up in debug mode as the constraint
            // system is unsatisified (ark-groth16 has a debug check for this).
            // In release mode the proof will be created, but will fail to verify.
            let blinding_r = Fq::rand(&mut OsRng);
            let blinding_s = Fq::rand(&mut OsRng);
            let public = SpendProofPublic {
                anchor,
                balance_commitment,
                nullifier: nf,
                rk,
            };
            let private = SpendProofPrivate {
                state_commitment_proof,
                note,
                v_blinding,
                spend_auth_randomizer,
                ak,
                nk,
            };
            let proof = SpendProof::prove(
                blinding_r,
                blinding_s,
                &pk,
                public.clone(),
                private
            ).expect("can create proof in release mode");

            proof.verify(&vk, public).expect("boom");
        }
    }

    proptest! {
            #![proptest_config(ProptestConfig::with_cases(2))]
        #[test]
        /// Check that the `SpendProof` verification fails, when using an
        /// incorrect nullifier.
        fn spend_proof_verification_nullifier_integrity_failure(seed_phrase_randomness in any::<[u8; 32]>(), spend_auth_randomizer in fr_strategy(), value_amount in 2..200u64, v_blinding in fr_strategy()) {
            let mut rng = OsRng;
            let (pk, vk) = generate_prepared_test_parameters::<SpendCircuit>(&mut rng);

            let seed_phrase = SeedPhrase::from_randomness(&seed_phrase_randomness);
            let sk_sender = SpendKey::from_seed_phrase_bip44(seed_phrase, &Bip44Path::new(0));
            let fvk_sender = sk_sender.full_viewing_key();
            let ivk_sender = fvk_sender.incoming();
            let (sender, _dtk_d) = ivk_sender.payment_address(0u32.into());

            let value_to_send = Value {
                amount: value_amount.into(),
                asset_id: asset::Cache::with_known_assets().get_unit("upenumbra").unwrap().id(),
            };

            let note = Note::generate(&mut rng, &sender, value_to_send);
            let note_commitment = note.commit();
            let rsk = sk_sender.spend_auth_key().randomize(&spend_auth_randomizer);
            let nk = *sk_sender.nullifier_key();
            let ak = sk_sender.spend_auth_key().into();
            let mut sct = tct::Tree::new();
            sct.insert(tct::Witness::Keep, note_commitment).unwrap();
            let anchor = sct.root();
            let state_commitment_proof = sct.witness(note_commitment).unwrap();
            let balance_commitment = value_to_send.commit(v_blinding);
            let rk: VerificationKey<SpendAuth> = rsk.into();
            let nf = Nullifier::derive(&nk, 0.into(), &note_commitment);

            let incorrect_nf = Nullifier::derive(&nk, 5.into(), &note_commitment);

            let blinding_r = Fq::rand(&mut OsRng);
            let blinding_s = Fq::rand(&mut OsRng);
            let public = SpendProofPublic {
                anchor,
                balance_commitment,
                nullifier: nf,
                rk,
            };
            let private = SpendProofPrivate {
                state_commitment_proof,
                note,
                v_blinding,
                spend_auth_randomizer,
                ak,
                nk,
            };
            let proof = SpendProof::prove(
                blinding_r,
                blinding_s,
                &pk,
                public, private
            )
            .expect("can create proof");

            let incorrect_public = SpendProofPublic {
                anchor,
                balance_commitment,
                nullifier: incorrect_nf,
                rk,
            };
            let proof_result = proof.verify(&vk, incorrect_public);
            assert!(proof_result.is_err());
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(2))]
    #[test]
    /// Check that the `SpendProof` verification fails when using balance
    /// commitments with different blinding factors.
    fn spend_proof_verification_balance_commitment_integrity_failure(seed_phrase_randomness in any::<[u8; 32]>(), spend_auth_randomizer in fr_strategy(), value_amount in 2..200u64, v_blinding in fr_strategy(), incorrect_blinding_factor in fr_strategy()) {
        let mut rng = OsRng;
        let (pk, vk) = generate_prepared_test_parameters::<SpendCircuit>(&mut rng);

        let seed_phrase = SeedPhrase::from_randomness(&seed_phrase_randomness);
        let sk_sender = SpendKey::from_seed_phrase_bip44(seed_phrase, &Bip44Path::new(0));
        let fvk_sender = sk_sender.full_viewing_key();
        let ivk_sender = fvk_sender.incoming();
        let (sender, _dtk_d) = ivk_sender.payment_address(0u32.into());

        let value_to_send = Value {
            amount: value_amount.into(),
            asset_id: asset::Cache::with_known_assets().get_unit("upenumbra").unwrap().id(),
        };

        let note = Note::generate(&mut rng, &sender, value_to_send);
        let note_commitment = note.commit();
        let rsk = sk_sender.spend_auth_key().randomize(&spend_auth_randomizer);
        let nk = *sk_sender.nullifier_key();
        let ak = sk_sender.spend_auth_key().into();
        let mut sct = tct::Tree::new();
        sct.insert(tct::Witness::Keep, note_commitment).unwrap();
        let anchor = sct.root();
        let state_commitment_proof = sct.witness(note_commitment).unwrap();
        let balance_commitment = value_to_send.commit(v_blinding);
        let rk: VerificationKey<SpendAuth> = rsk.into();
        let nf = Nullifier::derive(&nk, 0.into(), &note_commitment);

        let blinding_r = Fq::rand(&mut OsRng);
        let blinding_s = Fq::rand(&mut OsRng);
        let public = SpendProofPublic {
            anchor,
            balance_commitment,
            nullifier: nf,
            rk,
        };
        let private = SpendProofPrivate {
            state_commitment_proof,
            note,
            v_blinding,
            spend_auth_randomizer,
            ak,
            nk,
        };
        let proof = SpendProof::prove(
            blinding_r,
            blinding_s,
            &pk,
            public, private

        )
        .expect("can create proof");

        let incorrect_balance_commitment = value_to_send.commit(incorrect_blinding_factor);
        let incorrect_public = SpendProofPublic {
            anchor,
            balance_commitment: incorrect_balance_commitment,
            nullifier: nf,
            rk,
        };

        let proof_result = proof.verify(&vk, incorrect_public);
        assert!(proof_result.is_err());
    }
    }

    proptest! {
            #![proptest_config(ProptestConfig::with_cases(2))]
        #[test]
        /// Check that the `SpendProof` verification fails when the incorrect randomizable verification key is used.
        fn spend_proof_verification_fails_rk_integrity(seed_phrase_randomness in any::<[u8; 32]>(), spend_auth_randomizer in fr_strategy(), value_amount in 2..200u64, v_blinding in fr_strategy(), incorrect_spend_auth_randomizer in fr_strategy()) {
            let mut rng = OsRng;
            let (pk, vk) = generate_prepared_test_parameters::<SpendCircuit>(&mut rng);

            let seed_phrase = SeedPhrase::from_randomness(&seed_phrase_randomness);
            let sk_sender = SpendKey::from_seed_phrase_bip44(seed_phrase, &Bip44Path::new(0));
            let fvk_sender = sk_sender.full_viewing_key();
            let ivk_sender = fvk_sender.incoming();
            let (sender, _dtk_d) = ivk_sender.payment_address(0u32.into());

            let value_to_send = Value {
                amount: value_amount.into(),
                asset_id: asset::Cache::with_known_assets().get_unit("upenumbra").unwrap().id(),
            };

            let note = Note::generate(&mut rng, &sender, value_to_send);
            let note_commitment = note.commit();
            let rsk = sk_sender.spend_auth_key().randomize(&spend_auth_randomizer);
            let nk = *sk_sender.nullifier_key();
            let ak = sk_sender.spend_auth_key().into();
            let mut sct = tct::Tree::new();
            sct.insert(tct::Witness::Keep, note_commitment).unwrap();
            let anchor = sct.root();
            let state_commitment_proof = sct.witness(note_commitment).unwrap();
            let balance_commitment = value_to_send.commit(v_blinding);
            let rk: VerificationKey<SpendAuth> = rsk.into();
            let nf = Nullifier::derive(&nk, 0.into(), &note_commitment);

            let incorrect_rsk = sk_sender
                .spend_auth_key()
                .randomize(&incorrect_spend_auth_randomizer);
            let incorrect_rk: VerificationKey<SpendAuth> = incorrect_rsk.into();

            let blinding_r = Fq::rand(&mut OsRng);
            let blinding_s = Fq::rand(&mut OsRng);
            let public = SpendProofPublic {
                anchor,
                balance_commitment,
                nullifier: nf,
                rk,
            };
            let private = SpendProofPrivate {
                state_commitment_proof,
                note,
                v_blinding,
                spend_auth_randomizer,
                ak,
                nk,
            };
            let proof = SpendProof::prove(
                blinding_r,
                blinding_s,
                &pk,
                public, private
            )
            .expect("should be able to form proof");

            let incorrect_public = SpendProofPublic {
                anchor,
                balance_commitment,
                nullifier: nf,
                rk: incorrect_rk,
            };
            let proof_result = proof.verify(&vk, incorrect_public);
            assert!(proof_result.is_err());
        }
    }

    proptest! {
            #![proptest_config(ProptestConfig::with_cases(2))]
        #[test]
        /// Check that the `SpendProof` verification always suceeds for dummy (zero value) spends.
        fn spend_proof_dummy_verification_suceeds(seed_phrase_randomness in any::<[u8; 32]>(), spend_auth_randomizer in fr_strategy(), v_blinding in fr_strategy()) {
            let mut rng = OsRng;
            let (pk, vk) = generate_prepared_test_parameters::<SpendCircuit>(&mut rng);

            let seed_phrase = SeedPhrase::from_randomness(&seed_phrase_randomness);
            let sk_sender = SpendKey::from_seed_phrase_bip44(seed_phrase, &Bip44Path::new(0));
            let fvk_sender = sk_sender.full_viewing_key();
            let ivk_sender = fvk_sender.incoming();
            let (sender, _dtk_d) = ivk_sender.payment_address(0u32.into());

            let value_to_send = Value {
                amount: 0u64.into(),
                asset_id: asset::Cache::with_known_assets().get_unit("upenumbra").unwrap().id(),
            };

            let note = Note::generate(&mut rng, &sender, value_to_send);
            let note_commitment = note.commit();
            let rsk = sk_sender.spend_auth_key().randomize(&spend_auth_randomizer);
            let nk = *sk_sender.nullifier_key();
            let ak = sk_sender.spend_auth_key().into();
            let sct = tct::Tree::new();
            let anchor = sct.root();
            let state_commitment_proof = tct::Proof::dummy(&mut OsRng, note_commitment);
            // Using a random blinding factor here, but the proof will verify
            // since for dummies we only check if the value is zero, and choose
            // not to enforce the other equality constraint.
            let balance_commitment = value_to_send.commit(v_blinding);
            let rk: VerificationKey<SpendAuth> = rsk.into();
            let nf = Nullifier::derive(&nk, 0.into(), &note_commitment);

            let blinding_r = Fq::rand(&mut OsRng);
            let blinding_s = Fq::rand(&mut OsRng);
            let public = SpendProofPublic {
                anchor,
                balance_commitment,
                nullifier: nf,
                rk,
            };
            let private = SpendProofPrivate {
                state_commitment_proof,
                note,
                v_blinding,
                spend_auth_randomizer,
                ak,
                nk,
            };
            let proof = SpendProof::prove(
                blinding_r,
                blinding_s,
                &pk,
                public.clone(), private
            )
            .expect("should be able to form proof");

            let proof_result = proof.verify(&vk, public);
            assert!(proof_result.is_ok());
        }
    }

    struct MerkleProofCircuit {
        /// Witness: Inclusion proof for the note commitment.
        state_commitment_proof: tct::Proof,
        /// Public input: The merkle root of the state commitment tree
        pub anchor: tct::Root,
        pub epoch: Fq,
        pub block: Fq,
        pub commitment_index: Fq,
    }

    impl ConstraintSynthesizer<Fq> for MerkleProofCircuit {
        fn generate_constraints(
            self,
            cs: ConstraintSystemRef<Fq>,
        ) -> ark_relations::r1cs::Result<()> {
            // public inputs
            let anchor_var = FqVar::new_input(cs.clone(), || Ok(Fq::from(self.anchor)))?;
            let epoch_var = FqVar::new_input(cs.clone(), || Ok(self.epoch))?;
            let block_var = FqVar::new_input(cs.clone(), || Ok(self.block))?;
            let commitment_index_var = FqVar::new_input(cs.clone(), || Ok(self.commitment_index))?;

            // witnesses
            let merkle_path_var = tct::r1cs::MerkleAuthPathVar::new_witness(cs.clone(), || {
                Ok(self.state_commitment_proof.clone())
            })?;
            let claimed_note_commitment = StateCommitmentVar::new_witness(cs.clone(), || {
                Ok(self.state_commitment_proof.commitment())
            })?;
            let position_var = tct::r1cs::PositionVar::new_witness(cs.clone(), || {
                Ok(self.state_commitment_proof.position())
            })?;
            let position_bits = position_var.to_bits_le()?;
            merkle_path_var.verify(
                cs,
                &Boolean::TRUE,
                &position_bits,
                anchor_var,
                claimed_note_commitment.inner(),
            )?;

            // Now also verify the commitment index, block, and epoch numbers are all valid. This is not necessary
            // for Merkle proofs in general, but is here to ensure this code is exercised in tests.
            let computed_epoch = position_var.epoch()?;
            let computed_block = position_var.block()?;
            let computed_commitment_index = position_var.commitment()?;
            computed_epoch.enforce_equal(&epoch_var)?;
            computed_block.enforce_equal(&block_var)?;
            computed_commitment_index.enforce_equal(&commitment_index_var)?;
            Ok(())
        }
    }

    impl DummyWitness for MerkleProofCircuit {
        fn with_dummy_witness() -> Self {
            let seed_phrase = SeedPhrase::from_randomness(&[b'f'; 32]);
            let sk_sender = SpendKey::from_seed_phrase_bip44(seed_phrase, &Bip44Path::new(0));
            let fvk_sender = sk_sender.full_viewing_key();
            let ivk_sender = fvk_sender.incoming();
            let (address, _dtk_d) = ivk_sender.payment_address(0u32.into());

            let note = Note::from_parts(
                address,
                Value::from_str("1upenumbra").expect("valid value"),
                Rseed([1u8; 32]),
            )
            .expect("can make a note");
            let mut sct = tct::Tree::new();
            let note_commitment = note.commit();
            sct.insert(tct::Witness::Keep, note_commitment)
                .expect("able to insert note commitment into SCT");
            let anchor = sct.root();
            let state_commitment_proof = sct
                .witness(note_commitment)
                .expect("able to witness just-inserted note commitment");
            let position = state_commitment_proof.position();
            let epoch = Fq::from(position.epoch());
            let block = Fq::from(position.block());
            let commitment_index = Fq::from(position.commitment());

            Self {
                state_commitment_proof,
                anchor,
                epoch,
                block,
                commitment_index,
            }
        }
    }

    fn make_random_note_commitment(address: Address) -> StateCommitment {
        let note = Note::from_parts(
            address,
            Value::from_str("1upenumbra").expect("valid value"),
            Rseed([1u8; 32]),
        )
        .expect("can make a note");
        note.commit()
    }

    #[test]
    fn merkle_proof_verification_succeeds() {
        let mut rng = OsRng;
        let (pk, vk) = generate_prepared_test_parameters::<MerkleProofCircuit>(&mut rng);

        let seed_phrase = SeedPhrase::from_randomness(&[b'f'; 32]);
        let sk_sender = SpendKey::from_seed_phrase_bip44(seed_phrase, &Bip44Path::new(0));
        let fvk_sender = sk_sender.full_viewing_key();
        let ivk_sender = fvk_sender.incoming();
        let (address, _dtk_d) = ivk_sender.payment_address(0u32.into());
        // We will incrementally add notes to the state commitment tree, checking the merkle proofs verify
        // at each step.
        let mut sct = tct::Tree::new();

        for _ in 0..5 {
            let note_commitment = make_random_note_commitment(address);
            sct.insert(tct::Witness::Keep, note_commitment).unwrap();
            let anchor = sct.root();
            let state_commitment_proof = sct.witness(note_commitment).unwrap();
            let position = state_commitment_proof.position();
            let epoch = Fq::from(position.epoch());
            let block = Fq::from(position.block());
            let commitment_index = Fq::from(position.commitment());
            let circuit = MerkleProofCircuit {
                state_commitment_proof,
                anchor,
                epoch,
                block,
                commitment_index,
            };
            let proof = Groth16::<Bls12_377, LibsnarkReduction>::prove(&pk, circuit, &mut rng)
                .expect("should be able to form proof");

            let proof_result = Groth16::<Bls12_377, LibsnarkReduction>::verify_with_processed_vk(
                &vk,
                &[Fq::from(anchor), epoch, block, commitment_index],
                &proof,
            );
            assert!(proof_result.is_ok());
        }

        sct.end_block().expect("can end block");
        for _ in 0..100 {
            let note_commitment = make_random_note_commitment(address);
            sct.insert(tct::Witness::Forget, note_commitment).unwrap();
        }

        for _ in 0..5 {
            let note_commitment = make_random_note_commitment(address);
            sct.insert(tct::Witness::Keep, note_commitment).unwrap();
            let anchor = sct.root();
            let state_commitment_proof = sct.witness(note_commitment).unwrap();
            let position = state_commitment_proof.position();
            let epoch = Fq::from(position.epoch());
            let block = Fq::from(position.block());
            let commitment_index = Fq::from(position.commitment());
            let circuit = MerkleProofCircuit {
                state_commitment_proof,
                anchor,
                epoch,
                block,
                commitment_index,
            };
            let proof = Groth16::<Bls12_377, LibsnarkReduction>::prove(&pk, circuit, &mut rng)
                .expect("should be able to form proof");

            let proof_result = Groth16::<Bls12_377, LibsnarkReduction>::verify_with_processed_vk(
                &vk,
                &[Fq::from(anchor), epoch, block, commitment_index],
                &proof,
            );
            assert!(proof_result.is_ok());
        }

        sct.end_epoch().expect("can end epoch");
        for _ in 0..100 {
            let note_commitment = make_random_note_commitment(address);
            sct.insert(tct::Witness::Forget, note_commitment).unwrap();
        }

        for _ in 0..5 {
            let note_commitment = make_random_note_commitment(address);
            sct.insert(tct::Witness::Keep, note_commitment).unwrap();
            let anchor = sct.root();
            let state_commitment_proof = sct.witness(note_commitment).unwrap();
            let position = state_commitment_proof.position();
            let epoch = Fq::from(position.epoch());
            let block = Fq::from(position.block());
            let commitment_index = Fq::from(position.commitment());
            let circuit = MerkleProofCircuit {
                state_commitment_proof,
                anchor,
                epoch,
                block,
                commitment_index,
            };
            let proof = Groth16::<Bls12_377, LibsnarkReduction>::prove(&pk, circuit, &mut rng)
                .expect("should be able to form proof");

            let proof_result = Groth16::<Bls12_377, LibsnarkReduction>::verify_with_processed_vk(
                &vk,
                &[Fq::from(anchor), epoch, block, commitment_index],
                &proof,
            );
            assert!(proof_result.is_ok());
        }
    }
}
