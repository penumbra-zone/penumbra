use base64::prelude::*;
use std::str::FromStr;

use anyhow::Result;
use ark_groth16::r1cs_to_qap::LibsnarkReduction;
use ark_r1cs_std::uint8::UInt8;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use decaf377::{Bls12_377, Fq, Fr};
use decaf377_fmd as fmd;
use decaf377_ka as ka;

use ark_ff::ToConstraintField;
use ark_groth16::{Groth16, PreparedVerifyingKey, Proof, ProvingKey};
use ark_r1cs_std::prelude::*;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef};
use ark_snark::SNARK;
use penumbra_sdk_keys::{keys::Diversifier, Address};
use penumbra_sdk_proto::{penumbra::core::component::shielded_pool::v1 as pb, DomainType};
use penumbra_sdk_tct::r1cs::StateCommitmentVar;
use penumbra_sdk_tct::StateCommitment;

use crate::{note, Note, Rseed};
use decaf377::r1cs::{ElementVar, FqVar};
use penumbra_sdk_asset::{
    balance,
    balance::{commitment::BalanceCommitmentVar, BalanceVar},
    Value,
};
use penumbra_sdk_compliance::r1cs::{verify_compliance_integrity, ComplianceWitness};
use penumbra_sdk_proof_params::{DummyWitness, VerifyingKeyExt, GROTH16_PROOF_LENGTH_BYTES};

/// The public input for an [`OutputProof`].
#[derive(Clone, Debug)]
pub struct OutputProofPublic {
    /// A hiding commitment to the balance.
    pub balance_commitment: balance::Commitment,
    /// A hiding commitment to the note.
    pub note_commitment: note::StateCommitment,
    /// Ephemeral public key from the compliance ciphertext
    pub compliance_epk: decaf377::Element,
    /// Compliance ciphertext for regulated assets (packed as Fq elements for circuit input)
    pub compliance_ciphertext: Vec<Fq>,
    /// Asset registry Merkle root (asset regulation tree anchor)
    pub asset_anchor: StateCommitment,
    /// User compliance registry Merkle root (user tree anchor)
    pub compliance_anchor: StateCommitment,
    /// Target timestamp for key derivation (Unix timestamp in seconds)
    pub target_timestamp: u64,
    /// Hash of the receiver's compliance leaf (for binding with spend circuit)
    pub receiver_leaf_hash: StateCommitment,
    /// Hash of the counterparty's (sender's) compliance leaf (for binding with spend circuit)
    pub counterparty_leaf_hash: StateCommitment,
}

/// The private input for an [`OutputProof`].
#[derive(Clone, Debug)]
pub struct OutputProofPrivate {
    /// The note being created.
    pub note: Note,
    /// A blinding factor to hide the balance of the transaction.
    pub balance_blinding: Fr,
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
    /// The counterparty's (sender's) compliance leaf (for extracting address and binding)
    pub counterparty_leaf: penumbra_sdk_compliance::ComplianceLeaf,
    /// Shared transaction blinding nonce (same for spend and output in one transaction)
    pub tx_blinding_nonce: Fr,
}

#[cfg(test)]
fn check_satisfaction(public: &OutputProofPublic, private: &OutputProofPrivate) -> Result<()> {
    use penumbra_sdk_asset::Balance;

    if private.note.diversified_generator() == decaf377::Element::default() {
        anyhow::bail!("diversified generator is identity");
    }

    let balance_commitment =
        (-Balance::from(private.note.value())).commit(private.balance_blinding);
    if balance_commitment != public.balance_commitment {
        anyhow::bail!("balance commitment did not match public input");
    }

    let note_commitment = private.note.commit();
    if note_commitment != public.note_commitment {
        anyhow::bail!("note commitment did not match public input");
    }

    Ok(())
}

#[cfg(test)]
fn check_circuit_satisfaction(
    public: OutputProofPublic,
    private: OutputProofPrivate,
) -> Result<()> {
    use ark_relations::r1cs::{self, ConstraintSystem};

    let cs = ConstraintSystem::new_ref();
    let circuit = OutputCircuit { public, private };
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

/// Public Inputs:
/// * balance_commitment (vcm)
/// * note_commitment (ncm)
/// * asset_anchor (Merkle root of the asset registry)
/// * compliance_anchor (Merkle root of the compliance user tree)
/// * compliance_ciphertext (5 field elements: 8-byte Clue + 147-byte Payload)
///
/// Witnesses:
/// * Note Data:
///   - g_d (diversified generator)
///   - pk_d (transmission key)
///   - value (amount + asset_id)
///   - v_blinding (Fr)
///   - note_blinding (Fq)
/// * Compliance & Registry:
///   - cvk_pk (For Payload Decryption: shared_secret = r * CVK_PK)
///   - asset_path (index of asset in registry)
///   - compliance_path (Merkle path for compliance verification)
#[derive(Clone, Debug)]
pub struct OutputCircuit {
    public: OutputProofPublic,
    private: OutputProofPrivate,
}

impl OutputCircuit {
    fn new(public: OutputProofPublic, private: OutputProofPrivate) -> Self {
        Self { public, private }
    }

    pub fn into_parts(self) -> (OutputProofPublic, OutputProofPrivate) {
        (self.public, self.private)
    }
}

impl ConstraintSynthesizer<Fq> for OutputCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fq>) -> ark_relations::r1cs::Result<()> {
        // Witnesses
        // Note: In the allocation of the address on `NoteVar`, we check the diversified base is not identity.
        let note_var = note::NoteVar::new_witness(cs.clone(), || Ok(self.private.note.clone()))?;
        let balance_blinding_arr: [u8; 32] = self.private.balance_blinding.to_bytes();
        let balance_blinding_vars = UInt8::new_witness_vec(cs.clone(), &balance_blinding_arr)?;

        // Public inputs
        let claimed_note_commitment =
            StateCommitmentVar::new_input(cs.clone(), || Ok(self.public.note_commitment))?;
        let claimed_balance_commitment =
            BalanceCommitmentVar::new_input(cs.clone(), || Ok(self.public.balance_commitment))?;
        let claimed_asset_anchor =
            StateCommitmentVar::new_input(cs.clone(), || Ok(self.public.asset_anchor))?;
        let claimed_compliance_anchor =
            StateCommitmentVar::new_input(cs.clone(), || Ok(self.public.compliance_anchor))?;

        // Check integrity of balance commitment (negative for outputs).
        let balance_commitment =
            BalanceVar::from_negative_value_var(note_var.value()).commit(balance_blinding_vars)?;
        balance_commitment.enforce_equal(&claimed_balance_commitment)?;

        // Check note commitment integrity.
        let note_commitment = note_var.commit()?;
        note_commitment.enforce_equal(&claimed_note_commitment)?;

        let target_day_index = self.public.target_timestamp / 86400;
        let target_date_var = FqVar::new_input(cs.clone(), || Ok(Fq::from(target_day_index)))?;

        let _is_regulated_var = Boolean::new_witness(cs.clone(), || Ok(self.private.is_regulated))?;
        let _asset_position_var =
            FqVar::new_witness(cs.clone(), || Ok(Fq::from(self.private.asset_position)))?;
        let _compliance_position_var = FqVar::new_witness(cs.clone(), || {
            Ok(Fq::from(self.private.compliance_position))
        })?;

        let public_ciphertext_epk_var =
            ElementVar::new_input(cs.clone(), || Ok(self.public.compliance_epk))?;

        let mut ciphertext_vars = Vec::new();
        for fq in self.public.compliance_ciphertext.iter() {
            ciphertext_vars.push(FqVar::new_input(cs.clone(), || Ok(*fq))?);
        }

        let counterparty_leaf_var =
            penumbra_sdk_compliance::r1cs::ComplianceLeafVar::new_witness(cs.clone(), || {
                Ok(self.private.counterparty_leaf.clone())
            })?;

        let compliance_witness = ComplianceWitness {
            is_regulated: self.private.is_regulated,
            asset_path: self.private.asset_path.clone(),
            asset_position: self.private.asset_position,
            compliance_path: self.private.compliance_path.clone(),
            compliance_position: self.private.compliance_position,
            user_leaf: self.private.user_leaf.clone(),
        };

        verify_compliance_integrity(
            cs.clone(),
            claimed_asset_anchor.inner().clone(),
            claimed_compliance_anchor.inner().clone(),
            target_date_var,
            public_ciphertext_epk_var,
            ciphertext_vars,
            note_var.asset_id(),
            note_var.amount(),
            note_var.diversified_generator(),
            note_var.transmission_key(),
            counterparty_leaf_var.address.diversified_generator.clone(),
            counterparty_leaf_var.address.transmission_key.clone(),
            self.private.compliance_ephemeral_secret,
            compliance_witness,
        )?;

        let tx_blinding_nonce_var = FqVar::new_witness(cs.clone(), || {
            Ok(Fq::from_le_bytes_mod_order(
                &self.private.tx_blinding_nonce.to_bytes(),
            ))
        })?;

        let receiver_leaf_var =
            penumbra_sdk_compliance::r1cs::ComplianceLeafVar::new_witness(cs.clone(), || {
                Ok(self.private.user_leaf.clone())
            })?;
        let receiver_leaf_hash = receiver_leaf_var.commit(cs.clone())?;

        // Receiver uses COUNTERPARTY domain sep (matches spend's counterparty).
        let computed_blinded_receiver =
            penumbra_sdk_compliance::leaf_binding::r1cs::blind_counterparty_leaf(
                cs.clone(),
                receiver_leaf_hash,
                tx_blinding_nonce_var.clone(),
            )?;
        let claimed_blinded_receiver =
            FqVar::new_input(cs.clone(), || Ok(self.public.receiver_leaf_hash.0))?;
        computed_blinded_receiver.enforce_equal(&claimed_blinded_receiver)?;

        let counterparty_leaf_hash = counterparty_leaf_var.commit(cs.clone())?;

        // Counterparty uses SENDER domain sep (matches spend's sender).
        let computed_blinded_counterparty =
            penumbra_sdk_compliance::leaf_binding::r1cs::blind_sender_leaf(
                cs.clone(),
                counterparty_leaf_hash,
                tx_blinding_nonce_var,
            )?;
        let claimed_blinded_counterparty =
            FqVar::new_input(cs.clone(), || Ok(self.public.counterparty_leaf_hash.0))?;
        computed_blinded_counterparty.enforce_equal(&claimed_blinded_counterparty)?;

        Ok(())
    }
}

impl DummyWitness for OutputCircuit {
    fn with_dummy_witness() -> Self {
        use penumbra_sdk_compliance::crypto::encrypt_compliance_details;
        use penumbra_sdk_keys::keys::AddressComplianceKey;

        let diversifier_bytes = [1u8; 16];
        let pk_d_bytes = decaf377::Element::GENERATOR.vartime_compress().0;
        let clue_key_bytes = [1; 32];
        let diversifier = Diversifier(diversifier_bytes);
        let address = Address::from_components(
            diversifier,
            ka::Public(pk_d_bytes),
            fmd::ClueKey(clue_key_bytes),
        )
        .expect("generated 1 address");
        let note = Note::from_parts(
            address.clone(),
            Value::from_str("1upenumbra").expect("valid value"),
            Rseed([1u8; 32]),
        )
        .expect("can make a note");
        let balance_blinding = Fr::from(1u64);

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

        let asset_anchor = penumbra_sdk_tct::StateCommitment(Fq::from(0u64));
        let compliance_anchor = penumbra_sdk_tct::StateCommitment(Fq::from(0u64));

        let dummy_leaf = penumbra_sdk_compliance::ComplianceLeaf {
            address: address.clone(),
            key: dummy_ack.clone(),
            asset_id: penumbra_sdk_asset::asset::Id(Fq::from(0u64)),
        };
        // Note: receiver uses COUNTERPARTY domain sep, counterparty uses SENDER domain sep
        // This matches the circuit constraints in generate_constraints()
        let receiver_leaf_hash =
            penumbra_sdk_compliance::blind_counterparty_leaf(dummy_leaf.commit(), Fr::from(0u64));
        let counterparty_leaf_hash =
            penumbra_sdk_compliance::blind_sender_leaf(dummy_leaf.commit(), Fr::from(0u64));

        let public = OutputProofPublic {
            note_commitment: note.commit(),
            balance_commitment: balance::Commitment(decaf377::Element::GENERATOR),
            compliance_epk,
            compliance_ciphertext,
            asset_anchor,
            compliance_anchor,
            target_timestamp: 0, // Corresponds to date = 0
            receiver_leaf_hash,
            counterparty_leaf_hash,
        };
        let private = OutputProofPrivate {
            note,
            balance_blinding,
            asset_path: penumbra_sdk_compliance::MerklePath::default(),
            asset_position: 0,
            is_regulated: false,
            compliance_path: penumbra_sdk_compliance::MerklePath::default(),
            compliance_position: 0,
            user_leaf: penumbra_sdk_compliance::ComplianceLeaf {
                address: address,
                key: dummy_ack,
                asset_id: penumbra_sdk_asset::asset::Id(Fq::from(0u64)),
            },
            compliance_ephemeral_secret: ephemeral_secret,
            counterparty_leaf: dummy_leaf,
            tx_blinding_nonce: Fr::from(0u64),
        };
        OutputCircuit { public, private }
    }
}

#[derive(Clone, Debug)]
pub struct OutputProof([u8; GROTH16_PROOF_LENGTH_BYTES]);

impl OutputProof {
    #![allow(clippy::too_many_arguments)]
    pub fn prove(
        blinding_r: Fq,
        blinding_s: Fq,
        pk: &ProvingKey<Bls12_377>,
        public: OutputProofPublic,
        private: OutputProofPrivate,
    ) -> Result<Self, crate::ProofError> {
        let circuit = OutputCircuit::new(public, private);

        // Check constraint satisfaction before proving
        use ark_relations::r1cs::ConstraintSystem;
        let cs = ConstraintSystem::<Fq>::new_ref();
        circuit.clone().generate_constraints(cs.clone())?;

        if !cs.is_satisfied().unwrap_or(false) {
            return Err(crate::ProofError::UnsatisfiedConstraints(
                "Output circuit constraints not satisfied. Possible causes: \
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

    #[tracing::instrument(level="debug", skip(self, vk), fields(self = ?BASE64_STANDARD.encode(self.clone().encode_to_vec()), vk = ?vk.debug_id()))]
    pub fn verify(
        &self,
        vk: &PreparedVerifyingKey<Bls12_377>,
        public: OutputProofPublic,
    ) -> anyhow::Result<()> {
        let proof =
            Proof::deserialize_compressed_unchecked(&self.0[..]).map_err(|e| anyhow::anyhow!(e))?;
        let mut public_inputs = Vec::new();

        public_inputs.extend(
            public
                .note_commitment
                .0
                .to_field_elements()
                .ok_or_else(|| anyhow::anyhow!("note commitment invalid"))?,
        );
        public_inputs.extend(
            public
                .balance_commitment
                .0
                .to_field_elements()
                .ok_or_else(|| anyhow::anyhow!("balance commitment invalid"))?,
        );
        public_inputs.extend(
            public
                .asset_anchor
                .0
                .to_field_elements()
                .ok_or_else(|| anyhow::anyhow!("asset anchor invalid"))?,
        );
        public_inputs.extend(
            public
                .compliance_anchor
                .0
                .to_field_elements()
                .ok_or_else(|| anyhow::anyhow!("compliance anchor invalid"))?,
        );

        // 1. Target Date (Day Index) - Must match circuit allocation order!
        // The circuit computes `timestamp / 86400` during witness generation,
        // so the public input must be the day index, not the raw timestamp.
        let day_index = public.target_timestamp / 86400;
        public_inputs.extend(
            Fq::from(day_index)
                .to_field_elements()
                .ok_or_else(|| anyhow::anyhow!("target timestamp invalid"))?,
        );

        // 2. Compliance EPK - Must follow Target Date
        public_inputs.extend(
            public
                .compliance_epk
                .to_field_elements()
                .ok_or_else(|| anyhow::anyhow!("compliance epk invalid"))?,
        );

        // 3. Add packed compliance_ciphertext to public inputs (already in Fq format)
        public_inputs.extend(public.compliance_ciphertext);

        // 4. Add blinded leaf hashes to public inputs
        public_inputs.extend(
            public
                .receiver_leaf_hash
                .0
                .to_field_elements()
                .ok_or_else(|| anyhow::anyhow!("receiver leaf hash invalid"))?,
        );
        public_inputs.extend(
            public
                .counterparty_leaf_hash
                .0
                .to_field_elements()
                .ok_or_else(|| anyhow::anyhow!("counterparty leaf hash invalid"))?,
        );

        let proof_result = Groth16::<Bls12_377, LibsnarkReduction>::verify_with_processed_vk(
            vk,
            public_inputs.as_slice(),
            &proof,
        )
        .map_err(|err| anyhow::anyhow!(err))?;

        proof_result
            .then_some(())
            .ok_or_else(|| anyhow::anyhow!("output proof did not verify"))
    }
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
    use decaf377::{Fq, Fr};
    use penumbra_sdk_asset::{asset, Balance, Value};
    use penumbra_sdk_keys::keys::{Bip44Path, SeedPhrase, SpendKey};
    use penumbra_sdk_num::Amount;
    use penumbra_sdk_tct as tct; // FIX: Added import
    use proptest::prelude::*;

    use crate::test_proof_helpers::proof_test_helpers::{
        current_timestamp, mock_compliance_inputs,
    };
    use crate::{note, Note};

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

    prop_compose! {
        fn arb_valid_output_statement()(
            seed_phrase_randomness in any::<[u8; 32]>(),
            rseed_randomness in any::<[u8; 32]>(),
            amount in any::<u64>(),
            balance_blinding in fr_strategy(),
            asset_id64 in any::<u64>(),
            address_index in any::<u32>(),
            cvk_sk_rand in any::<[u8; 32]>()
        ) -> (OutputProofPublic, OutputProofPrivate, Fr) {
            use penumbra_sdk_compliance::crypto::encrypt_compliance_details;
            use penumbra_sdk_keys::keys::AddressComplianceKey;

            let seed_phrase = SeedPhrase::from_randomness(&seed_phrase_randomness);
            let sk_recipient = SpendKey::from_seed_phrase_bip44(seed_phrase, &Bip44Path::new(0));
            let fvk_recipient = sk_recipient.full_viewing_key();
            let ivk_recipient = fvk_recipient.incoming();
            let (dest, _dtk_d) = ivk_recipient.payment_address(address_index.into());

            let value_to_send = Value {
                amount: Amount::from(amount),
                asset_id: asset::Id(Fq::from(asset_id64)),
            };
            let note = Note::from_parts(
                dest.clone(),
                value_to_send,
                Rseed(rseed_randomness),
            ).expect("should be able to create note");
            let note_commitment = note.commit();
            let balance_commitment = (-Balance::from(value_to_send)).commit(balance_blinding);

            // ACK-based compliance (use real ACK for regulated asset)
            let cvk_sk = Fr::from_le_bytes_mod_order(&cvk_sk_rand);
            let ack_point = decaf377::Element::GENERATOR * cvk_sk;
            let user_ack = AddressComplianceKey::new(ack_point);
            let user_leaf = penumbra_sdk_compliance::ComplianceLeaf {
                address: dest.clone(),
                key: user_ack.clone(),
                asset_id: value_to_send.asset_id,
            };

            let asset_anchor = tct::StateCommitment(Fq::from(0u64));
            let compliance_anchor = tct::StateCommitment(Fq::from(0u64));

            // Generate valid compliance ciphertext using real encryption
            let mut rng = rand::thread_rng();
            let timestamp = current_timestamp();
            let date = timestamp / 86400;
            let (compliance_ciphertext_obj, ephemeral_secret) = encrypt_compliance_details(
                &mut rng,
                &user_ack,
                &dest,
                date,
                note.asset_id(),
                note.amount(),
                dest.clone(), // Use same address as counterparty for test
            ).expect("can encrypt compliance details");

            // Extract circuit inputs using unified method
            let (compliance_epk, packed_ciphertext) = compliance_ciphertext_obj.to_circuit_public_inputs();

            // Create dummy leaves for testing
            let dummy_leaf = penumbra_sdk_compliance::ComplianceLeaf {
                address: dest.clone(),
                key: user_ack.clone(),
                asset_id: value_to_send.asset_id,
            };
            let dummy_nonce = Fr::from(0u64);
            let receiver_leaf_hash = penumbra_sdk_compliance::blind_counterparty_leaf(dummy_leaf.commit(), dummy_nonce);
            let counterparty_leaf_hash = penumbra_sdk_compliance::blind_sender_leaf(dummy_leaf.commit(), dummy_nonce);

            let public = OutputProofPublic {
                balance_commitment,
                note_commitment,
                compliance_epk,
                compliance_ciphertext: packed_ciphertext,
                asset_anchor,
                compliance_anchor,
                target_timestamp: timestamp,
                receiver_leaf_hash,
                counterparty_leaf_hash,
            };
            let private = OutputProofPrivate {
                note,
                balance_blinding,
                asset_path: penumbra_sdk_compliance::MerklePath::default(),
                asset_position: 0,
                is_regulated: true,
                compliance_path: penumbra_sdk_compliance::MerklePath::default(),
                compliance_position: 0,
                user_leaf,
                compliance_ephemeral_secret: ephemeral_secret,
                counterparty_leaf: dummy_leaf,
                tx_blinding_nonce: dummy_nonce,
            };

            (public, private, cvk_sk)
        }
    }

    prop_compose! {
        fn arb_unregulated_output_statement()(
            seed_phrase_randomness in any::<[u8; 32]>(),
            rseed_randomness in any::<[u8; 32]>(),
            amount in any::<u64>(),
            balance_blinding in fr_strategy(),
            asset_id64 in any::<u64>(),
            address_index in any::<u32>(),
            tester_keys_rand in any::<[u8; 32]>()
        ) -> (OutputProofPublic, OutputProofPrivate, Fr) {
            use penumbra_sdk_compliance::crypto::encrypt_compliance_details;
            use penumbra_sdk_keys::keys::AddressComplianceKey;

            let seed_phrase = SeedPhrase::from_randomness(&seed_phrase_randomness);
            let sk_recipient = SpendKey::from_seed_phrase_bip44(seed_phrase, &Bip44Path::new(0));
            let fvk_recipient = sk_recipient.full_viewing_key();
            let ivk_recipient = fvk_recipient.incoming();
            let (dest, _dtk_d) = ivk_recipient.payment_address(address_index.into());

            let value_to_send = Value {
                amount: Amount::from(amount),
                asset_id: asset::Id(Fq::from(asset_id64)),
            };
            let note = Note::from_parts(
                dest.clone(),
                value_to_send,
                Rseed(rseed_randomness),
            ).expect("should be able to create note");
            let note_commitment = note.commit();
            let balance_commitment = (-Balance::from(value_to_send)).commit(balance_blinding);

            // Unregulated: use BLACK_HOLE_ACK
            let black_hole_ack = AddressComplianceKey::new(*penumbra_sdk_compliance::BLACK_HOLE_ACK);
            let user_leaf = penumbra_sdk_compliance::ComplianceLeaf {
                address: dest.clone(),
                key: black_hole_ack.clone(),
                asset_id: value_to_send.asset_id,
            };

            let asset_anchor = tct::StateCommitment(Fq::from(0u64));
            let compliance_anchor = tct::StateCommitment(Fq::from(0u64));

            // Generate valid compliance ciphertext using real encryption with BLACK_HOLE_ACK
            let mut rng = rand::thread_rng();
            let timestamp = current_timestamp();
            let date = timestamp / 86400;
            let (compliance_ciphertext_obj, ephemeral_secret) = encrypt_compliance_details(
                &mut rng,
                &black_hole_ack,
                &dest,
                date,
                note.asset_id(),
                note.amount(),
                dest.clone(),
            ).expect("can encrypt compliance details");

            let (compliance_epk, packed_ciphertext) = compliance_ciphertext_obj.to_circuit_public_inputs();

            // Create dummy leaves for blinding
            let dummy_leaf = user_leaf.clone();
            let dummy_nonce = Fr::from(0u64);
            let receiver_leaf_hash = penumbra_sdk_compliance::blind_counterparty_leaf(dummy_leaf.commit(), dummy_nonce);
            let counterparty_leaf_hash = penumbra_sdk_compliance::blind_sender_leaf(dummy_leaf.commit(), dummy_nonce);

            let public = OutputProofPublic {
                balance_commitment,
                note_commitment,
                compliance_epk,
                compliance_ciphertext: packed_ciphertext,
                asset_anchor,
                compliance_anchor,
                target_timestamp: timestamp,
                receiver_leaf_hash,
                counterparty_leaf_hash,
            };
            let private = OutputProofPrivate {
                note,
                balance_blinding,
                asset_path: penumbra_sdk_compliance::MerklePath::default(),
                asset_position: 0,
                is_regulated: false,
                compliance_path: penumbra_sdk_compliance::MerklePath::default(),
                compliance_position: 0,
                user_leaf: user_leaf.clone(),
                compliance_ephemeral_secret: ephemeral_secret,
                counterparty_leaf: dummy_leaf.clone(),
                tx_blinding_nonce: dummy_nonce,
            };

            let my_random_sk_scalar = Fr::from_le_bytes_mod_order(&tester_keys_rand);
            let my_fake_cvk_sk = my_random_sk_scalar;

            (public, private, my_fake_cvk_sk)
        }
    }

    proptest! {
        #![proptest_config(proptest::test_runner::Config::with_cases(1))]
        #[test]
        fn output_proof_happy_path((public, private, _cvk_sk) in arb_valid_output_statement()) {
            assert!(check_satisfaction(&public, &private).is_ok());
            assert!(check_circuit_satisfaction(public, private).is_ok());
        }
    }

    proptest! {
        #![proptest_config(proptest::test_runner::Config::with_cases(1))]
        #[test]
        fn output_proof_unregulated_asset((public, private, _cvk_sk) in arb_unregulated_output_statement()) {
            assert!(check_satisfaction(&public, &private).is_ok());
            assert!(check_circuit_satisfaction(public, private).is_ok());
        }
    }

    #[test]
    fn output_proof_full_groth16_roundtrip_regulated() {
        use crate::test_proof_helpers::proof_test_helpers::{full_groth16_roundtrip, CircuitType};
        full_groth16_roundtrip(CircuitType::Output, true);
    }

    #[test]
    fn output_proof_full_groth16_roundtrip_unregulated() {
        use crate::test_proof_helpers::proof_test_helpers::{full_groth16_roundtrip, CircuitType};
        full_groth16_roundtrip(CircuitType::Output, false);
    }

    prop_compose! {
        fn arb_invalid_output_note_commitment_integrity()(
            seed_phrase_randomness in any::<[u8; 32]>(),
            rseed_randomness in any::<[u8; 32]>(),
            amount in any::<u64>(),
            balance_blinding in fr_strategy(),
            asset_id64 in any::<u64>(),
            address_index in any::<u32>(),
            incorrect_note_blinding in fq_strategy(),
            cvk_sk_rand in any::<[u8; 32]>()
        ) -> (OutputProofPublic, OutputProofPrivate) {
            let seed_phrase = SeedPhrase::from_randomness(&seed_phrase_randomness);
            let sk_recipient = SpendKey::from_seed_phrase_bip44(seed_phrase, &Bip44Path::new(0));
            let fvk_recipient = sk_recipient.full_viewing_key();
            let ivk_recipient = fvk_recipient.incoming();
            let (dest, _dtk_d) = ivk_recipient.payment_address(address_index.into());

            let value_to_send = Value {
                amount: Amount::from(amount),
                asset_id: asset::Id(Fq::from(asset_id64)),
            };
            let note = Note::from_parts(
                dest.clone(),
                value_to_send,
                Rseed(rseed_randomness),
            ).expect("should be able to create note");
            let balance_commitment = (-Balance::from(value_to_send)).commit(balance_blinding);

            let incorrect_note_commitment = note::commitment(
                incorrect_note_blinding,
                value_to_send,
                note.diversified_generator(),
                note.transmission_key_s(),
                note.clue_key(),
            );

            let cvk_sk = Fr::from_le_bytes_mod_order(&cvk_sk_rand);
            let ack_point = decaf377::Element::GENERATOR * cvk_sk;
            let user_leaf = penumbra_sdk_compliance::ComplianceLeaf {
                address: dest,
                key: penumbra_sdk_keys::keys::AddressComplianceKey::new(ack_point),
                asset_id: value_to_send.asset_id,
            };

            let asset_anchor = tct::StateCommitment(Fq::from(0u64));
            let compliance_anchor = tct::StateCommitment(Fq::from(0u64));
            let (compliance_epk, packed_ciphertext) = mock_compliance_inputs();

            // Create dummy leaves for blinding
            let dummy_leaf = user_leaf.clone();
            let dummy_nonce = Fr::from(0u64);
            let receiver_leaf_hash = penumbra_sdk_compliance::blind_counterparty_leaf(dummy_leaf.commit(), dummy_nonce);
            let counterparty_leaf_hash = penumbra_sdk_compliance::blind_sender_leaf(dummy_leaf.commit(), dummy_nonce);

            let bad_public = OutputProofPublic {
                balance_commitment,
                note_commitment: incorrect_note_commitment,
                compliance_epk,
                compliance_ciphertext: packed_ciphertext,
                asset_anchor,
                compliance_anchor,
                target_timestamp: current_timestamp(),
                receiver_leaf_hash,
                counterparty_leaf_hash,
            };
            let private = OutputProofPrivate {
                note,
                balance_blinding,
                asset_path: penumbra_sdk_compliance::MerklePath::default(),
                asset_position: 0,
                is_regulated: false,
                compliance_path: penumbra_sdk_compliance::MerklePath::default(),
                compliance_position: 0,
                user_leaf: user_leaf.clone(),
                compliance_ephemeral_secret: Fr::from(0u64),
                counterparty_leaf: dummy_leaf.clone(),
                tx_blinding_nonce: dummy_nonce,
            };

            (bad_public, private)
        }
    }

    proptest! {
        #[test]
        fn output_proof_verification_fails_note_commitment_integrity((public, private) in arb_invalid_output_note_commitment_integrity()) {
            assert!(check_satisfaction(&public, &private).is_err());
            assert!(check_circuit_satisfaction(public, private).is_err());
        }
    }

    prop_compose! {
        fn arb_invalid_output_balance_commitment_integrity()(
            seed_phrase_randomness in any::<[u8; 32]>(),
            rseed_randomness in any::<[u8; 32]>(),
            amount in any::<u64>(),
            balance_blinding in fr_strategy(),
            asset_id64 in any::<u64>(),
            address_index in any::<u32>(),
            incorrect_v_blinding in fr_strategy(),
            cvk_sk_rand in any::<[u8; 32]>()
        ) -> (OutputProofPublic, OutputProofPrivate) {
            let seed_phrase = SeedPhrase::from_randomness(&seed_phrase_randomness);
            let sk_recipient = SpendKey::from_seed_phrase_bip44(seed_phrase, &Bip44Path::new(0));
            let fvk_recipient = sk_recipient.full_viewing_key();
            let ivk_recipient = fvk_recipient.incoming();
            let (dest, _dtk_d) = ivk_recipient.payment_address(address_index.into());

            let value_to_send = Value {
                amount: Amount::from(amount),
                asset_id: asset::Id(Fq::from(asset_id64)),
            };
            let note = Note::from_parts(
                dest.clone(),
                value_to_send,
                Rseed(rseed_randomness),
            ).expect("should be able to create note");
            let note_commitment = note.commit();

            let incorrect_balance_commitment = (-Balance::from(value_to_send)).commit(incorrect_v_blinding);

            let cvk_sk = Fr::from_le_bytes_mod_order(&cvk_sk_rand);
            let ack_point = decaf377::Element::GENERATOR * cvk_sk;
            let user_leaf = penumbra_sdk_compliance::ComplianceLeaf {
                address: dest,
                key: penumbra_sdk_keys::keys::AddressComplianceKey::new(ack_point),
                asset_id: value_to_send.asset_id,
            };

            let asset_anchor = tct::StateCommitment(Fq::from(0u64));
            let compliance_anchor = tct::StateCommitment(Fq::from(0u64));
            let (compliance_epk, packed_ciphertext) = mock_compliance_inputs();

            // Create dummy leaves for blinding
            let dummy_leaf = user_leaf.clone();
            let dummy_nonce = Fr::from(0u64);
            let receiver_leaf_hash = penumbra_sdk_compliance::blind_counterparty_leaf(dummy_leaf.commit(), dummy_nonce);
            let counterparty_leaf_hash = penumbra_sdk_compliance::blind_sender_leaf(dummy_leaf.commit(), dummy_nonce);

            let bad_public = OutputProofPublic {
                balance_commitment: incorrect_balance_commitment,
                note_commitment,
                compliance_epk,
                compliance_ciphertext: packed_ciphertext,
                asset_anchor,
                compliance_anchor,
                target_timestamp: current_timestamp(),
                receiver_leaf_hash,
                counterparty_leaf_hash,
            };

            let private = OutputProofPrivate {
                note,
                balance_blinding,
                asset_path: penumbra_sdk_compliance::MerklePath::default(),
                asset_position: 0,
                is_regulated: false,
                compliance_path: penumbra_sdk_compliance::MerklePath::default(),
                compliance_position: 0,
                user_leaf: user_leaf.clone(),
                compliance_ephemeral_secret: Fr::from(0u64),
                counterparty_leaf: dummy_leaf.clone(),
                tx_blinding_nonce: dummy_nonce,
            };

            (bad_public, private)
        }
    }

    proptest! {
        #[test]
        fn output_proof_verification_fails_balance_commitment_integrity((public, private) in arb_invalid_output_balance_commitment_integrity()) {
            assert!(check_satisfaction(&public, &private).is_err());
            assert!(check_circuit_satisfaction(public, private).is_err());
        }
    }
}
