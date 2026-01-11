//! Compliance integration tests: user segregation, multi-wallet, black hole, role-based privacy.

use {
    anyhow::Result,
    ark_groth16::{Groth16, PreparedVerifyingKey, ProvingKey},
    ark_snark::SNARK,
    cnidarium::{StateDelta, TempStorage},
    common::TempStorageExt as _,
    decaf377::{Bls12_377, Fq, Fr},
    penumbra_sdk_asset::{asset, Value},
    penumbra_sdk_compliance::{
        registry::{ComplianceRegistryRead, ComplianceRegistryWrite},
        scanning::{ComplianceScanner, FullComplianceData, ScannerRole},
        structs::{ComplianceCiphertext, ComplianceLeaf, MerklePath},
        BLACK_HOLE_ACK,
    },
    penumbra_sdk_keys::{
        keys::{
            AddressComplianceKey, Bip44Path, DailyKeySet, MasterComplianceKey, SeedPhrase, SpendKey,
        },
        Address,
    },
    penumbra_sdk_num,
    penumbra_sdk_proof_params::DummyWitness,
    penumbra_sdk_shielded_pool::{
        output::OutputCircuit, spend::SpendCircuit, timestamp_to_day_index, Note, Rseed,
    },
    penumbra_sdk_tct as tct,
    rand_core::OsRng,
};

mod common;

/// Scan dual ciphertexts using a ComplianceScanner with specific role.
fn scan_with_role(
    sender_ciphertext_bytes: &[u8],
    receiver_ciphertext_bytes: &[u8],
    daily_keys: &DailyKeySet,
    role: ScannerRole,
) -> anyhow::Result<Option<FullComplianceData>> {
    let scanner = ComplianceScanner::new(daily_keys.clone(), role);

    let sender_ct = ComplianceCiphertext::from_bytes(sender_ciphertext_bytes)?;
    let receiver_ct = ComplianceCiphertext::from_bytes(receiver_ciphertext_bytes)?;

    scanner.decrypt(&sender_ct, &receiver_ct)
}

/// Create dual compliance ciphertexts (sender and receiver).
fn create_spend_dual_ciphertext(
    sender_ack: &AddressComplianceKey,
    sender_address: &Address,
    receiver_ack: &AddressComplianceKey,
    receiver_address: &Address,
    date: u64,
    asset_id: asset::Id,
    amount: penumbra_sdk_num::Amount,
    is_regulated: bool,
) -> Result<(Vec<u8>, Vec<u8>, Fr, Fr)> {
    use penumbra_sdk_compliance::crypto::encrypt_compliance_details_dual;

    let (ack_s, ack_r) = if is_regulated {
        (sender_ack.clone(), receiver_ack.clone())
    } else {
        (
            AddressComplianceKey::new(*BLACK_HOLE_ACK),
            AddressComplianceKey::new(*BLACK_HOLE_ACK),
        )
    };

    let (s_ct, s_eph, r_ct, r_eph) = encrypt_compliance_details_dual(
        &mut OsRng,
        &ack_s,
        sender_address,
        &ack_r,
        receiver_address,
        date,
        asset_id,
        amount,
    )?;

    Ok((s_ct.to_bytes(), r_ct.to_bytes(), s_eph, r_eph))
}

/// Get current Unix timestamp in seconds
fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time should be after Unix epoch")
        .as_secs()
}

/// Test harness for user-segregated compliance integration tests
struct ComplianceTestHarness {
    _storage: TempStorage,
    regulated_token_id: asset::Id,
    unregulated_token_id: asset::Id,

    // Alice's Master Key and Wallet 1
    alice_master_key: MasterComplianceKey,
    alice_ack: AddressComplianceKey,
    alice_addr: Address,
    _alice_spend_key: SpendKey,

    // Alice's Wallet 2 (for multi-wallet testing)
    alice_ack_2: AddressComplianceKey,
    alice_addr_2: Address,

    // Bob's Master Key and Wallet
    bob_master_key: MasterComplianceKey,
    bob_ack: AddressComplianceKey,
    bob_addr: Address,
    _bob_spend_key: SpendKey,

    // Circuit Keys (unused in scanner-based tests, but needed for harness setup)
    _spend_pk: ProvingKey<Bls12_377>,
    _spend_vk: PreparedVerifyingKey<Bls12_377>,
    _output_pk: ProvingKey<Bls12_377>,
    _output_vk: PreparedVerifyingKey<Bls12_377>,
}

impl ComplianceTestHarness {
    async fn new() -> Result<Self> {
        let storage = TempStorage::new_with_penumbra_prefixes().await?;

        // 1. Setup Circuits
        let (spend_pk, spend_vk, output_pk, output_vk) = Self::setup_circuits();

        // 2. Define Assets
        let regulated_token_id = asset::Id(Fq::from(10001u64));
        let unregulated_token_id = asset::Id(Fq::from(20002u64));

        // 3. Register Assets on-chain
        {
            let mut state = StateDelta::new(storage.latest_snapshot());
            state
                .update_asset_regulation(regulated_token_id, true)
                .await?;
            state
                .update_asset_regulation(unregulated_token_id, false)
                .await?;
            storage.commit(state).await?;
        }

        // 4. Create Alice's Master Key and Two Wallets
        let alice_master_key = MasterComplianceKey::new(Fr::rand(&mut OsRng));
        let (alice_addr, alice_spend_key) = Self::create_wallet("Alice-Wallet-1");
        let alice_ack = alice_master_key.derive_address_key(alice_addr.diversifier());

        // Alice's second wallet (different diversifier, same master key)
        let (alice_addr_2, _) = Self::create_wallet("Alice-Wallet-2");
        let alice_ack_2 = alice_master_key.derive_address_key(alice_addr_2.diversifier());

        // 5. Create Bob's Master Key and Wallet (Completely Separate)
        let bob_master_key = MasterComplianceKey::new(Fr::rand(&mut OsRng));
        let (bob_addr, bob_spend_key) = Self::create_wallet("Bob-Wallet-1");
        let bob_ack = bob_master_key.derive_address_key(bob_addr.diversifier());

        Ok(Self {
            _storage: storage,
            regulated_token_id,
            unregulated_token_id,
            alice_master_key,
            alice_ack,
            alice_addr,
            _alice_spend_key: alice_spend_key,
            alice_ack_2,
            alice_addr_2,
            bob_master_key,
            bob_ack,
            bob_addr,
            _bob_spend_key: bob_spend_key,
            _spend_pk: spend_pk,
            _spend_vk: spend_vk,
            _output_pk: output_pk,
            _output_vk: output_vk,
        })
    }

    /// Register a user in the compliance registry and return their position.
    #[allow(dead_code)]
    async fn register_user(
        &self,
        addr: Address,
        ack: AddressComplianceKey,
        asset_id: asset::Id,
    ) -> Result<u64> {
        let mut state = StateDelta::new(self._storage.latest_snapshot());
        let leaf = ComplianceLeaf {
            address: addr,
            key: ack,
            asset_id,
        };
        state.add_compliance_leaf(leaf).await?;
        let count = state.get_user_count().await?;
        self._storage.commit(state).await?;
        Ok(count - 1)
    }

    #[allow(dead_code)]
    fn setup_circuits() -> (
        ProvingKey<Bls12_377>,
        PreparedVerifyingKey<Bls12_377>,
        ProvingKey<Bls12_377>,
        PreparedVerifyingKey<Bls12_377>,
    ) {
        let mut rng = OsRng;

        let spend_circuit = SpendCircuit::with_dummy_witness();
        let (spend_pk, spend_vk) =
            Groth16::<Bls12_377>::circuit_specific_setup(spend_circuit, &mut rng)
                .expect("can perform spend circuit setup");

        let output_circuit = OutputCircuit::with_dummy_witness();
        let (output_pk, output_vk) =
            Groth16::<Bls12_377>::circuit_specific_setup(output_circuit, &mut rng)
                .expect("can perform output circuit setup");

        (
            spend_pk,
            PreparedVerifyingKey::from(spend_vk),
            output_pk,
            PreparedVerifyingKey::from(output_vk),
        )
    }

    #[allow(dead_code)]
    fn create_wallet(_name: &str) -> (Address, SpendKey) {
        use rand_core::RngCore;
        let mut seed_bytes = [0u8; 32];
        OsRng.fill_bytes(&mut seed_bytes);
        let seed = SeedPhrase::from_randomness(&seed_bytes);

        let spend_key = SpendKey::from_seed_phrase_bip44(seed, &Bip44Path::new(0));
        let fvk = spend_key.full_viewing_key();
        let address = fvk.payment_address(0u32.into()).0;

        (address, spend_key)
    }

    #[allow(dead_code)]
    fn create_note(&self, recipient: Address, value: Value) -> Result<Note> {
        use rand_core::RngCore;
        let mut rseed_bytes = [0u8; 32];
        OsRng.fill_bytes(&mut rseed_bytes);
        let rseed = Rseed(rseed_bytes);
        Note::from_parts(recipient, value, rseed).map_err(Into::into)
    }

    #[allow(dead_code)]
    async fn get_asset_position(&self, asset_id: asset::Id) -> Result<u64> {
        let snapshot = self._storage.latest_snapshot();
        let index = snapshot
            .get_asset_index(asset_id)
            .await?
            .expect("asset not found");
        Ok(index)
    }

    #[allow(dead_code)]
    async fn get_asset_path(&self, asset_id: asset::Id) -> Result<MerklePath> {
        let snapshot = self._storage.latest_snapshot();
        let index = snapshot
            .get_asset_index(asset_id)
            .await?
            .expect("asset not found");
        let path = snapshot.get_asset_auth_path(index).await?;
        Ok(Self::convert_merkle_path(path))
    }

    #[allow(dead_code)]
    async fn get_user_path(&self, position: u64) -> Result<MerklePath> {
        let snapshot = self._storage.latest_snapshot();
        let path = snapshot.get_user_auth_path(position).await?;
        Ok(Self::convert_merkle_path(path))
    }

    /// Helper to convert from tct auth path to compliance MerklePath
    #[allow(dead_code)]
    fn convert_merkle_path(path: Vec<[tct::StateCommitment; 3]>) -> MerklePath {
        MerklePath::from(penumbra_sdk_compliance::structs::MerklePath {
            layers: path
                .into_iter()
                .map(
                    |siblings| penumbra_sdk_compliance::structs::MerklePathLayer {
                        siblings: siblings.iter().map(|c| c.0.to_bytes().to_vec()).collect(),
                    },
                )
                .collect(),
        })
    }
}

#[tokio::test]
async fn test_user_segregation_alice_cannot_decrypt_bobs_transactions() -> Result<()> {
    let _guard = common::set_tracing_subscriber();
    let harness = ComplianceTestHarness::new().await?;

    let value = Value {
        amount: 100u64.into(),
        asset_id: harness.regulated_token_id,
    };
    let timestamp = current_timestamp();
    let date = timestamp_to_day_index(timestamp);

    // Create dual ciphertexts for Alice -> Bob transaction
    let (alice_sender_ct, bob_receiver_ct, _, _) = create_spend_dual_ciphertext(
        &harness.alice_ack,
        &harness.alice_addr,
        &harness.bob_ack,
        &harness.bob_addr,
        date,
        value.asset_id,
        value.amount,
        true, // regulated
    )?;

    // Alice (Sender) can decrypt her own send
    let alice_daily_key = harness.alice_master_key.derive_daily_keys(date);
    let alice_sender_scanner_result = scan_with_role(
        &alice_sender_ct,
        &bob_receiver_ct,
        &alice_daily_key,
        ScannerRole::Sender,
    )?;
    assert!(
        alice_sender_scanner_result.is_some(),
        "Alice SHOULD decrypt her own send (Sender scanner)"
    );

    // Alice (Receiver) CANNOT decrypt Sender ciphertext (role privacy)
    let alice_receiver_scanner_result = scan_with_role(
        &alice_sender_ct,
        &bob_receiver_ct,
        &alice_daily_key,
        ScannerRole::Receiver,
    )?;
    assert!(
        alice_receiver_scanner_result.is_none(),
        "Alice CANNOT decrypt with Receiver scanner (role privacy)"
    );

    // Bob (Receiver) can decrypt his own receive
    let bob_daily_key = harness.bob_master_key.derive_daily_keys(date);
    let bob_receiver_scanner_result = scan_with_role(
        &alice_sender_ct,
        &bob_receiver_ct,
        &bob_daily_key,
        ScannerRole::Receiver,
    )?;
    assert!(
        bob_receiver_scanner_result.is_some(),
        "Bob SHOULD decrypt his own receive (Receiver scanner)"
    );

    // Bob (Sender) CANNOT decrypt Receiver ciphertext (role privacy)
    let bob_sender_scanner_result = scan_with_role(
        &alice_sender_ct,
        &bob_receiver_ct,
        &bob_daily_key,
        ScannerRole::Sender,
    )?;
    assert!(
        bob_sender_scanner_result.is_none(),
        "Bob CANNOT decrypt with Sender scanner (role privacy)"
    );

    // Alice CANNOT decrypt Bob's ciphertext (user segregation)
    let alice_try_bob_receiver = scan_with_role(
        &alice_sender_ct,
        &bob_receiver_ct,
        &alice_daily_key,
        ScannerRole::Receiver, // Alice tries to decrypt Bob's receiver ciphertext
    )?;
    assert!(
        alice_try_bob_receiver.is_none(),
        "Alice CANNOT decrypt Bob's ciphertext (user segregation)"
    );

    // Bob CANNOT decrypt Alice's ciphertext (user segregation)
    let bob_try_alice_sender = scan_with_role(
        &alice_sender_ct,
        &bob_receiver_ct,
        &bob_daily_key,
        ScannerRole::Sender, // Bob tries to decrypt Alice's sender ciphertext
    )?;
    assert!(
        bob_try_alice_sender.is_none(),
        "Bob CANNOT decrypt Alice's ciphertext (user segregation)"
    );

    Ok(())
}

#[tokio::test]
async fn test_alice_single_master_key_decrypts_all_her_wallets() -> Result<()> {
    let _guard = common::set_tracing_subscriber();
    let harness = ComplianceTestHarness::new().await?;

    let value = Value {
        amount: 100u64.into(),
        asset_id: harness.regulated_token_id,
    };
    let timestamp = current_timestamp();
    let date = timestamp_to_day_index(timestamp);

    // Create transactions from Alice's two different wallets sending to Bob

    // Wallet 1 -> Bob
    let (alice_wallet1_sender_ct, bob_wallet1_receiver_ct, _, _) = create_spend_dual_ciphertext(
        &harness.alice_ack,
        &harness.alice_addr,
        &harness.bob_ack,
        &harness.bob_addr,
        date,
        value.asset_id,
        value.amount,
        true,
    )?;

    // Wallet 2 -> Bob
    let (alice_wallet2_sender_ct, bob_wallet2_receiver_ct, _, _) = create_spend_dual_ciphertext(
        &harness.alice_ack_2,
        &harness.alice_addr_2,
        &harness.bob_ack,
        &harness.bob_addr,
        date,
        value.asset_id,
        value.amount,
        true,
    )?;

    // Single Master Key decrypts both wallets
    let alice_daily_key = harness.alice_master_key.derive_daily_keys(date);

    // Alice's Sender scanner can decrypt Wallet 1's transaction
    let wallet1_result = scan_with_role(
        &alice_wallet1_sender_ct,
        &bob_wallet1_receiver_ct,
        &alice_daily_key,
        ScannerRole::Sender,
    )?;
    assert!(
        wallet1_result.is_some(),
        "Alice's Master Key SHOULD decrypt Wallet 1"
    );

    // Alice's Sender scanner can decrypt Wallet 2's transaction
    let wallet2_result = scan_with_role(
        &alice_wallet2_sender_ct,
        &bob_wallet2_receiver_ct,
        &alice_daily_key,
        ScannerRole::Sender,
    )?;
    assert!(
        wallet2_result.is_some(),
        "Alice's Master Key SHOULD decrypt Wallet 2 (All-Seeing property!)"
    );

    Ok(())
}

#[tokio::test]
async fn test_unregulated_asset_black_hole_undecryptable() -> Result<()> {
    let _guard = common::set_tracing_subscriber();
    let harness = ComplianceTestHarness::new().await?;

    let value = Value {
        amount: 100u64.into(),
        asset_id: harness.unregulated_token_id,
    };
    let timestamp = current_timestamp();
    let date = timestamp_to_day_index(timestamp);

    // Create dual ciphertexts for unregulated asset (Alice -> Bob)
    // Both ciphertexts will be encrypted to BLACK_HOLE_ACK
    let (black_hole_sender_ct, black_hole_receiver_ct, _, _) = create_spend_dual_ciphertext(
        &harness.alice_ack,
        &harness.alice_addr,
        &harness.bob_ack,
        &harness.bob_addr,
        date,
        value.asset_id,
        value.amount,
        false, // UNREGULATED - will use BLACK_HOLE_ACK
    )?;

    // Nobody can decrypt BLACK_HOLE
    let alice_daily_key = harness.alice_master_key.derive_daily_keys(date);
    let bob_daily_key = harness.bob_master_key.derive_daily_keys(date);

    // Alice tries to decrypt with Sender scanner -> FAILURE
    let alice_sender_result = scan_with_role(
        &black_hole_sender_ct,
        &black_hole_receiver_ct,
        &alice_daily_key,
        ScannerRole::Sender,
    )?;
    assert!(
        alice_sender_result.is_none(),
        "Alice CANNOT decrypt BLACK_HOLE sender ciphertext"
    );

    // Alice tries to decrypt with Receiver scanner -> FAILURE
    let alice_receiver_result = scan_with_role(
        &black_hole_sender_ct,
        &black_hole_receiver_ct,
        &alice_daily_key,
        ScannerRole::Receiver,
    )?;
    assert!(
        alice_receiver_result.is_none(),
        "Alice CANNOT decrypt BLACK_HOLE receiver ciphertext"
    );

    // Bob tries to decrypt with Sender scanner -> FAILURE
    let bob_sender_result = scan_with_role(
        &black_hole_sender_ct,
        &black_hole_receiver_ct,
        &bob_daily_key,
        ScannerRole::Sender,
    )?;
    assert!(
        bob_sender_result.is_none(),
        "Bob CANNOT decrypt BLACK_HOLE sender ciphertext"
    );

    // Bob tries to decrypt with Receiver scanner -> FAILURE
    let bob_receiver_result = scan_with_role(
        &black_hole_sender_ct,
        &black_hole_receiver_ct,
        &bob_daily_key,
        ScannerRole::Receiver,
    )?;
    assert!(
        bob_receiver_result.is_none(),
        "Bob CANNOT decrypt BLACK_HOLE receiver ciphertext"
    );

    Ok(())
}

#[tokio::test]
async fn test_key_type_specificity() -> Result<()> {
    let _guard = common::set_tracing_subscriber();
    let harness = ComplianceTestHarness::new().await?;

    let value = Value {
        amount: 100u64.into(),
        asset_id: harness.regulated_token_id,
    };
    let timestamp = current_timestamp();
    let date = timestamp_to_day_index(timestamp);

    // Create dual ciphertexts for Alice -> Bob transaction
    let (alice_sender_ct, bob_receiver_ct, _, _) = create_spend_dual_ciphertext(
        &harness.alice_ack,
        &harness.alice_addr,
        &harness.bob_ack,
        &harness.bob_addr,
        date,
        value.asset_id,
        value.amount,
        true, // regulated
    )?;

    // Wrong date fails
    let wrong_date = date + 1;
    let alice_wrong_date_key = harness.alice_master_key.derive_daily_keys(wrong_date);

    let wrong_date_result = scan_with_role(
        &alice_sender_ct,
        &bob_receiver_ct,
        &alice_wrong_date_key,
        ScannerRole::Sender,
    )?;
    assert!(
        wrong_date_result.is_none(),
        "Wrong date key SHOULD fail detection (detection key specificity)"
    );

    // Correct date → detection succeeds
    let alice_correct_date_key = harness.alice_master_key.derive_daily_keys(date);
    let correct_date_result = scan_with_role(
        &alice_sender_ct,
        &bob_receiver_ct,
        &alice_correct_date_key,
        ScannerRole::Sender,
    )?;
    assert!(
        correct_date_result.is_some(),
        "Correct date key SHOULD succeed detection"
    );

    // Wrong issuer fails
    let bob_daily_key = harness.bob_master_key.derive_daily_keys(date);
    let wrong_issuer_result = scan_with_role(
        &alice_sender_ct,
        &bob_receiver_ct,
        &bob_daily_key,
        ScannerRole::Sender, // Same role, but wrong issuer
    )?;
    assert!(
        wrong_issuer_result.is_none(),
        "Wrong issuer key SHOULD fail decryption (encryption key specificity)"
    );

    // Correct issuer → decryption succeeds
    let correct_issuer_result = scan_with_role(
        &alice_sender_ct,
        &bob_receiver_ct,
        &alice_correct_date_key,
        ScannerRole::Sender,
    )?;
    assert!(
        correct_issuer_result.is_some(),
        "Correct issuer key SHOULD succeed decryption"
    );

    // Wrong role fails
    let alice_wrong_role_result = scan_with_role(
        &alice_sender_ct,
        &bob_receiver_ct,
        &alice_correct_date_key,
        ScannerRole::Receiver, // WRONG ROLE - Alice is sender, not receiver
    )?;
    assert!(
        alice_wrong_role_result.is_none(),
        "Wrong role scanner SHOULD fail (counterparty key specificity)"
    );

    // Bob tries to decrypt Alice's sender ciphertext with Sender scanner
    // This is the wrong role for Bob (he's the receiver)
    let bob_wrong_role_result = scan_with_role(
        &alice_sender_ct,
        &bob_receiver_ct,
        &bob_daily_key,
        ScannerRole::Sender, // WRONG ROLE - Bob is receiver, not sender
    )?;
    assert!(
        bob_wrong_role_result.is_none(),
        "Wrong role scanner SHOULD fail (counterparty key specificity)"
    );

    // Correct roles → both can decrypt their respective ciphertexts
    let alice_correct_role_result = scan_with_role(
        &alice_sender_ct,
        &bob_receiver_ct,
        &alice_correct_date_key,
        ScannerRole::Sender, // CORRECT ROLE - Alice is sender
    )?;
    assert!(
        alice_correct_role_result.is_some(),
        "Correct sender role SHOULD succeed"
    );

    let bob_correct_role_result = scan_with_role(
        &alice_sender_ct,
        &bob_receiver_ct,
        &bob_daily_key,
        ScannerRole::Receiver, // CORRECT ROLE - Bob is receiver
    )?;
    assert!(
        bob_correct_role_result.is_some(),
        "Correct receiver role SHOULD succeed"
    );

    // All wrong keys fails
    let wrong_date_wrong_issuer_key = harness.bob_master_key.derive_daily_keys(wrong_date);
    let all_wrong_result = scan_with_role(
        &alice_sender_ct,
        &bob_receiver_ct,
        &wrong_date_wrong_issuer_key,
        ScannerRole::Receiver, // Also wrong role
    )?;
    assert!(
        all_wrong_result.is_none(),
        "All wrong keys SHOULD definitively fail"
    );

    Ok(())
}

#[tokio::test]
#[ignore] // Slow - run with --ignored
async fn test_full_compliance_proof_roundtrip() -> Result<()> {
    use penumbra_sdk_shielded_pool::{OutputPlan, SpendPlan};

    let _guard = common::set_tracing_subscriber();
    let harness = ComplianceTestHarness::new().await?;

    let value = Value {
        amount: 100u64.into(),
        asset_id: harness.regulated_token_id,
    };
    let timestamp = current_timestamp();
    let date = timestamp_to_day_index(timestamp);

    let alice_position = harness
        .register_user(
            harness.alice_addr.clone(),
            harness.alice_ack.clone(),
            harness.regulated_token_id,
        )
        .await?;

    let bob_position = harness
        .register_user(
            harness.bob_addr.clone(),
            harness.bob_ack.clone(),
            harness.regulated_token_id,
        )
        .await?;

    let alice_merkle_path = harness.get_user_path(alice_position).await?;
    let bob_merkle_path = harness.get_user_path(bob_position).await?;
    let asset_position = harness
        .get_asset_position(harness.regulated_token_id)
        .await?;
    let asset_merkle_path = harness.get_asset_path(harness.regulated_token_id).await?;

    // Get compliance and asset anchors from state
    let snapshot = harness._storage.latest_snapshot();
    let compliance_anchor = snapshot.get_user_tree_root().await?;
    let asset_anchor = snapshot.get_asset_tree_root().await?;
    // STEP 2: Create compliance leaves

    let alice_leaf = ComplianceLeaf {
        address: harness.alice_addr.clone(),
        key: harness.alice_ack.clone(),
        asset_id: harness.regulated_token_id,
    };

    let bob_leaf = ComplianceLeaf {
        address: harness.bob_addr.clone(),
        key: harness.bob_ack.clone(),
        asset_id: harness.regulated_token_id,
    };
    // STEP 3: Create Note and SCT for Spend

    let spend_note = Note::from_parts(
        harness.alice_addr.clone(),
        value,
        Rseed::generate(&mut OsRng),
    )?;

    let mut sct = tct::Tree::new();
    sct.insert(tct::Witness::Keep, spend_note.commit())?;
    let anchor = sct.root();
    let state_commitment_proof = sct
        .witness(spend_note.commit())
        .expect("note was just inserted");

    let fvk = harness._alice_spend_key.full_viewing_key();
    // STEP 4: Create SpendPlan and set all compliance fields

    let mut spend_plan = SpendPlan::new(
        &mut OsRng,
        spend_note.clone(),
        state_commitment_proof.position(),
    );
    spend_plan.target_timestamp = timestamp;

    // Set compliance details via the plan method
    spend_plan.set_compliance_details(
        &mut OsRng,
        &harness.alice_ack,
        &harness.alice_addr,
        true, // is_regulated
        &harness.bob_addr,
        bob_leaf.clone(),
    )?;

    // Set the compliance fields that would normally come from enrich_with_compliance / gRPC
    spend_plan.compliance_path = alice_merkle_path.clone();
    spend_plan.compliance_position = alice_position;
    spend_plan.compliance_anchor = compliance_anchor;
    spend_plan.asset_path = asset_merkle_path.clone();
    spend_plan.asset_position = asset_position;
    spend_plan.asset_anchor = asset_anchor;

    // Get the tx_blinding_nonce from spend_plan to share with output_plan
    let tx_blinding_nonce = spend_plan.tx_blinding_nonce;
    // STEP 5: Generate Spend Proof via SpendPlan

    let spend_proof = spend_plan.spend_proof(
        fvk,
        state_commitment_proof.clone(),
        anchor,
        &harness._spend_pk,
        None,
    )?;

    // Build public inputs for verification
    use penumbra_sdk_compliance::structs::ComplianceCiphertext;
    use penumbra_sdk_shielded_pool::spend::SpendProofPublic;

    let spend_ct = ComplianceCiphertext::from_bytes(&spend_plan.compliance_ciphertext)?;
    let (spend_epk, spend_ct_circuit) = spend_ct.to_circuit_public_inputs();

    let spend_sender_leaf_hash = spend_plan.compliance_leaf.clone().unwrap().commit();
    let spend_counterparty_leaf_hash = spend_plan.counterparty_leaf.clone().unwrap().commit();
    let spend_sender_blinded =
        penumbra_sdk_compliance::blind_sender_leaf(spend_sender_leaf_hash, tx_blinding_nonce);
    let spend_counterparty_blinded = penumbra_sdk_compliance::blind_counterparty_leaf(
        spend_counterparty_leaf_hash,
        tx_blinding_nonce,
    );

    let spend_public = SpendProofPublic {
        anchor,
        balance_commitment: spend_plan.balance().commit(spend_plan.value_blinding),
        nullifier: spend_plan.nullifier(fvk),
        rk: spend_plan.rk(fvk),
        asset_anchor,
        compliance_anchor,
        compliance_epk: spend_epk,
        compliance_ciphertext: spend_ct_circuit,
        target_timestamp: timestamp,
        sender_leaf_hash: spend_sender_blinded,
        counterparty_leaf_hash: spend_counterparty_blinded,
    };
    // STEP 6: Verify Spend Proof

    spend_proof.verify(&harness._spend_vk, spend_public)?;
    // STEP 7: Create OutputPlan and set all compliance fields

    let mut output_plan = OutputPlan::new(&mut OsRng, value, harness.bob_addr.clone());
    output_plan.target_timestamp = timestamp;

    // Set compliance details via the plan method
    output_plan.set_compliance_details(
        &mut OsRng,
        &harness.bob_ack,
        true, // is_regulated
        &harness.alice_addr,
        alice_leaf.clone(),
        tx_blinding_nonce, // Share the same blinding nonce from spend
    )?;

    // Set the compliance fields that would normally come from enrich_with_compliance / gRPC
    output_plan.compliance_path = bob_merkle_path.clone();
    output_plan.compliance_position = bob_position;
    output_plan.compliance_anchor = compliance_anchor;
    output_plan.asset_path = asset_merkle_path.clone();
    output_plan.asset_position = asset_position;
    output_plan.asset_anchor = asset_anchor;
    // STEP 8: Generate Output Proof via OutputPlan

    let output_proof = output_plan.output_proof(&harness._output_pk, None)?;

    // Build public inputs for verification
    use penumbra_sdk_shielded_pool::output::OutputProofPublic;

    let output_ct = ComplianceCiphertext::from_bytes(&output_plan.compliance_ciphertext)?;
    let (output_epk, output_ct_circuit) = output_ct.to_circuit_public_inputs();

    let output_receiver_leaf_hash = output_plan.compliance_leaf.clone().unwrap().commit();
    let output_counterparty_leaf_hash = output_plan.counterparty_leaf.clone().unwrap().commit();
    let output_receiver_blinded = penumbra_sdk_compliance::blind_counterparty_leaf(
        output_receiver_leaf_hash,
        tx_blinding_nonce,
    );
    let output_sender_blinded = penumbra_sdk_compliance::blind_sender_leaf(
        output_counterparty_leaf_hash,
        tx_blinding_nonce,
    );

    let output_public = OutputProofPublic {
        balance_commitment: output_plan.balance().commit(output_plan.value_blinding),
        note_commitment: output_plan.output_note().commit(),
        compliance_epk: output_epk,
        compliance_ciphertext: output_ct_circuit,
        asset_anchor,
        compliance_anchor,
        target_timestamp: timestamp,
        receiver_leaf_hash: output_receiver_blinded,
        counterparty_leaf_hash: output_sender_blinded,
    };
    // STEP 9: Verify Output Proof

    output_proof.verify(&harness._output_vk, output_public)?;
    // STEP 10: Verify Leaf Hash Binding (Transaction Validator Check)

    // The validator checks: spend.counterparty == output.receiver
    // Both are Bob's leaf, blinded the same way (as counterparty)
    assert_eq!(
        spend_counterparty_blinded, output_receiver_blinded,
        "Spend counterparty must match output receiver (blinded)"
    );

    // And: output.sender == spend.sender
    // Both are Alice's leaf, blinded the same way (as sender)
    assert_eq!(
        output_sender_blinded, spend_sender_blinded,
        "Output sender must match spend sender (blinded)"
    );
    // STEP 11: Scanner Verification

    let alice_daily_key = harness.alice_master_key.derive_daily_keys(date);
    let bob_daily_key = harness.bob_master_key.derive_daily_keys(date);

    // Alice scans sender ciphertext (from spend_plan)
    let alice_scan_result = scan_with_role(
        &spend_plan.compliance_ciphertext,
        &output_plan.compliance_ciphertext,
        &alice_daily_key,
        ScannerRole::Sender,
    )?;
    assert!(
        alice_scan_result.is_some(),
        "Alice SHOULD decrypt sender ciphertext"
    );

    // Bob scans receiver ciphertext (from output_plan)
    let bob_scan_result = scan_with_role(
        &spend_plan.compliance_ciphertext,
        &output_plan.compliance_ciphertext,
        &bob_daily_key,
        ScannerRole::Receiver,
    )?;
    assert!(
        bob_scan_result.is_some(),
        "Bob SHOULD decrypt receiver ciphertext"
    );

    // Verify both parties decrypted the same transaction details
    let alice_data = alice_scan_result.expect("Alice decryption succeeded");
    let bob_data = bob_scan_result.expect("Bob decryption succeeded");

    assert_eq!(
        alice_data.asset_id, bob_data.asset_id,
        "Asset ID should match"
    );
    assert_eq!(
        alice_data.core.amount, bob_data.core.amount,
        "Amount should match"
    );

    // Verify counterparty information: Alice's counterparty should be Bob
    // (comparing diversified generators and transmission keys)
    assert_eq!(
        alice_data.extension.counterparty_diversified_generator,
        bob_data.core.self_diversified_generator,
        "Alice's counterparty should be Bob (diversified generator)"
    );
    assert_eq!(
        alice_data.extension.counterparty_transmission_key, bob_data.core.self_transmission_key,
        "Alice's counterparty should be Bob (transmission key)"
    );
    Ok(())
}
