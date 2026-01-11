//! Common test helpers for spend and output proof tests.

pub mod proof_test_helpers {
    /// Test asset ID for regulated assets
    pub const REGULATED_ASSET_ID: u64 = 1;
    /// Test asset ID for unregulated assets
    pub const UNREGULATED_ASSET_ID: u64 = 2;

    use ark_groth16::{Groth16, PreparedVerifyingKey, ProvingKey};
    use ark_snark::SNARK;
    use decaf377::{Bls12_377, Fq, Fr};
    use penumbra_sdk_asset::{asset, Value};
    use penumbra_sdk_keys::keys::{Bip44Path, SeedPhrase, SpendKey};
    use penumbra_sdk_tct as tct;
    use rand_core::OsRng;

    use crate::{Note, Rseed};

    /// Get current Unix timestamp in seconds
    pub fn current_timestamp() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time should be after Unix epoch")
            .as_secs()
    }

    /// Common setup for proof tests: creates keys, note, and circuit proving/verifying keys
    pub fn setup_proof_test_with_circuit<C, F>(
        asset_id: u64,
        amount: u64,
        circuit_generator: F,
    ) -> (
        SpendKey,
        Note,
        tct::Proof,
        tct::Root,
        ProvingKey<Bls12_377>,
        PreparedVerifyingKey<Bls12_377>,
    )
    where
        C: Clone + ark_relations::r1cs::ConstraintSynthesizer<Fq>,
        F: FnOnce() -> C,
    {
        let mut rng = OsRng;

        // Generate circuit keys
        let circuit_template = circuit_generator();
        let (pk, vk) = Groth16::<Bls12_377>::circuit_specific_setup(circuit_template, &mut rng)
            .expect("can perform circuit setup");
        let pvk = ark_groth16::prepare_verifying_key(&vk);

        // Generate user keys and address
        let seed_phrase = SeedPhrase::generate(rng);
        let sk = SpendKey::from_seed_phrase_bip44(seed_phrase, &Bip44Path::new(0));
        let fvk = sk.full_viewing_key();
        let ivk = fvk.incoming();
        let (address, _dtk_d) = ivk.payment_address(0u32.into());

        // Create note
        let value = Value {
            amount: amount.into(),
            asset_id: asset::Id(Fq::from(asset_id)),
        };

        let note = Note::from_parts(address, value, Rseed::generate(&mut rng))
            .expect("should be able to create note");

        // Mock SCT
        let mut sct = tct::Tree::new();
        let note_commitment = note.commit();
        sct.insert(tct::Witness::Keep, note_commitment).unwrap();
        let anchor = sct.root();
        let state_commitment_proof = sct.witness(note_commitment).unwrap();

        (sk, note, state_commitment_proof, anchor, pk, pvk)
    }

    /// Create dummy compliance anchors for testing
    pub fn dummy_compliance_anchors() -> (tct::StateCommitment, tct::StateCommitment) {
        (
            tct::StateCommitment(Fq::from(0u64)),
            tct::StateCommitment(Fq::from(0u64)),
        )
    }

    /// Setup Groth16 proving and verifying keys for a circuit.
    ///
    /// Generic helper that performs circuit-specific setup for any proof circuit.
    ///
    /// # Type Parameters
    /// * `C` - Circuit type implementing DummyWitness
    ///
    /// # Returns
    /// Tuple of (ProvingKey, PreparedVerifyingKey, blinding_r, blinding_s)
    pub fn setup_groth16_keys<C>() -> (
        ark_groth16::ProvingKey<Bls12_377>,
        ark_groth16::PreparedVerifyingKey<Bls12_377>,
        Fq,
        Fq,
    )
    where
        C: penumbra_sdk_proof_params::DummyWitness + Clone,
    {
        use ark_groth16::Groth16;
        use ark_snark::SNARK;

        let mut rng = rand::thread_rng();

        // Circuit-specific setup
        let circuit_template = C::with_dummy_witness();
        let (pk, vk) = Groth16::<Bls12_377>::circuit_specific_setup(circuit_template, &mut rng)
            .expect("cannot perform setup");
        let pvk = ark_groth16::prepare_verifying_key(&vk);

        // Generate random blinding factors
        let blinding_r = Fq::rand(&mut rng);
        let blinding_s = Fq::rand(&mut rng);

        (pk, pvk, blinding_r, blinding_s)
    }

    /// Helper to create mock compliance inputs for testing.
    /// Returns (compliance_epk, packed_ciphertext) from dummy ciphertext.
    pub fn mock_compliance_inputs() -> (decaf377::Element, Vec<Fq>) {
        use penumbra_sdk_compliance::structs::{ComplianceCiphertext, TOTAL_WIRE_BYTES};

        let dummy_ciphertext = vec![0u8; TOTAL_WIRE_BYTES];

        let ct = ComplianceCiphertext::from_bytes(&dummy_ciphertext)
            .expect("can deserialize dummy ciphertext");
        ct.to_circuit_public_inputs()
    }

    /// Circuit type for unified testing
    #[derive(Debug, Clone, Copy)]
    pub enum CircuitType {
        Spend,
        Output,
    }

    /// Test data bundle containing note, keys, and compliance information.
    ///
    /// This unified structure is used by both groth16 and plan tests to ensure consistency.
    pub struct TestData {
        pub note: Note,
        pub address: penumbra_sdk_keys::Address,
        pub value: penumbra_sdk_asset::Value,
        pub balance_blinding: Fr,
        pub fvk: penumbra_sdk_keys::keys::FullViewingKey,
        pub sk: SpendKey,
        pub ack: penumbra_sdk_keys::keys::AddressComplianceKey,
        pub user_leaf: penumbra_sdk_compliance::ComplianceLeaf,
        pub compliance_epk: decaf377::Element,
        pub compliance_ciphertext: Vec<Fq>,
        pub compliance_ciphertext_bytes: Vec<u8>,
        pub ephemeral_secret: Fr,
        pub asset_anchor: tct::StateCommitment,
        pub compliance_anchor: tct::StateCommitment,
        pub timestamp: u64,
    }

    /// Generate unified test data with compliance encryption.
    ///
    /// Creates a note, keys, and compliance data (regulated or unregulated).
    /// This data can be used by both groth16 circuit tests and plan-level tests.
    ///
    /// # Arguments
    /// * `rng` - Random number generator
    /// * `asset_id` - Asset ID for the note
    /// * `amount` - Amount for the note
    /// * `is_regulated` - Whether to use real ACK (regulated) or BLACK_HOLE_ACK (unregulated)
    pub fn generate_test_data(
        rng: &mut (impl rand::RngCore + rand_core::CryptoRng),
        asset_id: u64,
        amount: u64,
        is_regulated: bool,
    ) -> TestData {
        use penumbra_sdk_asset::Value;
        use penumbra_sdk_compliance::crypto::encrypt_compliance_details;
        use penumbra_sdk_keys::keys::AddressComplianceKey;
        use penumbra_sdk_num::Amount;

        let seed_phrase = SeedPhrase::generate(&mut *rng);
        let sk = SpendKey::from_seed_phrase_bip44(seed_phrase, &Bip44Path::new(0));
        let fvk = sk.full_viewing_key();
        let ivk = fvk.incoming();
        let (address, _dtk_d) = ivk.payment_address(0u32.into());

        let value = Value {
            amount: Amount::from(amount),
            asset_id: asset::Id(Fq::from(asset_id)),
        };

        let note = Note::from_parts(address.clone(), value, Rseed::generate(&mut *rng))
            .expect("can create note");

        let balance_blinding = Fr::rand(&mut *rng);

        // Generate compliance data based on regulation status
        let (ack, user_leaf, compliance_ciphertext_bytes, ephemeral_secret) = if is_regulated {
            // Regulated: use a random ACK
            let ack_sk = Fr::rand(&mut *rng);
            let ack_point = decaf377::Element::GENERATOR * ack_sk;
            let user_ack = AddressComplianceKey::new(ack_point);
            let user_leaf = penumbra_sdk_compliance::ComplianceLeaf {
                address: address.clone(),
                key: user_ack.clone(),
                asset_id: value.asset_id,
            };

            let timestamp = current_timestamp();
            let date = timestamp / 86400;
            let (ciphertext_obj, ephemeral_secret) = encrypt_compliance_details(
                &mut *rng,
                &user_ack,
                &address,
                date,
                note.asset_id(),
                note.amount(),
                address.clone(),
            )
            .expect("can encrypt");

            (
                user_ack,
                user_leaf,
                ciphertext_obj.to_bytes(),
                ephemeral_secret,
            )
        } else {
            // Unregulated: use BLACK_HOLE_ACK
            let black_hole_ack =
                AddressComplianceKey::new(*penumbra_sdk_compliance::BLACK_HOLE_ACK);
            let user_leaf = penumbra_sdk_compliance::ComplianceLeaf {
                address: address.clone(),
                key: black_hole_ack.clone(),
                asset_id: value.asset_id,
            };

            let timestamp = current_timestamp();
            let date = timestamp / 86400;
            let (ciphertext_obj, ephemeral_secret) = encrypt_compliance_details(
                &mut *rng,
                &black_hole_ack,
                &address,
                date,
                note.asset_id(),
                note.amount(),
                address.clone(),
            )
            .expect("can encrypt");

            (
                black_hole_ack,
                user_leaf,
                ciphertext_obj.to_bytes(),
                ephemeral_secret,
            )
        };

        // Use ComplianceCiphertext to deserialize and extract circuit inputs
        use penumbra_sdk_compliance::structs::ComplianceCiphertext;
        let ct = ComplianceCiphertext::from_bytes(&compliance_ciphertext_bytes)
            .expect("can deserialize ciphertext");
        let (compliance_epk, compliance_ciphertext) = ct.to_circuit_public_inputs();

        let asset_anchor = tct::StateCommitment(Fq::from(0u64));
        let compliance_anchor = tct::StateCommitment(Fq::from(0u64));
        let timestamp = current_timestamp();

        TestData {
            note,
            address,
            value,
            balance_blinding,
            fvk: fvk.clone(),
            sk,
            ack,
            user_leaf,
            compliance_epk,
            compliance_ciphertext,
            compliance_ciphertext_bytes,
            ephemeral_secret,
            asset_anchor,
            compliance_anchor,
            timestamp,
        }
    }

    /// Run Output circuit Groth16 roundtrip test.
    fn test_output_groth16(test_data: TestData, is_regulated: bool) {
        use crate::output::{OutputCircuit, OutputProof, OutputProofPrivate, OutputProofPublic};
        use decaf377::Fr;
        use penumbra_sdk_asset::Balance;

        // Setup
        let (pk, pvk, blinding_r, blinding_s) = setup_groth16_keys::<OutputCircuit>();

        let note_commitment = test_data.note.commit();
        let balance_commitment =
            (-Balance::from(test_data.value)).commit(test_data.balance_blinding);

        let dummy_leaf = penumbra_sdk_compliance::ComplianceLeaf {
            address: test_data.address.clone(),
            key: test_data.ack.clone(),
            asset_id: test_data.note.asset_id(),
        };
        let dummy_nonce = Fr::from(0u64);
        let receiver_leaf_hash =
            penumbra_sdk_compliance::blind_counterparty_leaf(dummy_leaf.commit(), dummy_nonce);
        let counterparty_leaf_hash =
            penumbra_sdk_compliance::blind_sender_leaf(dummy_leaf.commit(), dummy_nonce);

        let public = OutputProofPublic {
            balance_commitment,
            note_commitment,
            compliance_epk: test_data.compliance_epk,
            compliance_ciphertext: test_data.compliance_ciphertext,
            asset_anchor: test_data.asset_anchor,
            compliance_anchor: test_data.compliance_anchor,
            target_timestamp: test_data.timestamp,
            receiver_leaf_hash,
            counterparty_leaf_hash,
        };

        let private = OutputProofPrivate {
            note: test_data.note,
            balance_blinding: test_data.balance_blinding,
            asset_path: penumbra_sdk_compliance::MerklePath::default(),
            asset_position: 0,
            is_regulated,
            compliance_path: penumbra_sdk_compliance::MerklePath::default(),
            compliance_position: 0,
            user_leaf: test_data.user_leaf,
            compliance_ephemeral_secret: test_data.ephemeral_secret,
            counterparty_leaf: dummy_leaf.clone(),
            tx_blinding_nonce: dummy_nonce,
        };

        // Prove
        let proof = OutputProof::prove(blinding_r, blinding_s, &pk, public.clone(), private)
            .expect("can generate proof");

        // Verify
        proof.verify(&pvk, public).expect("proof should verify");
    }

    /// Run Spend circuit Groth16 roundtrip test.
    fn test_spend_groth16(test_data: TestData, is_regulated: bool) {
        use crate::spend::{SpendCircuit, SpendProof, SpendProofPrivate, SpendProofPublic};
        use penumbra_sdk_asset::Balance;
        use penumbra_sdk_sct::Nullifier;

        let mut rng = rand::thread_rng();

        // Setup
        let (pk, pvk, blinding_r, blinding_s) = setup_groth16_keys::<SpendCircuit>();

        // Create SCT for spend
        let mut sct = tct::Tree::new();
        let note_commitment = test_data.note.commit();
        sct.insert(tct::Witness::Keep, note_commitment).unwrap();
        let anchor = sct.root();
        let state_commitment_proof = sct.witness(note_commitment).unwrap();

        // Prepare public/private inputs
        let balance_commitment = Balance::from(test_data.value).commit(test_data.balance_blinding);
        let nullifier = Nullifier::derive(
            test_data.fvk.nullifier_key(),
            state_commitment_proof.position(),
            &note_commitment,
        );
        let randomizer = Fr::rand(&mut rng);
        let rk = test_data
            .fvk
            .spend_verification_key()
            .randomize(&randomizer);

        // Create dummy leaves and blinded hashes for testing
        let dummy_leaf = penumbra_sdk_compliance::ComplianceLeaf {
            address: test_data.address.clone(),
            key: test_data.ack.clone(),
            asset_id: test_data.note.asset_id(),
        };
        let dummy_nonce = Fr::from(0u64);
        let sender_leaf_hash =
            penumbra_sdk_compliance::blind_sender_leaf(dummy_leaf.commit(), dummy_nonce);
        let counterparty_leaf_hash =
            penumbra_sdk_compliance::blind_counterparty_leaf(dummy_leaf.commit(), dummy_nonce);

        let public = SpendProofPublic {
            anchor,
            balance_commitment,
            nullifier,
            rk,
            asset_anchor: test_data.asset_anchor,
            compliance_anchor: test_data.compliance_anchor,
            compliance_epk: test_data.compliance_epk,
            compliance_ciphertext: test_data.compliance_ciphertext,
            target_timestamp: test_data.timestamp,
            sender_leaf_hash,
            counterparty_leaf_hash,
        };

        let private = SpendProofPrivate {
            state_commitment_proof,
            note: test_data.note,
            v_blinding: test_data.balance_blinding,
            spend_auth_randomizer: randomizer,
            ak: *test_data.fvk.spend_verification_key(),
            nk: *test_data.fvk.nullifier_key(),
            asset_path: penumbra_sdk_compliance::MerklePath::default(),
            asset_position: 0,
            is_regulated,
            compliance_path: penumbra_sdk_compliance::MerklePath::default(),
            compliance_position: 0,
            user_leaf: test_data.user_leaf,
            compliance_ephemeral_secret: test_data.ephemeral_secret,
            counterparty_leaf: dummy_leaf.clone(),
            tx_blinding_nonce: dummy_nonce,
        };

        // Prove
        let proof = SpendProof::prove(blinding_r, blinding_s, &pk, public.clone(), private)
            .expect("can generate proof");

        // Verify
        proof.verify(&pvk, public).expect("proof should verify");
    }

    /// Unified Groth16 roundtrip test function.
    ///
    /// This function consolidates all Groth16 test logic for both Spend and Output circuits,
    /// and both regulated and unregulated assets.
    ///
    /// # Arguments
    /// * `circuit_type` - Whether to test SpendCircuit or OutputCircuit
    /// * `is_regulated` - Whether the asset is regulated (requires compliance checks)
    ///
    /// # Workflow
    /// 1. Generate test data with valid compliance encryption
    /// 2. Run circuit-specific Groth16 roundtrip (setup, prove, verify)
    pub fn full_groth16_roundtrip(circuit_type: CircuitType, is_regulated: bool) {
        let mut rng = rand::thread_rng();

        // Generate test data with compliance encryption
        // Use default test values: asset_id=1, amount=100
        let test_data = generate_test_data(&mut rng, 1, 100, is_regulated);

        // Run circuit-specific test
        match circuit_type {
            CircuitType::Output => test_output_groth16(test_data, is_regulated),
            CircuitType::Spend => test_spend_groth16(test_data, is_regulated),
        }
    }
}
