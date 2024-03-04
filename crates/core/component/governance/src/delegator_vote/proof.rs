use anyhow::Result;
use ark_ff::ToConstraintField;
use ark_groth16::{
    r1cs_to_qap::LibsnarkReduction, Groth16, PreparedVerifyingKey, Proof, ProvingKey,
};
use ark_r1cs_std::{prelude::*, uint8::UInt8};
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use ark_snark::SNARK;
use base64::{engine::general_purpose, Engine as _};
use decaf377::{
    r1cs::{ElementVar, FqVar},
    Bls12_377, Fq, Fr,
};
use decaf377_rdsa::{SpendAuth, VerificationKey};
use penumbra_asset::{
    balance::{self, commitment::BalanceCommitmentVar, Commitment},
    Value,
};
use penumbra_keys::keys::{
    AuthorizationKeyVar, Bip44Path, IncomingViewingKeyVar, NullifierKey, NullifierKeyVar,
    RandomizedVerificationKey, SeedPhrase, SpendAuthRandomizerVar, SpendKey,
};
use penumbra_proof_params::{DummyWitness, VerifyingKeyExt, GROTH16_PROOF_LENGTH_BYTES};
use penumbra_proto::{core::component::governance::v1 as pb, DomainType};
use penumbra_sct::{Nullifier, NullifierVar};
use penumbra_shielded_pool::{note, Note, Rseed};
use penumbra_tct::{
    self as tct,
    r1cs::{PositionVar, StateCommitmentVar},
    Root,
};
use std::str::FromStr;
use tap::Tap;

/// The public input for a [`DelegatorVoteProof`].
#[derive(Clone, Debug)]
pub struct DelegatorVoteProofPublic {
    /// the merkle root of the state commitment tree.
    pub anchor: tct::Root,
    /// value commitment of the note to be spent.
    pub balance_commitment: balance::Commitment,
    /// nullifier of the note to be spent.
    pub nullifier: Nullifier,
    /// the randomized verification spend key.
    pub rk: VerificationKey<SpendAuth>,
    /// the start position of the proposal being voted on.
    pub start_position: tct::Position,
}

/// The private input for a [`DelegatorVoteProof`].
#[derive(Clone, Debug)]
pub struct DelegatorVoteProofPrivate {
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
fn check_satisfaction(
    public: &DelegatorVoteProofPublic,
    private: &DelegatorVoteProofPrivate,
) -> Result<()> {
    use penumbra_keys::keys::FullViewingKey;

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

    if public.start_position.commitment() != 0 {
        anyhow::bail!("start position commitment index is not zero");
    }

    if private.state_commitment_proof.position() >= public.start_position {
        anyhow::bail!("note did not exist prior to the start of voting");
    }

    Ok(())
}

#[cfg(test)]
fn check_circuit_satisfaction(
    public: DelegatorVoteProofPublic,
    private: DelegatorVoteProofPrivate,
) -> Result<()> {
    use ark_relations::r1cs::{self, ConstraintSystem};

    let cs = ConstraintSystem::new_ref();
    let circuit = DelegatorVoteCircuit { public, private };
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

/// Groth16 proof for delegator voting.
#[derive(Clone, Debug)]
pub struct DelegatorVoteCircuit {
    public: DelegatorVoteProofPublic,
    private: DelegatorVoteProofPrivate,
}

impl ConstraintSynthesizer<Fq> for DelegatorVoteCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fq>) -> ark_relations::r1cs::Result<()> {
        // Witnesses
        let note_var = note::NoteVar::new_witness(cs.clone(), || Ok(self.private.note.clone()))?;
        let claimed_note_commitment = StateCommitmentVar::new_witness(cs.clone(), || {
            Ok(self.private.state_commitment_proof.commitment())
        })?;

        let delegator_position_var = tct::r1cs::PositionVar::new_witness(cs.clone(), || {
            Ok(self.private.state_commitment_proof.position())
        })?;
        let delegator_position_bits = delegator_position_var.to_bits_le()?;
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
        let start_position = PositionVar::new_input(cs.clone(), || Ok(self.public.start_position))?;

        // Note commitment integrity.
        let note_commitment_var = note_var.commit()?;
        note_commitment_var.enforce_equal(&claimed_note_commitment)?;

        // Nullifier integrity.
        let nullifier_var =
            NullifierVar::derive(&nk_var, &delegator_position_var, &claimed_note_commitment)?;
        nullifier_var.enforce_equal(&claimed_nullifier_var)?;

        // Merkle auth path verification against the provided anchor.
        merkle_path_var.verify(
            cs.clone(),
            &Boolean::TRUE,
            &delegator_position_bits,
            anchor_var,
            claimed_note_commitment.inner(),
        )?;

        // Check integrity of randomized verification key.
        let computed_rk_var = ak_element_var.randomize(&spend_auth_randomizer_var)?;
        computed_rk_var.enforce_equal(&rk_var)?;

        // Check integrity of diversified address.
        let ivk = IncomingViewingKeyVar::derive(&nk_var, &ak_element_var)?;
        let computed_transmission_key =
            ivk.diversified_public(&note_var.diversified_generator())?;
        computed_transmission_key.enforce_equal(&note_var.transmission_key())?;

        // Check integrity of balance commitment.
        let balance_commitment = note_var.value().commit(v_blinding_vars)?;
        balance_commitment.enforce_equal(&claimed_balance_commitment_var)?;

        // Check elements were not identity.
        let identity = ElementVar::new_constant(cs, decaf377::Element::default())?;
        identity.enforce_not_equal(&note_var.diversified_generator())?;
        identity.enforce_not_equal(&ak_element_var.inner)?;

        // Additionally, check that the start position has a zero commitment index, since this is
        // the only sensible start time for a vote.
        let zero_constant = FqVar::constant(Fq::from(0u64));
        let commitment = start_position.commitment()?;
        commitment.enforce_equal(&zero_constant)?;

        // Additionally, check that the position of the spend proof is before the start
        // start_height, which ensures that the note being voted with was created before voting
        // started.
        //
        // Also note that `FpVar::enforce_cmp` requires that the field elements have size
        // (p-1)/2, which is true for positions as they are 64 bits at most.
        //
        // This MUST be strict inequality (hence passing false to `should_also_check_equality`)
        // because you could delegate and vote on the proposal in the same block.
        delegator_position_var.position.enforce_cmp(
            &start_position.position,
            core::cmp::Ordering::Less,
            false,
        )?;

        Ok(())
    }
}

impl DummyWitness for DelegatorVoteCircuit {
    fn with_dummy_witness() -> Self {
        let seed_phrase = SeedPhrase::from_randomness(&[b'f'; 32]);
        let sk_sender = SpendKey::from_seed_phrase_bip44(seed_phrase, &Bip44Path::new(0));
        let fvk_sender = sk_sender.full_viewing_key();
        let ivk_sender = fvk_sender.incoming();
        let (address, _dtk_d) = ivk_sender.payment_address(0u32.into());

        let spend_auth_randomizer = Fr::from(1u64);
        let rsk = sk_sender.spend_auth_key().randomize(&spend_auth_randomizer);
        let nk = *sk_sender.nullifier_key();
        let ak = sk_sender.spend_auth_key().into();
        let note = Note::from_parts(
            address,
            Value::from_str("1upenumbra").expect("valid value"),
            Rseed([1u8; 32]),
        )
        .expect("can make a note");
        let v_blinding = Fr::from(1u64);
        let rk: VerificationKey<SpendAuth> = rsk.into();
        let nullifier = Nullifier(Fq::from(1u64));
        let mut sct = tct::Tree::new();
        let note_commitment = note.commit();
        sct.insert(tct::Witness::Keep, note_commitment)
            .expect("able to insert note commitment into SCT");
        let anchor = sct.root();
        let state_commitment_proof = sct
            .witness(note_commitment)
            .expect("able to witness just-inserted note commitment");
        let start_position = state_commitment_proof.position();

        let public = DelegatorVoteProofPublic {
            anchor,
            balance_commitment: balance::Commitment(decaf377::Element::GENERATOR),
            nullifier,
            rk,
            start_position,
        };
        let private = DelegatorVoteProofPrivate {
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

#[derive(Debug, thiserror::Error)]
pub enum VerificationError {
    #[error("error deserializing compressed proof: {0:?}")]
    ProofDeserialize(ark_serialize::SerializationError),
    #[error("Fq types are Bls12-377 field members")]
    Anchor,
    #[error("balance commitment is a Bls12-377 field member")]
    BalanceCommitment,
    #[error("nullifier is a Bls12-377 field member")]
    Nullifier,
    #[error("could not decompress element points: {0:?}")]
    DecompressRk(decaf377::EncodingError),
    #[error("randomized spend key is a Bls12-377 field member")]
    Rk,
    #[error("start position is a Bls12-377 field member")]
    StartPosition,
    #[error("error verifying proof: {0:?}")]
    SynthesisError(ark_relations::r1cs::SynthesisError),
    #[error("delegator vote proof did not verify")]
    InvalidProof,
}

#[derive(Clone, Debug, Copy)]
pub struct DelegatorVoteProof([u8; GROTH16_PROOF_LENGTH_BYTES]);

impl DelegatorVoteProof {
    pub fn prove(
        blinding_r: Fq,
        blinding_s: Fq,
        pk: &ProvingKey<Bls12_377>,
        public: DelegatorVoteProofPublic,
        private: DelegatorVoteProofPrivate,
    ) -> anyhow::Result<Self> {
        let circuit = DelegatorVoteCircuit { public, private };
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
    #[tracing::instrument(
        level="debug",
        skip(self, vk),
        fields(
            self = ?general_purpose::STANDARD.encode(self.clone().encode_to_vec()),
            vk = ?vk.debug_id()
        )
    )]
    pub fn verify(
        &self,
        vk: &PreparedVerifyingKey<Bls12_377>,
        DelegatorVoteProofPublic {
            anchor: Root(anchor),
            balance_commitment: Commitment(balance_commitment),
            nullifier: Nullifier(nullifier),
            rk,
            start_position,
        }: DelegatorVoteProofPublic,
    ) -> Result<(), VerificationError> {
        let proof = Proof::deserialize_compressed_unchecked(&self.0[..])
            .map_err(VerificationError::ProofDeserialize)?;
        let element_rk = decaf377::Encoding(rk.to_bytes())
            .vartime_decompress()
            .map_err(VerificationError::DecompressRk)?;

        /// Shorthand helper, convert expressions into field elements.
        macro_rules! to_field_elements {
            ($fe:expr, $err:expr) => {
                $fe.to_field_elements().ok_or($err)?
            };
        }

        use VerificationError::*;
        let public_inputs = [
            to_field_elements!(Fq::from(anchor), Anchor),
            to_field_elements!(balance_commitment, BalanceCommitment),
            to_field_elements!(nullifier, Nullifier),
            to_field_elements!(element_rk, Rk),
            to_field_elements!(start_position, StartPosition),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .tap(|public_inputs| tracing::trace!(?public_inputs));

        let start = std::time::Instant::now();
        Groth16::<Bls12_377, LibsnarkReduction>::verify_with_processed_vk(
            vk,
            public_inputs.as_slice(),
            &proof,
        )
        .map_err(VerificationError::SynthesisError)?
        .tap(|proof_result| tracing::debug!(?proof_result, elapsed = ?start.elapsed()))
        .then_some(())
        .ok_or(VerificationError::InvalidProof)
    }
}

impl DomainType for DelegatorVoteProof {
    type Proto = pb::ZkDelegatorVoteProof;
}

impl From<DelegatorVoteProof> for pb::ZkDelegatorVoteProof {
    fn from(proof: DelegatorVoteProof) -> Self {
        pb::ZkDelegatorVoteProof {
            inner: proof.0.to_vec(),
        }
    }
}

impl TryFrom<pb::ZkDelegatorVoteProof> for DelegatorVoteProof {
    type Error = anyhow::Error;

    fn try_from(proto: pb::ZkDelegatorVoteProof) -> Result<Self, Self::Error> {
        Ok(DelegatorVoteProof(proto.inner[..].try_into()?))
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use decaf377::{Fq, Fr};
    use penumbra_asset::{asset, Value};
    use penumbra_keys::keys::{SeedPhrase, SpendKey};
    use penumbra_num::Amount;
    use penumbra_sct::Nullifier;
    use proptest::prelude::*;

    fn fr_strategy() -> BoxedStrategy<Fr> {
        any::<[u8; 32]>()
            .prop_map(|bytes| Fr::from_le_bytes_mod_order(&bytes[..]))
            .boxed()
    }

    prop_compose! {
        fn arb_valid_delegator_vote_statement()(v_blinding in fr_strategy(), spend_auth_randomizer in fr_strategy(), asset_id64 in any::<u64>(), address_index in any::<u32>(), amount in any::<u64>(), seed_phrase_randomness in any::<[u8; 32]>(), rseed_randomness in any::<[u8; 32]>(), num_commitments in 0..100) -> (DelegatorVoteProofPublic, DelegatorVoteProofPrivate) {
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
                sct.insert(tct::Witness::Keep, dummy_note_commitment).expect("can insert note commitment into SCT");
            }

            sct.insert(tct::Witness::Keep, note_commitment).expect("can insert note commitment into SCT");
            let anchor = sct.root();
            let state_commitment_proof = sct.witness(note_commitment).expect("can witness note commitment");

            // All proposals should have a position commitment index of zero, so we need to end the epoch
            // and get the position that corresponds to the first commitment in the new epoch.
            sct.end_epoch().expect("should be able to end an epoch");
            let first_note_commitment = Note::from_parts(sender, value_to_send, Rseed([u8::MAX; 32])).expect("can create note").commit();
            sct.insert(tct::Witness::Keep, first_note_commitment).expect("can insert note commitment into SCT");
            let start_position = sct.witness(first_note_commitment).expect("can witness note commitment").position();

            let balance_commitment = value_to_send.commit(v_blinding);
            let rk: VerificationKey<SpendAuth> = rsk.into();
            let nullifier = Nullifier::derive(&nk, state_commitment_proof.position(), &note_commitment);

            let public = DelegatorVoteProofPublic {
                anchor,
                balance_commitment,
                nullifier,
                rk,
                start_position,
            };
            let private = DelegatorVoteProofPrivate {
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
        fn delegator_vote_happy_path((public, private) in arb_valid_delegator_vote_statement()) {
            assert!(check_satisfaction(&public, &private).is_ok());
            assert!(check_circuit_satisfaction(public, private).is_ok());
        }
    }

    prop_compose! {
        // This strategy generates a delegator vote statement that votes on a proposal with
        // a non-zero position commitment index. The circuit should be unsatisfiable in this case.
        fn arb_invalid_delegator_vote_statement_nonzero_start()(v_blinding in fr_strategy(), spend_auth_randomizer in fr_strategy(), asset_id64 in any::<u64>(), address_index in any::<u32>(), amount in any::<u64>(), seed_phrase_randomness in any::<[u8; 32]>(), rseed_randomness in any::<[u8; 32]>(), num_commitments in 0..100) -> (DelegatorVoteProofPublic, DelegatorVoteProofPrivate) {
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
                sct.insert(tct::Witness::Keep, dummy_note_commitment).expect("can insert note commitment into SCT");
            }

            sct.insert(tct::Witness::Keep, note_commitment).expect("can insert note commitment into SCT");
            let anchor = sct.root();
            let state_commitment_proof = sct.witness(note_commitment).expect("can witness note commitment");

            let rseed = Rseed([num_commitments as u8; 32]);
            let not_first_note_commitment = Note::from_parts(sender, value_to_send, rseed).expect("can create note").commit();
            sct.insert(tct::Witness::Keep, not_first_note_commitment).expect("can insert note commitment into SCT");
            // All proposals should have a position commitment index of zero, but this one will not, so
            // expect the proof to fail.
            let start_position = sct.witness(not_first_note_commitment).expect("can witness note commitment").position();

            let balance_commitment = value_to_send.commit(v_blinding);
            let rk: VerificationKey<SpendAuth> = rsk.into();
            let nullifier = Nullifier::derive(&nk, state_commitment_proof.position(), &note_commitment);

            let public = DelegatorVoteProofPublic {
                anchor,
                balance_commitment,
                nullifier,
                rk,
                start_position,
            };
            let private = DelegatorVoteProofPrivate {
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
        fn delegator_vote_invalid_start_position((public, private) in arb_invalid_delegator_vote_statement_nonzero_start()) {
            assert!(check_satisfaction(&public, &private).is_err());
            assert!(check_circuit_satisfaction(public, private).is_err());
        }
    }
}
