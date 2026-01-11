use base64::prelude::*;
use std::str::FromStr;
use tct::Root;

use anyhow::Result;
use ark_r1cs_std::{
    boolean::Boolean,
    prelude::{EqGadget, FieldVar},
    uint8::UInt8,
    ToBitsGadget,
};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use decaf377::{r1cs::FqVar, Bls12_377, Fq, Fr};

use ark_ff::ToConstraintField;
use ark_groth16::{
    r1cs_to_qap::LibsnarkReduction, Groth16, PreparedVerifyingKey, Proof, ProvingKey,
};
use ark_r1cs_std::prelude::AllocVar;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef};
use ark_snark::SNARK;
use decaf377_rdsa::{SpendAuth, VerificationKey};
use penumbra_sdk_proto::{penumbra::core::component::shielded_pool::v1 as pb, DomainType};
use penumbra_sdk_tct as tct;
use penumbra_sdk_tct::r1cs::StateCommitmentVar;

use crate::{note, Note, Rseed};
use decaf377::r1cs::ElementVar;
use penumbra_sdk_asset::{
    balance::commitment::BalanceCommitmentVar,
    balance::{self, Commitment},
    Value,
};
use penumbra_sdk_compliance::r1cs::{verify_compliance_integrity, ComplianceWitness};
use penumbra_sdk_keys::keys::{
    AuthorizationKeyVar, Bip44Path, IncomingViewingKeyVar, NullifierKey, NullifierKeyVar,
    RandomizedVerificationKey, SeedPhrase, SpendAuthRandomizerVar, SpendKey,
};
use penumbra_sdk_proof_params::{DummyWitness, VerifyingKeyExt, GROTH16_PROOF_LENGTH_BYTES};
use penumbra_sdk_sct::{Nullifier, NullifierVar};
use tap::Tap;

/// The public input for a [`SpendProof`].
#[derive(Clone, Debug)]
pub struct SpendProofPublic {
    /// the merkle root of the state commitment tree.
    pub anchor: tct::Root,
    /// balance commitment of the note to be spent.
    pub balance_commitment: balance::Commitment,
    /// nullifier of the note to be spent.
    pub nullifier: Nullifier,
    /// the randomized verification spend key.
    pub rk: VerificationKey<SpendAuth>,
    /// Asset registry Merkle root (asset regulation tree anchor)
    pub asset_anchor: tct::StateCommitment,
    /// User compliance registry Merkle root (user tree anchor)
    pub compliance_anchor: tct::StateCommitment,
    /// Ephemeral public key from the compliance ciphertext
    pub compliance_epk: decaf377::Element,
    /// Compliance ciphertext for regulated assets (packed as Fq elements for circuit input)
    pub compliance_ciphertext: Vec<Fq>,
    /// Target timestamp for key derivation (Unix timestamp in seconds)
    pub target_timestamp: u64,
    /// Hash of the sender's compliance leaf (for binding with output circuit)
    pub sender_leaf_hash: tct::StateCommitment,
    /// Hash of the counterparty's (receiver's) compliance leaf (for binding with output circuit)
    pub counterparty_leaf_hash: tct::StateCommitment,
}

/// The private input for a [`SpendProof`].
#[derive(Clone, Debug)]
pub struct SpendProofPrivate {
    /// Inclusion proof for the note commitment.
    pub state_commitment_proof: tct::Proof,
    /// The note being spent.
    pub note: Note,
    /// The blinding factor used for generating the balance commitment.
    pub v_blinding: Fr,
    /// The randomizer used for generating the randomized spend auth key.
    pub spend_auth_randomizer: Fr,
    /// The spend authorization key.
    pub ak: VerificationKey<SpendAuth>,
    /// The nullifier deriving key.
    pub nk: NullifierKey,
    /// Asset registry Merkle path proving asset regulation status
    pub asset_path: penumbra_sdk_compliance::MerklePath,
    /// Position of the asset in the asset registry QuadTree
    pub asset_position: u64,
    /// Whether this asset is regulated (requires compliance)
    pub is_regulated: bool,
    /// Compliance Merkle path proving user is in compliance registry
    pub compliance_path: penumbra_sdk_compliance::MerklePath,
    /// Position of the compliance leaf in the QuadTree
    pub compliance_position: u64,
    /// User's compliance leaf (address, ACK, asset_id) - replaces old CVK
    pub user_leaf: penumbra_sdk_compliance::ComplianceLeaf,
    /// Ephemeral secret used to encrypt the compliance ciphertext (needed by circuit for verification)
    pub compliance_ephemeral_secret: Fr,
    /// The counterparty's (receiver's) compliance leaf (for extracting address and binding)
    pub counterparty_leaf: penumbra_sdk_compliance::ComplianceLeaf,
    /// Shared transaction blinding nonce (same for spend and output in one transaction)
    pub tx_blinding_nonce: Fr,
}

#[cfg(test)]
fn check_satisfaction(public: &SpendProofPublic, private: &SpendProofPrivate) -> Result<()> {
    use penumbra_sdk_keys::keys::FullViewingKey;

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

    let amount_u128: u128 = private.note.value().amount.into();
    if amount_u128 != 0u128 {
        private.state_commitment_proof.verify(public.anchor)?;
    }

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

impl SpendCircuit {
    pub fn into_parts(self) -> (SpendProofPublic, SpendProofPrivate) {
        (self.public, self.private)
    }
}

impl ConstraintSynthesizer<Fq> for SpendCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fq>) -> ark_relations::r1cs::Result<()> {
        // === Witness Allocation ===
        // Note: in the allocation of the address on `NoteVar` we check the diversified base is not identity.
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
        // Note: in the allocation of `AuthorizationKeyVar` we check it is not identity.
        let ak_element_var: AuthorizationKeyVar =
            AuthorizationKeyVar::new_witness(cs.clone(), || Ok(self.private.ak))?;
        let nk_var = NullifierKeyVar::new_witness(cs.clone(), || Ok(self.private.nk))?;

        // === Public Input Allocation ===
        let anchor_var = FqVar::new_input(cs.clone(), || Ok(Fq::from(self.public.anchor)))?;
        let claimed_balance_commitment_var =
            BalanceCommitmentVar::new_input(cs.clone(), || Ok(self.public.balance_commitment))?;
        let claimed_nullifier_var =
            NullifierVar::new_input(cs.clone(), || Ok(self.public.nullifier))?;
        let rk_var = RandomizedVerificationKey::new_input(cs.clone(), || Ok(self.public.rk))?;
        let claimed_asset_anchor =
            StateCommitmentVar::new_input(cs.clone(), || Ok(self.public.asset_anchor))?;
        let claimed_compliance_anchor =
            StateCommitmentVar::new_input(cs.clone(), || Ok(self.public.compliance_anchor))?;

        // Allocate target date (Unix day index) as public input
        // Convert timestamp to day index (timestamp / SECONDS_PER_DAY)
        let target_day_index = self.public.target_timestamp / 86400;
        let target_date_var = FqVar::new_input(cs.clone(), || Ok(Fq::from(target_day_index)))?;

        // Allocate witness for asset regulation status
        let _is_regulated_var = Boolean::new_witness(cs.clone(), || Ok(self.private.is_regulated))?;

        // Allocate asset position
        let _asset_position_var =
            FqVar::new_witness(cs.clone(), || Ok(Fq::from(self.private.asset_position)))?;

        // Allocate compliance position
        let _compliance_position_var = FqVar::new_witness(cs.clone(), || {
            Ok(Fq::from(self.private.compliance_position))
        })?;

        // Allocate ephemeral public key as a separate public input
        let public_ciphertext_epk_var =
            ElementVar::new_input(cs.clone(), || Ok(self.public.compliance_epk))?;

        // Allocate compliance ciphertext as public inputs (packed field elements)
        let mut ciphertext_vars = Vec::new();
        for fq in self.public.compliance_ciphertext.iter() {
            ciphertext_vars.push(FqVar::new_input(cs.clone(), || Ok(*fq))?);
        }

        // === Spend-Specific Integrity Checks ===
        let note_commitment_var = note_var.commit()?;
        note_commitment_var.enforce_equal(&claimed_note_commitment)?;

        // Nullifier integrity.
        let nullifier_var = NullifierVar::derive(&nk_var, &position_var, &claimed_note_commitment)?;
        nullifier_var.enforce_equal(&claimed_nullifier_var)?;

        // Merkle auth path verification against the provided anchor.
        //
        // We short circuit the merkle path verification if the note is a _dummy_ spend (a spend
        // with zero value), since these are never committed to the state commitment tree.
        let is_not_dummy = note_var.amount().is_eq(&FqVar::zero())?.not();
        merkle_path_var.verify(
            cs.clone(),
            &is_not_dummy,
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

        // === Unified Compliance Verification (ACK System) ===

        // Allocate counterparty leaf as witness (needed for extracting counterparty address)
        let counterparty_leaf_var =
            penumbra_sdk_compliance::r1cs::ComplianceLeafVar::new_witness(cs.clone(), || {
                Ok(self.private.counterparty_leaf.clone())
            })?;

        // Create the compliance witness struct
        let compliance_witness = ComplianceWitness {
            is_regulated: self.private.is_regulated,
            asset_path: self.private.asset_path.clone(),
            asset_position: self.private.asset_position,
            compliance_path: self.private.compliance_path.clone(),
            compliance_position: self.private.compliance_position,
            user_leaf: self.private.user_leaf.clone(),
        };

        // Call the unified compliance integrity verification gadget
        verify_compliance_integrity(
            cs.clone(),
            // Public inputs
            claimed_asset_anchor.inner().clone(),
            claimed_compliance_anchor.inner().clone(),
            target_date_var,
            public_ciphertext_epk_var,
            ciphertext_vars,
            // Note data (individual components from note_var)
            note_var.asset_id(),
            note_var.amount(),
            note_var.diversified_generator(),
            note_var.transmission_key(),
            // Counterparty address components (from counterparty leaf)
            counterparty_leaf_var.address.diversified_generator.clone(),
            counterparty_leaf_var.address.transmission_key.clone(),
            // Witness data
            self.private.compliance_ephemeral_secret,
            compliance_witness,
        )?;

        // === Counterparty Leaf Binding ===

        // Allocate the shared transaction blinding nonce as a witness
        let tx_blinding_nonce_var = FqVar::new_witness(cs.clone(), || {
            Ok(Fq::from_le_bytes_mod_order(
                &self.private.tx_blinding_nonce.to_bytes(),
            ))
        })?;

        // Allocate sender's leaf as witness and compute its hash
        let sender_leaf_var =
            penumbra_sdk_compliance::r1cs::ComplianceLeafVar::new_witness(cs.clone(), || {
                Ok(self.private.user_leaf.clone())
            })?;
        let sender_leaf_hash = sender_leaf_var.commit(cs.clone())?;

        // Blind the sender leaf hash with the transaction nonce
        // Uses the shared leaf_binding module to ensure consistency
        let computed_blinded_sender =
            penumbra_sdk_compliance::leaf_binding::r1cs::blind_sender_leaf(
                cs.clone(),
                sender_leaf_hash,
                tx_blinding_nonce_var.clone(),
            )?;

        // Allocate claimed blinded sender leaf hash as public input and verify
        let claimed_blinded_sender =
            FqVar::new_input(cs.clone(), || Ok(self.public.sender_leaf_hash.0))?;
        computed_blinded_sender.enforce_equal(&claimed_blinded_sender)?;

        // Compute counterparty's leaf hash (already allocated above)
        let counterparty_leaf_hash = counterparty_leaf_var.commit(cs.clone())?;

        // Blind the counterparty leaf hash with the transaction nonce
        // Uses the shared leaf_binding module to ensure consistency
        let computed_blinded_counterparty =
            penumbra_sdk_compliance::leaf_binding::r1cs::blind_counterparty_leaf(
                cs.clone(),
                counterparty_leaf_hash,
                tx_blinding_nonce_var,
            )?;

        // Allocate claimed blinded counterparty leaf hash as public input and verify
        let claimed_blinded_counterparty =
            FqVar::new_input(cs.clone(), || Ok(self.public.counterparty_leaf_hash.0))?;
        computed_blinded_counterparty.enforce_equal(&claimed_blinded_counterparty)?;

        // NOTE: The counterparty address is verified to match counterparty_leaf.address
        // inside the verify_compliance_integrity gadget when it decrypts and validates
        // the compliance ciphertext. The gadget ensures that the decrypted counterparty
        // address matches the one provided in counterparty_leaf.
        // By binding the blinded counterparty_leaf_hash as a public input (blinded with the
        // shared tx_blinding_nonce), we create a cryptographic link that will be validated
        // at the transaction level against the output circuit, WITHOUT leaking which leaf
        // is transacting.

        Ok(())
    }
}

impl DummyWitness for SpendCircuit {
    fn with_dummy_witness() -> Self {
        use penumbra_sdk_compliance::crypto::encrypt_compliance_details;
        use penumbra_sdk_keys::keys::AddressComplianceKey;

        let seed_phrase = SeedPhrase::from_randomness(&[b'f'; 32]);
        let sk_sender = SpendKey::from_seed_phrase_bip44(seed_phrase, &Bip44Path::new(0));
        let fvk_sender = sk_sender.full_viewing_key();
        let ivk_sender = fvk_sender.incoming();
        let (address, _dtk_d) = ivk_sender.payment_address(0u32.into());

        let note = Note::from_parts(
            address.clone(),
            Value::from_str("1upenumbra").unwrap(),
            Rseed([1u8; 32]),
        )
        .unwrap();

        let mut sct = tct::Tree::new();
        sct.insert(tct::Witness::Keep, note.commit()).unwrap();
        let state_commitment_proof = sct.witness(note.commit()).unwrap();

        // Helper to create a dummy path of depth 16
        let dummy_path = penumbra_sdk_compliance::MerklePath {
            layers: (0..16)
                .map(|_| penumbra_sdk_compliance::structs::MerklePathLayer {
                    siblings: vec![vec![0u8; 32], vec![0u8; 32], vec![0u8; 32]],
                })
                .collect(),
        };

        // Create a dummy ACK for unregulated assets (using BLACK_HOLE_ACK)
        let dummy_ack = AddressComplianceKey::new(*penumbra_sdk_compliance::BLACK_HOLE_ACK);

        // Generate valid compliance ciphertext that will satisfy circuit constraints
        let mut rng = rand::thread_rng();
        let date = 0u64; // Day index = 0
        let (compliance_ciphertext_obj, ephemeral_secret) = encrypt_compliance_details(
            &mut rng,
            &dummy_ack,
            &address,
            date,
            note.asset_id(),
            note.amount(),
            address.clone(), // Use same address as counterparty for dummy
        )
        .expect("can encrypt compliance details");

        // Extract circuit inputs using unified method
        let (compliance_epk, compliance_ciphertext) =
            compliance_ciphertext_obj.to_circuit_public_inputs();

        let dummy_leaf = penumbra_sdk_compliance::ComplianceLeaf {
            address: address.clone(),
            key: dummy_ack.clone(),
            asset_id: penumbra_sdk_asset::asset::Id(Fq::from(0u64)),
        };
        let sender_leaf_hash =
            penumbra_sdk_compliance::blind_sender_leaf(dummy_leaf.commit(), Fr::from(0u64));
        let counterparty_leaf_hash =
            penumbra_sdk_compliance::blind_counterparty_leaf(dummy_leaf.commit(), Fr::from(0u64));

        let public = SpendProofPublic {
            anchor: sct.root(),
            balance_commitment: balance::Commitment(decaf377::Element::GENERATOR),
            nullifier: Nullifier(Fq::from(1u64)),
            rk: sk_sender.spend_auth_key().randomize(&Fr::from(1u64)).into(),
            asset_anchor: tct::StateCommitment(Fq::from(0u64)),
            compliance_anchor: tct::StateCommitment(Fq::from(0u64)),
            compliance_epk,
            compliance_ciphertext,
            target_timestamp: 0, // Corresponds to date = 0
            sender_leaf_hash,
            counterparty_leaf_hash,
        };
        let private = SpendProofPrivate {
            state_commitment_proof,
            note,
            v_blinding: Fr::from(1u64),
            spend_auth_randomizer: Fr::from(1u64),
            ak: sk_sender.spend_auth_key().into(),
            nk: *sk_sender.nullifier_key(),
            asset_path: dummy_path.clone(), // Must be length 16
            asset_position: 0,
            is_regulated: false,
            compliance_path: dummy_path, // Must be length 16
            compliance_position: 0,
            user_leaf: penumbra_sdk_compliance::ComplianceLeaf {
                address: address.clone(),
                key: dummy_ack,
                asset_id: penumbra_sdk_asset::asset::Id(Fq::from(0u64)),
            },
            compliance_ephemeral_secret: ephemeral_secret,
            counterparty_leaf: dummy_leaf,
            tx_blinding_nonce: Fr::from(0u64),
        };
        Self { public, private }
    }
}

#[derive(Clone, Debug)]
pub struct SpendProof([u8; GROTH16_PROOF_LENGTH_BYTES]);

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
    #[error("asset anchor is a Bls12-377 field member")]
    AssetAnchor,
    #[error("compliance anchor is a Bls12-377 field member")]
    ComplianceAnchor,
    #[error("compliance epk is a Bls12-377 field member")]
    ComplianceEpk,
    #[error("target timestamp is a Bls12-377 field member")]
    TargetTimestamp,
    #[error("sender leaf hash is a Bls12-377 field member")]
    SenderLeafHash,
    #[error("counterparty leaf hash is a Bls12-377 field member")]
    CounterpartyLeafHash,
    #[error("error verifying proof: {0:?}")]
    SynthesisError(ark_relations::r1cs::SynthesisError),
    #[error("spend proof did not verify")]
    InvalidProof,
}

impl SpendProof {
    /// Generate a `SpendProof` given the proving key, public inputs,
    /// witness data, and two random elements `blinding_r` and `blinding_s`.
    pub fn prove(
        blinding_r: Fq,
        blinding_s: Fq,
        pk: &ProvingKey<Bls12_377>,
        public: SpendProofPublic,
        private: SpendProofPrivate,
    ) -> Result<Self, crate::ProofError> {
        let circuit = SpendCircuit { public, private };

        // Check constraint satisfaction before proving
        use ark_relations::r1cs::ConstraintSystem;
        let cs = ConstraintSystem::<Fq>::new_ref();
        circuit.clone().generate_constraints(cs.clone())?;

        if !cs.is_satisfied().unwrap_or(false) {
            return Err(crate::ProofError::UnsatisfiedConstraints(
                "Spend circuit constraints not satisfied. Possible causes: \
                 asset registry verification failed, compliance registry verification failed, \
                 or ciphertext binding failed"
                    .to_string(),
            ));
        }

        let proof = Groth16::<Bls12_377, LibsnarkReduction>::create_proof_with_reduction(
            circuit, pk, blinding_r, blinding_s,
        )
        .map_err(|e| crate::ProofError::ProofGenerationFailed(format!("{:?}", e)))?;

        let mut proof_bytes = [0u8; GROTH16_PROOF_LENGTH_BYTES];
        Proof::serialize_compressed(&proof, &mut proof_bytes[..]).map_err(|e| {
            crate::ProofError::ProofGenerationFailed(format!("serialization failed: {:?}", e))
        })?;

        Ok(Self(proof_bytes))
    }

    /// Called to verify the proof using the provided public inputs.
    // For debugging proof verification failures,
    // to check that the proof data and verification keys are consistent.
    #[tracing::instrument(level="debug", skip(self, vk), fields(self = ?BASE64_STANDARD.encode(self.clone().encode_to_vec()), vk = ?vk.debug_id()))]
    pub fn verify(
        &self,
        vk: &PreparedVerifyingKey<Bls12_377>,
        SpendProofPublic {
            anchor: Root(anchor),
            balance_commitment: Commitment(balance_commitment),
            nullifier: Nullifier(nullifier),
            rk,
            asset_anchor,
            compliance_anchor,
            compliance_epk,
            compliance_ciphertext,
            target_timestamp,
            sender_leaf_hash,
            counterparty_leaf_hash,
        }: SpendProofPublic,
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

        // The circuit computes `timestamp / 86400` during witness generation,
        // so the public input must be the day index, not the raw timestamp.
        let day_index = target_timestamp / 86400;

        let mut public_inputs = [
            to_field_elements!(Fq::from(anchor), Anchor),
            to_field_elements!(balance_commitment, BalanceCommitment),
            to_field_elements!(nullifier, Nullifier),
            to_field_elements!(element_rk, Rk),
            to_field_elements!(asset_anchor.0, AssetAnchor),
            to_field_elements!(compliance_anchor.0, ComplianceAnchor),
            to_field_elements!(Fq::from(day_index), TargetTimestamp), // Day index, not raw timestamp
            to_field_elements!(compliance_epk, ComplianceEpk),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

        // Add packed compliance_ciphertext to public inputs (already in Fq format)
        public_inputs.extend(compliance_ciphertext);

        // Add blinded leaf hashes to public inputs
        public_inputs.extend(to_field_elements!(sender_leaf_hash.0, SenderLeafHash));
        public_inputs.extend(to_field_elements!(
            counterparty_leaf_hash.0,
            CounterpartyLeafHash
        ));

        tracing::trace!(?public_inputs);

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
    use ark_r1cs_std::prelude::Boolean;
    use decaf377::{Fq, Fr};
    use penumbra_sdk_asset::{asset, Value};
    use penumbra_sdk_keys::{
        keys::{Bip44Path, SeedPhrase, SpendKey},
        Address,
    };
    use penumbra_sdk_num::Amount;
    use penumbra_sdk_proof_params::generate_prepared_test_parameters;
    use penumbra_sdk_sct::Nullifier;
    use penumbra_sdk_tct::StateCommitment;
    use proptest::prelude::*;

    use crate::test_proof_helpers::proof_test_helpers::current_timestamp;
    use crate::Note;
    use decaf377_rdsa::{SpendAuth, VerificationKey};
    use penumbra_sdk_tct as tct;
    use rand_core::OsRng;

    fn fr_strategy() -> BoxedStrategy<Fr> {
        any::<[u8; 32]>()
            .prop_map(|bytes| Fr::from_le_bytes_mod_order(&bytes[..]))
            .boxed()
    }

    prop_compose! {
        // Unregulated strategy
        fn arb_valid_spend_statement()(
            v_blinding in fr_strategy(),
            spend_auth_randomizer in fr_strategy(),
            asset_id64 in any::<u64>(),
            address_index in any::<u32>(),
            amount in any::<u64>(),
            seed_phrase_randomness in any::<[u8; 32]>(),
            rseed_randomness in any::<[u8; 32]>(),
            num_commitments in 0..100,
            tester_keys_rand in any::<[u8; 32]>()
        ) -> (SpendProofPublic, SpendProofPrivate, Fr) {
            use penumbra_sdk_compliance::crypto::encrypt_compliance_details;

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
                sender.clone(),
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
                let dummy_note_commitment = Note::from_parts(sender.clone(), value_to_send, rseed).expect("can create note").commit();
                sct.insert(tct::Witness::Keep, dummy_note_commitment).expect("should be able to insert note commitments into the SCT");
            }

            sct.insert(tct::Witness::Keep, note_commitment).expect("should be able to insert note commitments into the SCT");
            let anchor = sct.root();
            let state_commitment_proof = sct.witness(note_commitment).expect("can witness note commitment");
            let balance_commitment = value_to_send.commit(v_blinding);
            let rk: VerificationKey<SpendAuth> = rsk.into();
            let nullifier = Nullifier::derive(&nk, state_commitment_proof.position(), &note_commitment);

            // Unregulated: use BLACK_HOLE_ACK with real encryption
            let black_hole_ack = penumbra_sdk_keys::keys::AddressComplianceKey::new(*penumbra_sdk_compliance::BLACK_HOLE_ACK);
            let user_leaf = penumbra_sdk_compliance::ComplianceLeaf {
                address: sender.clone(),
                key: black_hole_ack.clone(),
                asset_id: value_to_send.asset_id,
            };

            // Generate valid compliance ciphertext using real encryption with BLACK_HOLE_ACK
            let mut rng = rand::thread_rng();
            let timestamp = current_timestamp();
            let date = timestamp / 86400;
            let (compliance_ciphertext_obj, ephemeral_secret) = encrypt_compliance_details(
                &mut rng,
                &black_hole_ack,
                &sender,
                date,
                note.asset_id(),
                note.amount(),
                sender.clone(),
            ).expect("can encrypt compliance details");

            let (compliance_epk, packed_ciphertext) = compliance_ciphertext_obj.to_circuit_public_inputs();

            // Use sender as counterparty for testing (self-send scenario)
            let counterparty_leaf = penumbra_sdk_compliance::ComplianceLeaf {
                address: sender.clone(),
                key: black_hole_ack,
                asset_id: value_to_send.asset_id,
            };
            let tx_blinding_nonce = Fr::from(0u64);
            let sender_leaf_hash = penumbra_sdk_compliance::blind_sender_leaf(user_leaf.commit(), tx_blinding_nonce);
            let counterparty_leaf_hash = penumbra_sdk_compliance::blind_counterparty_leaf(counterparty_leaf.commit(), tx_blinding_nonce);

            let public = SpendProofPublic {
                anchor,
                balance_commitment,
                nullifier,
                rk,
                asset_anchor: tct::StateCommitment(Fq::from(0u64)),
                compliance_anchor: tct::StateCommitment(Fq::from(0u64)),
                compliance_epk,
                compliance_ciphertext: packed_ciphertext,
                target_timestamp: timestamp,
                sender_leaf_hash,
                counterparty_leaf_hash,
            };

            let private = SpendProofPrivate {
                state_commitment_proof,
                note,
                v_blinding,
                spend_auth_randomizer,
                ak,
                nk,
                asset_path: penumbra_sdk_compliance::MerklePath::default(),
                asset_position: 0,
                is_regulated: false,
                compliance_path: penumbra_sdk_compliance::MerklePath::default(),
                compliance_position: 0,
                user_leaf,
                compliance_ephemeral_secret: ephemeral_secret,
                counterparty_leaf,
                tx_blinding_nonce,
            };

            // Tester random key (not used in circuit, just for test structure)
            let my_random_sk_scalar = Fr::from_le_bytes_mod_order(&tester_keys_rand);
            let my_fake_cvk_sk = my_random_sk_scalar;

            (public, private, my_fake_cvk_sk)
        }
    }

    prop_compose! {
        // Regulated strategy
        fn arb_regulated_spend_statement()(
            v_blinding in fr_strategy(),
            spend_auth_randomizer in fr_strategy(),
            asset_id64 in any::<u64>(),
            address_index in any::<u32>(),
            amount in any::<u64>(),
            seed_phrase_randomness in any::<[u8; 32]>(),
            rseed_randomness in any::<[u8; 32]>(),
            num_commitments in 0..100,
            cvk_sk_rand in any::<[u8; 32]>()
        ) -> (SpendProofPublic, SpendProofPrivate, Fr) {
            use penumbra_sdk_compliance::crypto::encrypt_compliance_details;

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
                sender.clone(),
                value_to_send,
                Rseed(rseed_randomness),
            ).expect("should be able to create note");
            let note_commitment = note.commit();
            let rsk = sk_sender.spend_auth_key().randomize(&spend_auth_randomizer);
            let nk = *sk_sender.nullifier_key();
            let ak: VerificationKey<SpendAuth> = sk_sender.spend_auth_key().into();

            let mut sct = tct::Tree::new();
            for i in 0..num_commitments {
                let rseed = Rseed([i as u8; 32]);
                let dummy_note_commitment = Note::from_parts(sender.clone(), value_to_send, rseed).expect("can create note").commit();
                sct.insert(tct::Witness::Keep, dummy_note_commitment).expect("should be able to insert note commitments into the SCT");
            }
            sct.insert(tct::Witness::Keep, note_commitment).expect("should be able to insert note commitments into the SCT");
            let anchor = sct.root();
            let state_commitment_proof = sct.witness(note_commitment).expect("can witness note commitment");
            let balance_commitment = value_to_send.commit(v_blinding);
            let rk: VerificationKey<SpendAuth> = rsk.into();
            let nullifier = Nullifier::derive(&nk, state_commitment_proof.position(), &note_commitment);

            // Generate issuer keypair (regulated asset)
            let cvk_sk = Fr::from_le_bytes_mod_order(&cvk_sk_rand);
            let ack_point = decaf377::Element::GENERATOR * cvk_sk;
            let user_ack = penumbra_sdk_keys::keys::AddressComplianceKey::new(ack_point);
            let user_leaf = penumbra_sdk_compliance::ComplianceLeaf {
                address: sender.clone(),
                key: user_ack.clone(),
                asset_id: value_to_send.asset_id,
            };

            // Generate valid compliance ciphertext using real encryption with the ACK
            let mut rng = rand::thread_rng();
            let timestamp = current_timestamp();
            let date = timestamp / 86400;
            let (compliance_ciphertext_obj, ephemeral_secret) = encrypt_compliance_details(
                &mut rng,
                &user_ack,
                &sender,
                date,
                note.asset_id(),
                note.amount(),
                sender.clone(),
            ).expect("can encrypt compliance details");

            let (compliance_epk, packed_ciphertext) = compliance_ciphertext_obj.to_circuit_public_inputs();

            // Use sender as counterparty for testing (self-send scenario)
            let counterparty_leaf = penumbra_sdk_compliance::ComplianceLeaf {
                address: sender.clone(),
                key: user_ack,
                asset_id: value_to_send.asset_id,
            };
            let tx_blinding_nonce = Fr::from(0u64);
            let sender_leaf_hash = penumbra_sdk_compliance::blind_sender_leaf(user_leaf.commit(), tx_blinding_nonce);
            let counterparty_leaf_hash = penumbra_sdk_compliance::blind_counterparty_leaf(counterparty_leaf.commit(), tx_blinding_nonce);

            let public = SpendProofPublic {
                anchor,
                balance_commitment,
                nullifier,
                rk,
                asset_anchor: tct::StateCommitment(Fq::from(0u64)),
                compliance_anchor: tct::StateCommitment(Fq::from(0u64)),
                compliance_epk,
                compliance_ciphertext: packed_ciphertext,
                target_timestamp: timestamp,
                sender_leaf_hash,
                counterparty_leaf_hash,
            };
            let private = SpendProofPrivate {
                state_commitment_proof,
                note,
                v_blinding,
                spend_auth_randomizer,
                ak,
                nk,
                asset_path: penumbra_sdk_compliance::MerklePath::default(),
                asset_position: 0,
                is_regulated: true,
                compliance_path: penumbra_sdk_compliance::MerklePath::default(),
                compliance_position: 0,
                user_leaf,
                compliance_ephemeral_secret: ephemeral_secret,
                counterparty_leaf,
                tx_blinding_nonce,
            };
            (public, private, cvk_sk)
        }
    }

    proptest! {
        #![proptest_config(proptest::test_runner::Config::with_cases(1))]
        #[test]
        fn spend_proof_unregulated_asset((public, private, _) in arb_valid_spend_statement()) {
            assert!(check_satisfaction(&public, &private).is_ok());
            assert!(check_circuit_satisfaction(public, private).is_ok());
        }
    }

    proptest! {
        #![proptest_config(proptest::test_runner::Config::with_cases(1))]
        #[test]
        /// Test that regulated spends (using real issuer key) also satisfy the circuit
        fn spend_proof_regulated_asset((public, private, _) in arb_regulated_spend_statement()) {
            assert!(check_satisfaction(&public, &private).is_ok());
            assert!(check_circuit_satisfaction(public, private).is_ok());
        }
    }

    #[test]
    fn spend_proof_full_groth16_roundtrip_regulated() {
        use crate::test_proof_helpers::proof_test_helpers::{full_groth16_roundtrip, CircuitType};
        full_groth16_roundtrip(CircuitType::Spend, true);
    }

    #[test]
    fn spend_proof_full_groth16_roundtrip_unregulated() {
        use crate::test_proof_helpers::proof_test_helpers::{full_groth16_roundtrip, CircuitType};
        full_groth16_roundtrip(CircuitType::Spend, false);
    }

    prop_compose! {
        // This strategy generates a spend statement that uses a Merkle root
        // from prior to the note commitment being added to the SCT. The Merkle
        // path should not verify using this invalid root, and as such the circuit
        // should be unsatisfiable.
        fn arb_invalid_spend_statement_incorrect_anchor()(v_blinding in fr_strategy(), spend_auth_randomizer in fr_strategy(), asset_id64 in any::<u64>(), address_index in any::<u32>(), amount in any::<u64>(), seed_phrase_randomness in any::<[u8; 32]>(), rseed_randomness in any::<[u8; 32]>(), num_commitments in 0..100) -> (SpendProofPublic, SpendProofPrivate) {
            use penumbra_sdk_compliance::crypto::encrypt_compliance_details;

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
                sender.clone(),
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
                let dummy_note_commitment = Note::from_parts(sender.clone(), value_to_send, rseed).expect("can create note").commit();
                sct.insert(tct::Witness::Keep, dummy_note_commitment).expect("should be able to insert note commitments into the SCT");
            }
            let incorrect_anchor = sct.root();

            sct.insert(tct::Witness::Keep, note_commitment).expect("should be able to insert note commitments into the SCT");
            let state_commitment_proof = sct.witness(note_commitment).expect("can witness note commitment");
            let balance_commitment = value_to_send.commit(v_blinding);
            let rk: VerificationKey<SpendAuth> = rsk.into();
            let nullifier = Nullifier::derive(&nk, state_commitment_proof.position(), &note_commitment);

            let black_hole_ack = penumbra_sdk_keys::keys::AddressComplianceKey::new(*penumbra_sdk_compliance::BLACK_HOLE_ACK);
            let user_leaf = penumbra_sdk_compliance::ComplianceLeaf {
                address: sender.clone(),
                key: black_hole_ack.clone(),
                asset_id: value_to_send.asset_id,
            };

            let mut rng = rand::thread_rng();
            let timestamp = current_timestamp();
            let date = timestamp / 86400;
            let (compliance_ciphertext_obj, ephemeral_secret) = encrypt_compliance_details(
                &mut rng,
                &black_hole_ack,
                &sender,
                date,
                note.asset_id(),
                note.amount(),
                sender.clone(),
            ).expect("can encrypt compliance details");

            let (compliance_epk, packed_ciphertext) = compliance_ciphertext_obj.to_circuit_public_inputs();

            let counterparty_leaf = penumbra_sdk_compliance::ComplianceLeaf {
                address: sender.clone(),
                key: black_hole_ack.clone(),
                asset_id: value_to_send.asset_id,
            };
            let tx_blinding_nonce = Fr::from(0u64);
            let sender_leaf_hash = penumbra_sdk_compliance::blind_sender_leaf(user_leaf.commit(), tx_blinding_nonce);
            let counterparty_leaf_hash = penumbra_sdk_compliance::blind_counterparty_leaf(counterparty_leaf.commit(), tx_blinding_nonce);

            let public = SpendProofPublic {
                anchor: incorrect_anchor,
                balance_commitment,
                nullifier,
                rk,
                asset_anchor: tct::StateCommitment(Fq::from(0u64)),
                compliance_anchor: tct::StateCommitment(Fq::from(0u64)),
                compliance_epk,
                compliance_ciphertext: packed_ciphertext,
                target_timestamp: timestamp,
                sender_leaf_hash,
                counterparty_leaf_hash,
            };

            let private = SpendProofPrivate {
                state_commitment_proof,
                note,
                v_blinding,
                spend_auth_randomizer,
                ak,
                nk,
                asset_path: penumbra_sdk_compliance::MerklePath::default(),
                asset_position: 0,
                is_regulated: false,
                compliance_path: penumbra_sdk_compliance::MerklePath::default(),
                compliance_position: 0,
                user_leaf,
                compliance_ephemeral_secret: ephemeral_secret,
                counterparty_leaf,
                tx_blinding_nonce,
            };
            (public, private)
        }
    }

    proptest! {
    #[test]
    /// Check that the `SpendCircuit` is not satisfied when using an incorrect
    /// TCT root (`anchor`).
    fn spend_proof_verification_merkle_path_integrity_failure((public, private) in arb_invalid_spend_statement_incorrect_anchor()) {
        assert!(check_satisfaction(&public, &private).is_err());
            assert!(check_circuit_satisfaction(public, private).is_err());
    }
    }

    prop_compose! {
        // Recall: The transmission key `pk_d` is derived as:
        //
        // `pk_d ​= [ivk] B_d`
        //
        // where `B_d` is the diversified basepoint and `ivk` is the incoming
        // viewing key.
        //
        // This strategy generates a spend statement that is spending a note
        // that corresponds to a diversified address associated with a different
        // IVK, i.e. the prover cannot demonstrate the transmission key `pk_d`
        // was derived as above and the circuit should be unsatisfiable.
        fn arb_invalid_spend_statement_diversified_address()(v_blinding in fr_strategy(), spend_auth_randomizer in fr_strategy(), asset_id64 in any::<u64>(), address_index in any::<u32>(), amount in any::<u64>(), seed_phrase_randomness in any::<[u8; 32]>(), incorrect_seed_phrase_randomness in any::<[u8; 32]>(), rseed_randomness in any::<[u8; 32]>()) -> (SpendProofPublic, SpendProofPrivate) {
            use penumbra_sdk_compliance::crypto::encrypt_compliance_details;

            let seed_phrase = SeedPhrase::from_randomness(&seed_phrase_randomness);
            let sk_sender = SpendKey::from_seed_phrase_bip44(seed_phrase, &Bip44Path::new(0));
            let fvk_sender = sk_sender.full_viewing_key();
            let ivk_sender = fvk_sender.incoming();
            let (_sender, _dtk_d) = ivk_sender.payment_address(address_index.into());
            let value_to_send = Value {
                amount: Amount::from(amount),
                asset_id: asset::Id(Fq::from(asset_id64)),
            };

            let wrong_seed_phrase = SeedPhrase::from_randomness(&incorrect_seed_phrase_randomness);
            let wrong_sk_sender = SpendKey::from_seed_phrase_bip44(wrong_seed_phrase, &Bip44Path::new(0));
            let wrong_fvk_sender = wrong_sk_sender.full_viewing_key();
            let wrong_ivk_sender = wrong_fvk_sender.incoming();
            let (wrong_sender, _dtk_d) = wrong_ivk_sender.payment_address(address_index.into());

            let note = Note::from_parts(
                wrong_sender.clone(),
                value_to_send,
                Rseed(rseed_randomness),
            ).expect("should be able to create note");
            let note_commitment = note.commit();
            let rsk = sk_sender.spend_auth_key().randomize(&spend_auth_randomizer);
            let nk = *sk_sender.nullifier_key();
            let ak: VerificationKey<SpendAuth> = sk_sender.spend_auth_key().into();

            let mut sct = tct::Tree::new();
            sct.insert(tct::Witness::Keep, note_commitment).expect("should be able to insert note commitments into the SCT");
            let anchor = sct.root();
            let state_commitment_proof = sct.witness(note_commitment).expect("can witness note commitment");
            let balance_commitment = value_to_send.commit(v_blinding);
            let rk: VerificationKey<SpendAuth> = rsk.into();
            let nullifier = Nullifier::derive(&nk, state_commitment_proof.position(), &note_commitment);

            // Use BLACK_HOLE_ACK with real encryption for unregulated assets
            let black_hole_ack = penumbra_sdk_keys::keys::AddressComplianceKey::new(*penumbra_sdk_compliance::BLACK_HOLE_ACK);
            let user_leaf = penumbra_sdk_compliance::ComplianceLeaf {
                address: wrong_sender.clone(),
                key: black_hole_ack.clone(),
                asset_id: value_to_send.asset_id,
            };

            let mut rng = rand::thread_rng();
            let timestamp = current_timestamp();
            let date = timestamp / 86400;
            let (compliance_ciphertext_obj, ephemeral_secret) = encrypt_compliance_details(
                &mut rng,
                &black_hole_ack,
                &wrong_sender,
                date,
                note.asset_id(),
                note.amount(),
                wrong_sender.clone(),
            ).expect("can encrypt compliance details");

            let (compliance_epk, packed_ciphertext) = compliance_ciphertext_obj.to_circuit_public_inputs();

            let counterparty_leaf = penumbra_sdk_compliance::ComplianceLeaf {
                address: wrong_sender.clone(),
                key: black_hole_ack,
                asset_id: value_to_send.asset_id,
            };
            let tx_blinding_nonce = Fr::from(0u64);
            let sender_leaf_hash = penumbra_sdk_compliance::blind_sender_leaf(user_leaf.commit(), tx_blinding_nonce);
            let counterparty_leaf_hash = penumbra_sdk_compliance::blind_counterparty_leaf(counterparty_leaf.commit(), tx_blinding_nonce);

            let public = SpendProofPublic {
                anchor,
                balance_commitment,
                nullifier,
                rk,
                asset_anchor: tct::StateCommitment(Fq::from(0u64)),
                compliance_anchor: tct::StateCommitment(Fq::from(0u64)),
                compliance_epk,
                compliance_ciphertext: packed_ciphertext,
                target_timestamp: timestamp,
                sender_leaf_hash,
                counterparty_leaf_hash,
            };

            let private = SpendProofPrivate {
                state_commitment_proof,
                note,
                v_blinding,
                spend_auth_randomizer,
                ak,
                nk,
                asset_path: penumbra_sdk_compliance::MerklePath::default(),
                asset_position: 0,
                is_regulated: false,
                compliance_path: penumbra_sdk_compliance::MerklePath::default(),
                compliance_position: 0,
                user_leaf,
                compliance_ephemeral_secret: ephemeral_secret,
                counterparty_leaf,
                tx_blinding_nonce,
            };
            (public, private)
        }
    }

    proptest! {
        #[test]
        /// Check that the `SpendCircuit` is not satisfied when the diversified address is wrong.
        fn spend_proof_verification_diversified_address_integrity_failure((public, private) in arb_invalid_spend_statement_diversified_address()) {
            assert!(check_satisfaction(&public, &private).is_err());
            assert!(check_circuit_satisfaction(public, private).is_err());
        }
    }

    prop_compose! {
        // This strategy generates a spend statement that derives a nullifier
        // using a different position.
        fn arb_invalid_spend_statement_nullifier()(v_blinding in fr_strategy(), spend_auth_randomizer in fr_strategy(), asset_id64 in any::<u64>(), address_index in any::<u32>(), amount in any::<u64>(), seed_phrase_randomness in any::<[u8; 32]>(), rseed_randomness in any::<[u8; 32]>(), num_commitments in 0..100) -> (SpendProofPublic, SpendProofPrivate) {
            use penumbra_sdk_compliance::crypto::encrypt_compliance_details;

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
                sender.clone(),
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
                let dummy_note_commitment = Note::from_parts(sender.clone(), value_to_send, rseed).expect("can create note").commit();
                sct.insert(tct::Witness::Keep, dummy_note_commitment).expect("should be able to insert note commitments into the SCT");
            }
            // Insert one more note commitment and witness it.
            let rseed = Rseed([num_commitments as u8; 32]);
            let dummy_note_commitment = Note::from_parts(sender.clone(), value_to_send, rseed).expect("can create note").commit();
            sct.insert(tct::Witness::Keep, dummy_note_commitment).expect("should be able to insert note commitments into the SCT");
            let incorrect_position = sct.witness(dummy_note_commitment).expect("can witness note commitment").position();

            sct.insert(tct::Witness::Keep, note_commitment).expect("should be able to insert note commitments into the SCT");
            let anchor = sct.root();
            let state_commitment_proof = sct.witness(note_commitment).expect("can witness note commitment");
            let balance_commitment = value_to_send.commit(v_blinding);
            let rk: VerificationKey<SpendAuth> = rsk.into();
            let incorrect_nf = Nullifier::derive(&nk, incorrect_position, &note_commitment);

            let black_hole_ack = penumbra_sdk_keys::keys::AddressComplianceKey::new(*penumbra_sdk_compliance::BLACK_HOLE_ACK);
            let user_leaf = penumbra_sdk_compliance::ComplianceLeaf {
                address: sender.clone(),
                key: black_hole_ack.clone(),
                asset_id: value_to_send.asset_id,
            };

            let mut rng = rand::thread_rng();
            let timestamp = current_timestamp();
            let date = timestamp / 86400;
            let (compliance_ciphertext_obj, ephemeral_secret) = encrypt_compliance_details(
                &mut rng,
                &black_hole_ack,
                &sender,
                date,
                note.asset_id(),
                note.amount(),
                sender.clone(),
            ).expect("can encrypt compliance details");

            let (compliance_epk, packed_ciphertext) = compliance_ciphertext_obj.to_circuit_public_inputs();

            let counterparty_leaf = penumbra_sdk_compliance::ComplianceLeaf {
                address: sender.clone(),
                key: black_hole_ack.clone(),
                asset_id: value_to_send.asset_id,
            };
            let tx_blinding_nonce = Fr::from(0u64);
            let sender_leaf_hash = penumbra_sdk_compliance::blind_sender_leaf(user_leaf.commit(), tx_blinding_nonce);
            let counterparty_leaf_hash = penumbra_sdk_compliance::blind_counterparty_leaf(counterparty_leaf.commit(), tx_blinding_nonce);

            let public = SpendProofPublic {
                anchor,
                balance_commitment,
                nullifier: incorrect_nf,
                rk,
                asset_anchor: tct::StateCommitment(Fq::from(0u64)),
                compliance_anchor: tct::StateCommitment(Fq::from(0u64)),
                compliance_epk,
                compliance_ciphertext: packed_ciphertext,
                target_timestamp: timestamp,
                sender_leaf_hash,
                counterparty_leaf_hash,
            };

            let private = SpendProofPrivate {
                state_commitment_proof,
                note,
                v_blinding,
                spend_auth_randomizer,
                ak,
                nk,
                asset_path: penumbra_sdk_compliance::MerklePath::default(),
                asset_position: 0,
                is_regulated: false,
                compliance_path: penumbra_sdk_compliance::MerklePath::default(),
                compliance_position: 0,
                user_leaf,
                compliance_ephemeral_secret: ephemeral_secret,
                counterparty_leaf,
                tx_blinding_nonce,
            };
            (public, private)
        }
    }

    proptest! {
        #[test]
        /// Check that the `SpendCircuit` is not satisfied, when using an
        /// incorrect nullifier.
        fn spend_proof_verification_nullifier_integrity_failure((public, private) in arb_invalid_spend_statement_nullifier()) {
            assert!(check_satisfaction(&public, &private).is_err());
            assert!(check_circuit_satisfaction(public, private).is_err());
        }
    }

    prop_compose! {
        // This statement uses a randomly generated incorrect value blinding factor for deriving the
        // balance commitment.
        fn arb_invalid_spend_statement_v_blinding_factor()(v_blinding in fr_strategy(), incorrect_v_blinding in fr_strategy(), spend_auth_randomizer in fr_strategy(), asset_id64 in any::<u64>(), address_index in any::<u32>(), amount in any::<u64>(), seed_phrase_randomness in any::<[u8; 32]>(), rseed_randomness in any::<[u8; 32]>(), num_commitments in 0..100) -> (SpendProofPublic, SpendProofPrivate) {
            use penumbra_sdk_compliance::crypto::encrypt_compliance_details;

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
                sender.clone(),
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
                let dummy_note_commitment = Note::from_parts(sender.clone(), value_to_send, rseed).expect("can create note").commit();
                sct.insert(tct::Witness::Keep, dummy_note_commitment).expect("should be able to insert note commitments into the SCT");
            }

            sct.insert(tct::Witness::Keep, note_commitment).expect("should be able to insert note commitments into the SCT");
            let anchor = sct.root();
            let state_commitment_proof = sct.witness(note_commitment).expect("can witness note commitment");
            let balance_commitment = value_to_send.commit(v_blinding);
            let rk: VerificationKey<SpendAuth> = rsk.into();
            let nullifier = Nullifier::derive(&nk, state_commitment_proof.position(), &note_commitment);

            // Use BLACK_HOLE_ACK with real encryption for unregulated assets
            let black_hole_ack = penumbra_sdk_keys::keys::AddressComplianceKey::new(*penumbra_sdk_compliance::BLACK_HOLE_ACK);
            let user_leaf = penumbra_sdk_compliance::ComplianceLeaf {
                address: sender.clone(),
                key: black_hole_ack.clone(),
                asset_id: value_to_send.asset_id,
            };

            let mut rng = rand::thread_rng();
            let timestamp = current_timestamp();
            let date = timestamp / 86400;
            let (compliance_ciphertext_obj, ephemeral_secret) = encrypt_compliance_details(
                &mut rng,
                &black_hole_ack,
                &sender,
                date,
                note.asset_id(),
                note.amount(),
                sender.clone(),
            ).expect("can encrypt compliance details");

            let (compliance_epk, packed_ciphertext) = compliance_ciphertext_obj.to_circuit_public_inputs();

            let counterparty_leaf = penumbra_sdk_compliance::ComplianceLeaf {
                address: sender.clone(),
                key: black_hole_ack,
                asset_id: value_to_send.asset_id,
            };
            let tx_blinding_nonce = Fr::from(0u64);
            let sender_leaf_hash = penumbra_sdk_compliance::blind_sender_leaf(user_leaf.commit(), tx_blinding_nonce);
            let counterparty_leaf_hash = penumbra_sdk_compliance::blind_counterparty_leaf(counterparty_leaf.commit(), tx_blinding_nonce);

            let public = SpendProofPublic {
                anchor,
                balance_commitment,
                nullifier,
                rk,
                asset_anchor: tct::StateCommitment(Fq::from(0u64)),
                compliance_anchor: tct::StateCommitment(Fq::from(0u64)),
                compliance_epk,
                compliance_ciphertext: packed_ciphertext,
                target_timestamp: timestamp,
                sender_leaf_hash,
                counterparty_leaf_hash,
            };
            let private = SpendProofPrivate {
                state_commitment_proof,
                note,
                v_blinding: incorrect_v_blinding,
                spend_auth_randomizer,
                ak,
                nk,
                asset_path: penumbra_sdk_compliance::MerklePath::default(),
                asset_position: 0,
                is_regulated: false,
                compliance_path: penumbra_sdk_compliance::MerklePath::default(),
                compliance_position: 0,
                user_leaf,
                compliance_ephemeral_secret: ephemeral_secret,
                counterparty_leaf,
                tx_blinding_nonce,
            };
            (public, private)
        }
    }

    proptest! {
        #[test]
        /// Check that the `SpendCircuit` is not satisfied when using balance
        /// commitments with different blinding factors.
        fn spend_proof_verification_balance_commitment_integrity_failure((public, private) in arb_invalid_spend_statement_v_blinding_factor()) {
            assert!(check_satisfaction(&public, &private).is_err());
            assert!(check_circuit_satisfaction(public, private).is_err());
        }
    }

    prop_compose! {
        // This statement uses a randomly generated incorrect spend auth randomizer for deriving the
        // randomized verification key.
        fn arb_invalid_spend_statement_rk_integrity()(v_blinding in fr_strategy(), spend_auth_randomizer in fr_strategy(), asset_id64 in any::<u64>(), address_index in any::<u32>(), amount in any::<u64>(), seed_phrase_randomness in any::<[u8; 32]>(), rseed_randomness in any::<[u8; 32]>(), num_commitments in 0..100, incorrect_spend_auth_randomizer in fr_strategy()) -> (SpendProofPublic, SpendProofPrivate) {
            use penumbra_sdk_compliance::crypto::encrypt_compliance_details;

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
                sender.clone(),
                value_to_send,
                Rseed(rseed_randomness),
            ).expect("should be able to create note");
            let note_commitment = note.commit();
            let nk = *sk_sender.nullifier_key();
            let ak: VerificationKey<SpendAuth> = sk_sender.spend_auth_key().into();

            let mut sct = tct::Tree::new();

            // Next, we simulate the case where the SCT is not empty by adding `num_commitments`
            // unrelated items in the SCT.
            for i in 0..num_commitments {
                // To avoid duplicate note commitments, we use the `i` counter as the Rseed randomness
                let rseed = Rseed([i as u8; 32]);
                let dummy_note_commitment = Note::from_parts(sender.clone(), value_to_send, rseed).expect("can create note").commit();
                sct.insert(tct::Witness::Keep, dummy_note_commitment).expect("should be able to insert note commitments into the SCT");
            }

            sct.insert(tct::Witness::Keep, note_commitment).expect("should be able to insert note commitments into the SCT");
            let anchor = sct.root();
            let state_commitment_proof = sct.witness(note_commitment).expect("can witness note commitment");
            let balance_commitment = value_to_send.commit(v_blinding);
            let nullifier = Nullifier::derive(&nk, state_commitment_proof.position(), &note_commitment);

            let incorrect_rsk = sk_sender
                .spend_auth_key()
                .randomize(&incorrect_spend_auth_randomizer);
            let incorrect_rk: VerificationKey<SpendAuth> = incorrect_rsk.into();

            let black_hole_ack = penumbra_sdk_keys::keys::AddressComplianceKey::new(*penumbra_sdk_compliance::BLACK_HOLE_ACK);
            let user_leaf = penumbra_sdk_compliance::ComplianceLeaf {
                address: sender.clone(),
                key: black_hole_ack.clone(),
                asset_id: value_to_send.asset_id,
            };

            let mut rng = rand::thread_rng();
            let timestamp = current_timestamp();
            let date = timestamp / 86400;
            let (compliance_ciphertext_obj, ephemeral_secret) = encrypt_compliance_details(
                &mut rng,
                &black_hole_ack,
                &sender,
                date,
                note.asset_id(),
                note.amount(),
                sender.clone(),
            ).expect("can encrypt compliance details");

            let (compliance_epk, packed_ciphertext) = compliance_ciphertext_obj.to_circuit_public_inputs();

            let counterparty_leaf = penumbra_sdk_compliance::ComplianceLeaf {
                address: sender.clone(),
                key: black_hole_ack.clone(),
                asset_id: value_to_send.asset_id,
            };
            let tx_blinding_nonce = Fr::from(0u64);
            let sender_leaf_hash = penumbra_sdk_compliance::blind_sender_leaf(user_leaf.commit(), tx_blinding_nonce);
            let counterparty_leaf_hash = penumbra_sdk_compliance::blind_counterparty_leaf(counterparty_leaf.commit(), tx_blinding_nonce);

            let public = SpendProofPublic {
                anchor,
                balance_commitment,
                nullifier,
                rk: incorrect_rk,
                asset_anchor: tct::StateCommitment(Fq::from(0u64)),
                compliance_anchor: tct::StateCommitment(Fq::from(0u64)),
                compliance_epk,
                compliance_ciphertext: packed_ciphertext,
                target_timestamp: timestamp,
                sender_leaf_hash,
                counterparty_leaf_hash,
            };

            let private = SpendProofPrivate {
                state_commitment_proof,
                note,
                v_blinding,
                spend_auth_randomizer,
                ak,
                nk,
                asset_path: penumbra_sdk_compliance::MerklePath::default(),
                asset_position: 0,
                is_regulated: false,
                compliance_path: penumbra_sdk_compliance::MerklePath::default(),
                compliance_position: 0,
                user_leaf,
                compliance_ephemeral_secret: ephemeral_secret,
                counterparty_leaf,
                tx_blinding_nonce,
            };
            (public, private)
        }
    }

    proptest! {
        #[test]
        /// Check that the `SpendCircuit` is not satisfied when the incorrect randomizable verification key is used.
        fn spend_proof_verification_fails_rk_integrity((public, private) in arb_invalid_spend_statement_rk_integrity()) {
            assert!(check_satisfaction(&public, &private).is_err());
            assert!(check_circuit_satisfaction(public, private).is_err());
        }
    }

    prop_compose! {
        fn arb_valid_dummy_spend_statement()(v_blinding in fr_strategy(), spend_auth_randomizer in fr_strategy(), asset_id64 in any::<u64>(), address_index in any::<u32>(), seed_phrase_randomness in any::<[u8; 32]>(), rseed_randomness in any::<[u8; 32]>()) -> (SpendProofPublic, SpendProofPrivate) {
            use penumbra_sdk_compliance::crypto::encrypt_compliance_details;

            let seed_phrase = SeedPhrase::from_randomness(&seed_phrase_randomness);
            let sk_sender = SpendKey::from_seed_phrase_bip44(seed_phrase, &Bip44Path::new(0));
            let fvk_sender = sk_sender.full_viewing_key();
            let ivk_sender = fvk_sender.incoming();
            let (sender, _dtk_d) = ivk_sender.payment_address(address_index.into());
            let value_to_send = Value {
                amount: Amount::from(0u64),
                asset_id: asset::Id(Fq::from(asset_id64)),
            };
            let note = Note::from_parts(
                sender.clone(),
                value_to_send,
                Rseed(rseed_randomness),
            ).expect("should be able to create note");
            let note_commitment = note.commit();
            let rsk = sk_sender.spend_auth_key().randomize(&spend_auth_randomizer);
            let nk = *sk_sender.nullifier_key();
            let ak: VerificationKey<SpendAuth> = sk_sender.spend_auth_key().into();

            let mut sct = tct::Tree::new();
            sct.insert(tct::Witness::Keep, note_commitment).expect("should be able to insert note commitments into the SCT");

            let state_commitment_proof = sct.witness(note_commitment).expect("can witness note commitment");
            let balance_commitment = value_to_send.commit(v_blinding);
            let rk: VerificationKey<SpendAuth> = rsk.into();
            let nullifier = Nullifier::derive(&nk, state_commitment_proof.position(), &note_commitment);

            // use an invalid anchor to verify that the circuit skips inclusion checks for dummy
            // spends
            let invalid_anchor = tct::Tree::new().root();

            // Unregulated: use BLACK_HOLE_ACK with real encryption
            let black_hole_ack = penumbra_sdk_keys::keys::AddressComplianceKey::new(*penumbra_sdk_compliance::BLACK_HOLE_ACK);
            let user_leaf = penumbra_sdk_compliance::ComplianceLeaf {
                address: sender.clone(),
                key: black_hole_ack.clone(),
                asset_id: value_to_send.asset_id,
            };

            // Generate valid compliance ciphertext using real encryption with BLACK_HOLE_ACK
            let mut rng = rand::thread_rng();
            let timestamp = current_timestamp();
            let date = timestamp / 86400;
            let (compliance_ciphertext_obj, ephemeral_secret) = encrypt_compliance_details(
                &mut rng,
                &black_hole_ack,
                &sender,
                date,
                note.asset_id(),
                note.amount(),
                sender.clone(),
            ).expect("can encrypt compliance details");

            let (compliance_epk, packed_ciphertext) = compliance_ciphertext_obj.to_circuit_public_inputs();

            // Use sender as counterparty for testing (self-send scenario)
            let counterparty_leaf = penumbra_sdk_compliance::ComplianceLeaf {
                address: sender.clone(),
                key: black_hole_ack,
                asset_id: value_to_send.asset_id,
            };
            let tx_blinding_nonce = Fr::from(0u64);
            let sender_leaf_hash = penumbra_sdk_compliance::blind_sender_leaf(user_leaf.commit(), tx_blinding_nonce);
            let counterparty_leaf_hash = penumbra_sdk_compliance::blind_counterparty_leaf(counterparty_leaf.commit(), tx_blinding_nonce);

            let public = SpendProofPublic {
                anchor: invalid_anchor,
                balance_commitment,
                nullifier,
                rk,
                asset_anchor: tct::StateCommitment(Fq::from(0u64)),
                compliance_anchor: tct::StateCommitment(Fq::from(0u64)),
                compliance_epk,
                compliance_ciphertext: packed_ciphertext,
                target_timestamp: timestamp,
                sender_leaf_hash,
                counterparty_leaf_hash,
            };

            let private = SpendProofPrivate {
                state_commitment_proof,
                note,
                v_blinding,
                spend_auth_randomizer,
                ak,
                nk,
                asset_path: penumbra_sdk_compliance::MerklePath::default(),
                asset_position: 0,
                is_regulated: false,
                compliance_path: penumbra_sdk_compliance::MerklePath::default(),
                compliance_position: 0,
                user_leaf,
                compliance_ephemeral_secret: ephemeral_secret,
                counterparty_leaf,
                tx_blinding_nonce,
            };
            (public, private)
        }
    }

    proptest! {
        #[test]
        /// Check that the `SpendCircuit` is always satisfied for dummy (zero value) spends.
        fn spend_proof_dummy_verification_suceeds((public, private) in arb_valid_dummy_spend_statement()) {
            assert!(check_satisfaction(&public, &private).is_ok());
            assert!(check_circuit_satisfaction(public, private).is_ok());
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
            let note_commitment = make_random_note_commitment(address.clone());
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
            let note_commitment = make_random_note_commitment(address.clone());
            sct.insert(tct::Witness::Forget, note_commitment).unwrap();
        }

        for _ in 0..5 {
            let note_commitment = make_random_note_commitment(address.clone());
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
            let note_commitment = make_random_note_commitment(address.clone());
            sct.insert(tct::Witness::Forget, note_commitment).unwrap();
        }

        for _ in 0..5 {
            let note_commitment = make_random_note_commitment(address.clone());
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

    #[test]
    fn point_compression_roundtrip_for_ecdh() {
        // Verify that compress_to_field() used in ECDH is deterministic and doesn't lose information
        let cvk_sk = Fr::from(999u64);
        let cvk_pk = decaf377::Element::GENERATOR * cvk_sk;

        let ephemeral_sk = Fr::from(12345u64);
        let ephemeral_pk = decaf377::Element::GENERATOR * ephemeral_sk;

        // Prover side: shared_secret = cvk_pk * ephemeral_sk
        let shared_secret_prover = cvk_pk * ephemeral_sk;
        let compressed_prover = shared_secret_prover.vartime_compress_to_field();

        // Issuer side: shared_secret = ephemeral_pk * cvk_sk
        let shared_secret_issuer = ephemeral_pk * cvk_sk;
        let compressed_issuer = shared_secret_issuer.vartime_compress_to_field();

        // Both should produce identical compressed field elements
        assert_eq!(
            compressed_prover, compressed_issuer,
            "ECDH shared secrets must match"
        );

        // Verify compression is deterministic
        let compressed_again = shared_secret_prover.vartime_compress_to_field();
        assert_eq!(
            compressed_prover, compressed_again,
            "Compression must be deterministic"
        );
    }
}
