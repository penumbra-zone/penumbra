use std::str::FromStr;

use ark_groth16::r1cs_to_qap::LibsnarkReduction;
use ark_r1cs_std::uint64::UInt64;
use ark_r1cs_std::{prelude::*, uint8::UInt8};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use decaf377::FieldExt;
use decaf377::{r1cs::FqVar, Bls12_377, Fq, Fr};

use ark_ff::ToConstraintField;
use ark_groth16::{Groth16, PreparedVerifyingKey, Proof, ProvingKey, VerifyingKey};
use ark_r1cs_std::prelude::AllocVar;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef};
use ark_snark::SNARK;
use decaf377_rdsa::{SpendAuth, VerificationKey};
use penumbra_proto::{core::crypto::v1alpha1 as pb, DomainType};
use penumbra_tct as tct;
use rand::{CryptoRng, Rng};
use rand_core::OsRng;
use tct::r1cs::PositionVar;

use crate::proofs::groth16::{gadgets, ParameterSetup, VerifyingKeyExt};
use crate::{
    balance,
    balance::commitment::BalanceCommitmentVar,
    keys::{
        AuthorizationKeyVar, IncomingViewingKeyVar, NullifierKey, NullifierKeyVar,
        RandomizedVerificationKey, SeedPhrase, SpendAuthRandomizerVar, SpendKey,
    },
    note,
    nullifier::NullifierVar,
    Note, Nullifier, Rseed, Value,
};

use super::GROTH16_PROOF_LENGTH_BYTES;

/// Groth16 proof for delegator voting.
#[derive(Clone, Debug)]
pub struct DelegatorVoteCircuit {
    // Witnesses
    /// Inclusion proof for the note commitment.
    state_commitment_proof: tct::Proof,
    /// The note being spent.
    note: Note,
    /// The blinding factor used for generating the value commitment.
    v_blinding: Fr,
    /// The randomizer used for generating the randomized spend auth key.
    spend_auth_randomizer: Fr,
    /// The spend authorization key.
    ak: VerificationKey<SpendAuth>,
    /// The nullifier deriving key.
    nk: NullifierKey,

    // Public inputs
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

impl DelegatorVoteCircuit {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        state_commitment_proof: tct::Proof,
        note: Note,
        v_blinding: Fr,
        spend_auth_randomizer: Fr,
        ak: VerificationKey<SpendAuth>,
        nk: NullifierKey,
        anchor: tct::Root,
        balance_commitment: balance::Commitment,
        nullifier: Nullifier,
        rk: VerificationKey<SpendAuth>,
        start_position: tct::Position,
    ) -> Self {
        Self {
            state_commitment_proof,
            note,
            v_blinding,
            spend_auth_randomizer,
            ak,
            nk,
            anchor,
            balance_commitment,
            nullifier,
            rk,
            start_position,
        }
    }
}

impl ConstraintSynthesizer<Fq> for DelegatorVoteCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fq>) -> ark_relations::r1cs::Result<()> {
        // Witnesses
        let note_var = note::NoteVar::new_witness(cs.clone(), || Ok(self.note.clone()))?;
        let claimed_note_commitment = note::StateCommitmentVar::new_witness(cs.clone(), || {
            Ok(self.state_commitment_proof.commitment())
        })?;

        let position_var = tct::r1cs::PositionVar::new_witness(cs.clone(), || {
            Ok(self.state_commitment_proof.position())
        })?;
        let position_u64: u64 = self.state_commitment_proof.position().into();
        let position_u64_var = UInt64::new_witness(cs.clone(), || Ok(position_u64))?;
        let position_bits = position_u64_var.to_bits_le();
        let merkle_path_var = tct::r1cs::MerkleAuthPathVar::new_witness(cs.clone(), || {
            Ok(self.state_commitment_proof)
        })?;

        let v_blinding_arr: [u8; 32] = self.v_blinding.to_bytes();
        let v_blinding_vars = UInt8::new_witness_vec(cs.clone(), &v_blinding_arr)?;

        let spend_auth_randomizer_var =
            SpendAuthRandomizerVar::new_witness(cs.clone(), || Ok(self.spend_auth_randomizer))?;
        let ak_element_var: AuthorizationKeyVar =
            AuthorizationKeyVar::new_witness(cs.clone(), || Ok(self.ak))?;
        let nk_var = NullifierKeyVar::new_witness(cs.clone(), || Ok(self.nk))?;

        // Public inputs
        let anchor_var = FqVar::new_input(cs.clone(), || Ok(Fq::from(self.anchor)))?;
        let claimed_balance_commitment_var =
            BalanceCommitmentVar::new_input(cs.clone(), || Ok(self.balance_commitment))?;
        let claimed_nullifier_var = NullifierVar::new_input(cs.clone(), || Ok(self.nullifier))?;
        let rk_var = RandomizedVerificationKey::new_input(cs.clone(), || Ok(self.rk.clone()))?;
        let start_position = PositionVar::new_input(cs.clone(), || Ok(self.start_position))?;

        // Note commitment integrity.
        let note_commitment_var = note_var.commit()?;
        note_commitment_var.enforce_equal(&claimed_note_commitment)?;

        // Nullifier integrity.
        let nullifier_var = nk_var.derive_nullifier(&position_var, &claimed_note_commitment)?;
        nullifier_var.enforce_equal(&claimed_nullifier_var)?;

        // Merkle auth path verification against the provided anchor.
        merkle_path_var.verify(
            cs.clone(),
            &Boolean::TRUE,
            &position_bits,
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
        gadgets::element_not_identity(
            cs.clone(),
            &Boolean::TRUE,
            note_var.diversified_generator(),
        )?;
        gadgets::element_not_identity(cs, &Boolean::TRUE, ak_element_var.inner)?;

        // Additionally, check that the start position has a zero commitment index, since this is
        // the only sensible start time for a vote.
        let zero_constant = FqVar::constant(Fq::from(0u64));
        start_position.commitment()?.enforce_equal(&zero_constant)?;

        // Additionally, check that the position of the spend proof is before the start
        // start_height, which ensures that the note being voted with was created before voting
        // started.
        position_var
            .inner
            .enforce_cmp(&start_position.inner, core::cmp::Ordering::Less, false)?;

        Ok(())
    }
}

impl ParameterSetup for DelegatorVoteCircuit {
    fn generate_test_parameters() -> (ProvingKey<Bls12_377>, VerifyingKey<Bls12_377>) {
        let seed_phrase = SeedPhrase::from_randomness([b'f'; 32]);
        let sk_sender = SpendKey::from_seed_phrase(seed_phrase, 0);
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
        let note_commitment = note.commit();
        sct.insert(tct::Witness::Keep, note_commitment).unwrap();
        let anchor = sct.root();
        let state_commitment_proof = sct.witness(note_commitment).unwrap();
        let start_position = state_commitment_proof.position();

        let circuit = DelegatorVoteCircuit {
            state_commitment_proof,
            note,
            v_blinding,
            spend_auth_randomizer,
            ak,
            nk,
            anchor,
            balance_commitment: balance::Commitment(decaf377::basepoint()),
            nullifier,
            rk,
            start_position,
        };
        let (pk, vk) =
            Groth16::<Bls12_377, LibsnarkReduction>::circuit_specific_setup(circuit, &mut OsRng)
                .expect("can perform circuit specific setup");
        (pk, vk)
    }
}

#[derive(Clone, Debug)]
pub struct DelegatorVoteProof(Proof<Bls12_377>);

impl DelegatorVoteProof {
    #![allow(clippy::too_many_arguments)]
    pub fn prove<R: CryptoRng + Rng>(
        rng: &mut R,
        pk: &ProvingKey<Bls12_377>,
        state_commitment_proof: tct::Proof,
        note: Note,
        spend_auth_randomizer: Fr,
        ak: VerificationKey<SpendAuth>,
        nk: NullifierKey,
        anchor: tct::Root,
        balance_commitment: balance::Commitment,
        nullifier: Nullifier,
        rk: VerificationKey<SpendAuth>,
        start_position: tct::Position,
    ) -> anyhow::Result<Self> {
        // The blinding factor for the value commitment is zero since it
        // is not blinded.
        let zero_blinding = Fr::from(0);
        let circuit = DelegatorVoteCircuit {
            state_commitment_proof,
            note,
            v_blinding: zero_blinding,
            spend_auth_randomizer,
            ak,
            nk,
            anchor,
            balance_commitment,
            nullifier,
            rk,
            start_position,
        };
        let proof = Groth16::<Bls12_377, LibsnarkReduction>::prove(pk, circuit, rng)
            .map_err(|err| anyhow::anyhow!(err))?;
        Ok(Self(proof))
    }

    /// Called to verify the proof using the provided public inputs.
    // For debugging proof verification failures,
    // to check that the proof data and verification keys are consistent.
    #[tracing::instrument(level="debug", skip(self, vk), fields(self = ?base64::encode(&self.clone().encode_to_vec()), vk = ?vk.debug_id()))]
    pub fn verify(
        &self,
        vk: &PreparedVerifyingKey<Bls12_377>,
        anchor: tct::Root,
        balance_commitment: balance::Commitment,
        nullifier: Nullifier,
        rk: VerificationKey<SpendAuth>,
        start_position: tct::Position,
    ) -> anyhow::Result<()> {
        let mut public_inputs = Vec::new();
        public_inputs.extend(Fq::from(anchor.0).to_field_elements().unwrap());
        public_inputs.extend(balance_commitment.0.to_field_elements().unwrap());
        public_inputs.extend(nullifier.0.to_field_elements().unwrap());
        let element_rk = decaf377::Encoding(rk.to_bytes())
            .vartime_decompress()
            .expect("expect only valid element points");
        public_inputs.extend(element_rk.to_field_elements().unwrap());
        public_inputs.extend(
            Fq::from(u64::from(start_position))
                .to_field_elements()
                .unwrap(),
        );

        tracing::trace!(?public_inputs);
        let start = std::time::Instant::now();
        let proof_result = Groth16::<Bls12_377, LibsnarkReduction>::verify_with_processed_vk(
            &vk,
            public_inputs.as_slice(),
            &self.0,
        )
        .map_err(|err| anyhow::anyhow!(err))?;
        tracing::debug!(?proof_result, elapsed = ?start.elapsed());
        proof_result
            .then_some(())
            .ok_or_else(|| anyhow::anyhow!("delegator vote proof did not verify"))
    }
}

impl DomainType for DelegatorVoteProof {
    type Proto = pb::ZkDelegatorVoteProof;
}

impl From<DelegatorVoteProof> for pb::ZkDelegatorVoteProof {
    fn from(proof: DelegatorVoteProof) -> Self {
        let mut proof_bytes = [0u8; GROTH16_PROOF_LENGTH_BYTES];
        Proof::serialize_compressed(&proof.0, &mut proof_bytes[..]).expect("can serialize Proof");
        pb::ZkDelegatorVoteProof {
            inner: proof_bytes.to_vec(),
        }
    }
}

impl TryFrom<pb::ZkDelegatorVoteProof> for DelegatorVoteProof {
    type Error = anyhow::Error;

    fn try_from(proto: pb::ZkDelegatorVoteProof) -> Result<Self, Self::Error> {
        Ok(DelegatorVoteProof(
            Proof::deserialize_compressed(&proto.inner[..]).map_err(|e| anyhow::anyhow!(e))?,
        ))
    }
}
