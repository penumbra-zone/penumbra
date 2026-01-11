pub mod structs;
pub use structs::{
    ComplianceCiphertext,
    ComplianceLeaf,
    CompliancePayload,
    MerklePath,
    MerklePathLayer,
    MsgRegisterAsset,
    MsgRegisterUser,
    // Wire format constants
    AMOUNT_BYTES,
    ASSET_ID_BYTES,
    CIPHERTEXT_PAYLOAD_BYTES,
    DETECTION_TAG_BYTES,
    ENCRYPTED_CORE_BYTES,
    ENCRYPTED_EXTENSION_BYTES,
    EPK_BYTES,
    GENERATOR_BYTES,
    KEY_BYTES,
    NUM_CIPHERTEXT_FQS,
    TOTAL_PLAINTEXT_BYTES,
    TOTAL_WIRE_BYTES,
};

pub mod tree;
pub use tree::{QuadTree, DEFAULT_DEPTH, ZERO_HASHES};

pub mod state_key;

// Registry requires cnidarium for state access
#[cfg(feature = "component")]
pub mod registry;
#[cfg(feature = "component")]
pub use registry::{ComplianceRegistryRead, ComplianceRegistryWrite};

#[cfg(feature = "component")]
pub mod component;
#[cfg(feature = "component")]
pub use component::{Compliance, RpcServer};

pub mod r1cs;
pub use r1cs::{verify_compliance_integrity, verify_quad_path, ComplianceWitness};

pub mod crypto;
pub use crypto::{
    decrypt_compliance_details, encrypt_compliance_details, encrypt_compliance_details_dual,
    DecryptedComplianceData, BLACK_HOLE_ACK,
};

pub mod scanning;
pub use scanning::{
    decrypt_core, decrypt_extension, decrypt_full, scan_for_asset, ComplianceScanner, CoreData,
    ExtensionData, FullComplianceData, ScannerRole,
};

// Scanner requires tokio and rusqlite for async storage
#[cfg(feature = "component")]
pub mod scanner;
#[cfg(feature = "component")]
pub use scanner::{
    decrypt_with_daily_keys, decrypt_with_mck, scan_transaction, scan_transaction_for_compliance,
    scan_transaction_for_compliance_with_daily_keys, scan_transactions,
    scan_transactions_for_compliance, ComplianceStorage, ComplianceWorker, DetectedCiphertext,
    DetectedTransfer, PartialAddress,
};

pub mod leaf_binding;
pub use leaf_binding::{
    blind_counterparty_leaf, blind_sender_leaf, DOMAIN_SEP_COUNTERPARTY, DOMAIN_SEP_SENDER,
};

/// Test helpers for compliance tests. Re-exported for use in other crates' tests.
#[cfg(any(test, feature = "test-helpers"))]
pub mod test_helpers {
    use decaf377::Fr;
    use penumbra_sdk_asset::asset;
    use penumbra_sdk_keys::keys::{AddressComplianceKey, Diversifier, MasterComplianceKey};
    use penumbra_sdk_keys::Address;
    use penumbra_sdk_num::Amount;
    use rand_core::{OsRng, RngCore};

    use crate::crypto::encrypt_compliance_details_dual;
    use crate::structs::ComplianceCiphertext;

    /// Create a random MasterComplianceKey.
    pub fn make_mck() -> MasterComplianceKey {
        MasterComplianceKey::new(Fr::rand(&mut OsRng))
    }

    /// Create an address with a specific diversifier byte pattern.
    pub fn make_address(div_byte: u8) -> Address {
        let mut rng = OsRng;
        let diversifier = Diversifier([div_byte; 16]);
        let scalar = Fr::rand(&mut rng);
        let point = decaf377::Element::GENERATOR * scalar;
        let pk_d = decaf377_ka::Public(point.vartime_compress().0);
        let mut ck_bytes = [0u8; 32];
        rng.fill_bytes(&mut ck_bytes);
        let ck = decaf377_fmd::ClueKey(ck_bytes);
        Address::from_components(diversifier, pk_d, ck).unwrap()
    }

    /// Create an ACK and matching address from an MCK with a specific diversifier.
    pub fn make_wallet(mck: &MasterComplianceKey, div_byte: u8) -> (AddressComplianceKey, Address) {
        let mut rng = OsRng;
        let diversifier = Diversifier([div_byte; 16]);
        let ack = mck.derive_address_key(&diversifier);
        let scalar = Fr::rand(&mut rng);
        let point = decaf377::Element::GENERATOR * scalar;
        let pk_d = decaf377_ka::Public(point.vartime_compress().0);
        let mut ck_bytes = [0u8; 32];
        rng.fill_bytes(&mut ck_bytes);
        let ck = decaf377_fmd::ClueKey(ck_bytes);
        let addr = Address::from_components(diversifier, pk_d, ck).unwrap();
        (ack, addr)
    }

    /// Encrypt compliance details for a sender/receiver pair, returning both ciphertexts.
    pub fn encrypt_dual(
        mck: &MasterComplianceKey,
        sender_div: u8,
        receiver_div: u8,
        date: u64,
        asset_id: u64,
        amount: u128,
    ) -> (ComplianceCiphertext, ComplianceCiphertext) {
        let mut rng = OsRng;
        let (ack_s, addr_s) = make_wallet(mck, sender_div);
        let (ack_r, addr_r) = make_wallet(mck, receiver_div);
        let (s_ct, _, r_ct, _) = encrypt_compliance_details_dual(
            &mut rng,
            &ack_s,
            &addr_s,
            &ack_r,
            &addr_r,
            date,
            asset::Id(decaf377::Fq::from(asset_id)),
            Amount::from(amount),
        )
        .unwrap();
        (s_ct, r_ct)
    }
}

// Integration tests require cnidarium, tokio, and scanner
#[cfg(all(test, feature = "component"))]
mod tests {
    use super::*;
    use cnidarium::{StateDelta, StateWrite, TempStorage};
    use decaf377::Fq;
    use penumbra_sdk_asset::asset;
    use penumbra_sdk_keys::{keys::AddressComplianceKey, Address};
    use penumbra_sdk_proto::StateWriteProto;
    use penumbra_sdk_tct::StateCommitment;

    #[tokio::test]
    async fn test_compliance_path_generation() {
        // 1. Initialize storage
        let storage = TempStorage::new().await.unwrap();
        let snapshot = storage.latest_snapshot();
        let mut state = StateDelta::new(snapshot);

        // Manually initialize trees (since we can't clone StateDelta for init_chain)
        let user_tree = QuadTree::new();
        let tree_bytes = bincode::serialize(&user_tree).unwrap();
        state.put_raw(state_key::user_tree().to_string(), tree_bytes);
        state.put_proto(state_key::user_count().to_string(), 0u64);

        // 2. Create and add a single ComplianceLeaf (User 1)
        let leaf = ComplianceLeaf {
            address: Address::dummy(&mut rand::thread_rng()),
            key: AddressComplianceKey::new(decaf377::Element::GENERATOR),
            asset_id: asset::Id(Fq::from(100u64)),
        };

        // Calculate the leaf commitment
        let user1_commit = leaf.commit();

        // Add the leaf to the tree
        state.add_compliance_leaf(leaf.clone()).await.unwrap();

        // 3. Verification

        // Retrieve the UserTree
        let tree = state.get_user_tree().await.unwrap();

        // Get the authentication path for User 1 (position 0)
        let path = tree.auth_path(0);

        // Assert: path is not empty
        assert!(!path.is_empty(), "Authentication path should not be empty");
        assert_eq!(
            path.len(),
            DEFAULT_DEPTH as usize,
            "Path length should match tree depth"
        );

        // Assert: First layer siblings should be zero hashes
        // Since we only added User 1 at position 0, siblings at positions 1, 2, 3
        // in the first layer should all be zero hashes for level 0
        let first_layer_siblings = path[0];
        let zero_hash_level_0 = ZERO_HASHES[0];

        // All three siblings in the first layer should be zero hashes
        // because positions 1, 2, 3 are empty
        assert_eq!(
            first_layer_siblings[0].0, zero_hash_level_0.0,
            "Sibling 1 (pos 1) should be zero hash"
        );
        assert_eq!(
            first_layer_siblings[1].0, zero_hash_level_0.0,
            "Sibling 2 (pos 2) should be zero hash"
        );
        assert_eq!(
            first_layer_siblings[2].0, zero_hash_level_0.0,
            "Sibling 3 (pos 3) should be zero hash"
        );

        // 4. Manual Check: Verify path computation from leaf to root
        let mut current_hash = user1_commit;
        let mut current_position = 0u64;

        for (_level, siblings) in path.iter().enumerate() {
            // Determine which child (0-3) we are at this level
            let child_index = (current_position % 4) as usize;

            // Reconstruct the 4 children for this parent
            let children = match child_index {
                0 => [current_hash, siblings[0], siblings[1], siblings[2]],
                1 => [siblings[0], current_hash, siblings[1], siblings[2]],
                2 => [siblings[0], siblings[1], current_hash, siblings[2]],
                3 => [siblings[0], siblings[1], siblings[2], current_hash],
                _ => unreachable!(),
            };

            // Hash the 4 children to get the parent
            let parent_hash = poseidon377::hash_4(
                &Fq::from(0u64),
                (children[0].0, children[1].0, children[2].0, children[3].0),
            );
            current_hash = StateCommitment(parent_hash);

            // Move to parent position
            current_position /= 4;
        }

        // After traversing the entire path, we should arrive at the root
        let tree_root = tree.root();
        assert_eq!(
            current_hash.0, tree_root.0,
            "Computed root from path should match tree root"
        );

        // 5. Additional verification: Use the tree's built-in verification
        let verified = QuadTree::verify_auth_path(0, user1_commit, &path, tree_root, DEFAULT_DEPTH);
        assert!(verified, "Built-in path verification should succeed");
    }

    #[tokio::test]
    async fn test_multiple_users_path() {
        let storage = TempStorage::new().await.unwrap();
        let snapshot = storage.latest_snapshot();
        let mut state = StateDelta::new(snapshot);

        // Manually initialize trees
        let user_tree = QuadTree::new();
        let tree_bytes = bincode::serialize(&user_tree).unwrap();
        state.put_raw(state_key::user_tree().to_string(), tree_bytes);
        state.put_proto(state_key::user_count().to_string(), 0u64);

        let mut rng = rand::thread_rng();
        let mut commitments = Vec::new();

        // Add 4 users (filling the first group of 4 siblings)
        for i in 0..4u64 {
            let leaf = ComplianceLeaf {
                address: Address::dummy(&mut rng),
                key: AddressComplianceKey::new(
                    decaf377::Element::GENERATOR * decaf377::Fr::from(i + 1),
                ),
                asset_id: asset::Id(Fq::from(i)),
            };

            commitments.push(leaf.commit());
            state.add_compliance_leaf(leaf).await.unwrap();
        }

        let tree = state.get_user_tree().await.unwrap();

        // Now verify the path for User 0
        let path = tree.auth_path(0);

        // The first layer siblings should NOT all be zero hashes now
        // because we have users at positions 1, 2, 3
        let first_layer_siblings = path[0];
        let _zero_hash_level_0 = ZERO_HASHES[0];

        // Verify that siblings match the commitments we inserted
        assert_eq!(
            first_layer_siblings[0].0, commitments[1].0,
            "Sibling 0 should be User 1's commitment"
        );
        assert_eq!(
            first_layer_siblings[1].0, commitments[2].0,
            "Sibling 1 should be User 2's commitment"
        );
        assert_eq!(
            first_layer_siblings[2].0, commitments[3].0,
            "Sibling 2 should be User 3's commitment"
        );

        // Verify the path is valid
        let tree_root = tree.root();
        let verified =
            QuadTree::verify_auth_path(0, commitments[0], &path, tree_root, DEFAULT_DEPTH);
        assert!(
            verified,
            "Path verification should succeed with multiple users"
        );
    }

    #[tokio::test]
    async fn test_different_positions() {
        let storage = TempStorage::new().await.unwrap();
        let snapshot = storage.latest_snapshot();
        let mut state = StateDelta::new(snapshot);

        // Manually initialize trees
        let user_tree = QuadTree::new();
        let tree_bytes = bincode::serialize(&user_tree).unwrap();
        state.put_raw(state_key::user_tree().to_string(), tree_bytes);
        state.put_proto(state_key::user_count().to_string(), 0u64);

        let mut rng = rand::thread_rng();

        // Add users at positions 0, 5, 10
        let positions = vec![0, 5, 10];
        let mut leaves = Vec::new();

        for &pos in &positions {
            // Add empty users to reach the position
            while state.get_user_count().await.unwrap() < pos {
                let dummy_leaf = ComplianceLeaf {
                    address: Address::dummy(&mut rng),
                    key: AddressComplianceKey::new(decaf377::Element::GENERATOR),
                    asset_id: asset::Id(Fq::from(0u64)),
                };
                state.add_compliance_leaf(dummy_leaf).await.unwrap();
            }

            // Add the actual user
            let leaf = ComplianceLeaf {
                address: Address::dummy(&mut rng),
                key: AddressComplianceKey::new(
                    decaf377::Element::GENERATOR * decaf377::Fr::from(pos + 1),
                ),
                asset_id: asset::Id(Fq::from(pos)),
            };
            state.add_compliance_leaf(leaf.clone()).await.unwrap();
            leaves.push((pos, leaf.commit()));
        }

        let tree = state.get_user_tree().await.unwrap();
        let tree_root = tree.root();

        // Verify paths for each position
        for (pos, commitment) in leaves {
            let path = tree.auth_path(pos);
            let verified =
                QuadTree::verify_auth_path(pos, commitment, &path, tree_root, DEFAULT_DEPTH);
            assert!(
                verified,
                "Path verification should succeed for position {}",
                pos
            );
        }
    }

    #[tokio::test]
    async fn test_end_to_end_three_phases() {
        use penumbra_sdk_keys::keys::MasterComplianceKey;
        use penumbra_sdk_num::Amount;

        // ========== SETUP ==========
        let storage = TempStorage::new().await.unwrap();
        let snapshot = storage.latest_snapshot();
        let mut state = StateDelta::new(snapshot);

        // Initialize trees
        let user_tree = QuadTree::new();
        let asset_tree = QuadTree::new();
        let tree_bytes = bincode::serialize(&user_tree).unwrap();
        state.put_raw(state_key::user_tree().to_string(), tree_bytes);
        state.put_proto(state_key::user_count().to_string(), 0u64);

        let tree_bytes = bincode::serialize(&asset_tree).unwrap();
        state.put_raw(state_key::asset_tree().to_string(), tree_bytes);
        state.put_proto(state_key::asset_count().to_string(), 0u64);

        let mut rng = rand::thread_rng();

        // ========== PHASE 1: Asset Registry Integration ==========

        // Create a regulated asset (e.g., USDC)
        let usdc_asset_id = asset::Id(Fq::from(1000u64));

        // Register asset as regulated
        state
            .update_asset_regulation(usdc_asset_id, true)
            .await
            .unwrap();

        // Verify regulation status
        let regulation_status = state.get_asset_status(usdc_asset_id).await.unwrap();
        assert_eq!(
            regulation_status,
            Some(true),
            "USDC should be registered as regulated"
        );

        // ========== PHASE 2: Wallet ACK Integration ==========

        // Create sender and receiver addresses
        let sender_address = Address::dummy(&mut rng);
        let recipient_address = Address::dummy(&mut rng);

        // Use demo MCK to derive compliance leaves (Phase 2)
        let demo_mck = MasterComplianceKey::demo();
        let sender_leaf = ComplianceLeaf::new(&demo_mck, sender_address.clone(), usdc_asset_id);
        let recipient_leaf =
            ComplianceLeaf::new(&demo_mck, recipient_address.clone(), usdc_asset_id);

        // Register sender and recipient in registry
        state
            .add_compliance_leaf(sender_leaf.clone())
            .await
            .unwrap();
        state
            .add_compliance_leaf(recipient_leaf.clone())
            .await
            .unwrap();

        // Verify sender position (should be 0)
        let sender_position = state
            .get_user_leaf_position(&sender_address, usdc_asset_id)
            .await
            .unwrap()
            .expect("Sender should have position");
        assert_eq!(sender_position, 0, "Sender should be at position 0");

        // Verify recipient position (should be 1)
        let recipient_position = state
            .get_user_leaf_position(&recipient_address, usdc_asset_id)
            .await
            .unwrap()
            .expect("Recipient should have position");
        assert_eq!(recipient_position, 1, "Recipient should be at position 1");

        // ========== PHASE 3A: Merkle Path Generation ==========

        // Get merkle paths for both users
        let sender_auth_path = state.get_user_auth_path(sender_position).await.unwrap();
        let recipient_auth_path = state.get_user_auth_path(recipient_position).await.unwrap();

        // Verify auth paths have correct depth
        assert_eq!(
            sender_auth_path.len(),
            DEFAULT_DEPTH as usize,
            "Sender auth path should have correct depth"
        );
        assert_eq!(
            recipient_auth_path.len(),
            DEFAULT_DEPTH as usize,
            "Recipient auth path should have correct depth"
        );

        // ========== PHASE 3B: MerklePath Conversion ==========

        // Convert auth paths to MerklePath format (for ZK circuits)
        let sender_merkle_path = MerklePath::from_auth_path(sender_auth_path.clone());
        let recipient_merkle_path = MerklePath::from_auth_path(recipient_auth_path.clone());

        // Verify conversion worked
        assert_eq!(
            sender_merkle_path.layers.len(),
            DEFAULT_DEPTH as usize,
            "Sender MerklePath should have correct number of layers"
        );
        assert_eq!(
            recipient_merkle_path.layers.len(),
            DEFAULT_DEPTH as usize,
            "Recipient MerklePath should have correct number of layers"
        );

        // Verify each layer has 3 siblings (arity 4 tree)
        for layer in &sender_merkle_path.layers {
            assert_eq!(
                layer.siblings.len(),
                3,
                "Each layer should have 3 siblings in arity-4 tree"
            );
        }

        // ========== PHASE 3C: Transaction Encryption Simulation ==========

        // Simulate transaction parameters
        let amount = Amount::from(100u64);
        let date = 20250101u64; // Example date for key derivation

        // Encrypt compliance details for both sender and receiver
        let (sender_ciphertext, sender_ephemeral, receiver_ciphertext, receiver_ephemeral) =
            encrypt_compliance_details_dual(
                &mut rng,
                &sender_leaf.key,
                &sender_address,
                &recipient_leaf.key,
                &recipient_address,
                date,
                usdc_asset_id,
                amount,
            )
            .unwrap();

        // Verify ciphertexts were generated
        assert_eq!(
            sender_ciphertext.to_bytes().len(),
            TOTAL_WIRE_BYTES,
            "Sender ciphertext should have correct wire format size"
        );
        assert_eq!(
            receiver_ciphertext.to_bytes().len(),
            TOTAL_WIRE_BYTES,
            "Receiver ciphertext should have correct wire format size"
        );

        // ========== VERIFICATION: Full Workflow ==========

        // 1. Verify asset is regulated
        let final_status = state.get_asset_status(usdc_asset_id).await.unwrap();
        assert_eq!(final_status, Some(true), "Asset should remain regulated");

        // 2. Verify both users are registered (via position lookup)
        assert!(state
            .get_user_leaf_position(&sender_address, usdc_asset_id)
            .await
            .unwrap()
            .is_some());
        assert!(state
            .get_user_leaf_position(&recipient_address, usdc_asset_id)
            .await
            .unwrap()
            .is_some());

        // 3. Verify merkle paths are valid
        let tree = state.get_user_tree().await.unwrap();
        let tree_root = tree.root();

        let sender_commit = sender_leaf.commit();
        let recipient_commit = recipient_leaf.commit();

        assert!(
            QuadTree::verify_auth_path(
                sender_position,
                sender_commit,
                &sender_auth_path,
                tree_root,
                DEFAULT_DEPTH
            ),
            "Sender merkle path should verify"
        );

        assert!(
            QuadTree::verify_auth_path(
                recipient_position,
                recipient_commit,
                &recipient_auth_path,
                tree_root,
                DEFAULT_DEPTH
            ),
            "Recipient merkle path should verify"
        );

        // Ephemeral secrets are different (randomized encryption)
        assert_ne!(sender_ephemeral, receiver_ephemeral);
    }

    /// Tests detection and decryption: Alice's key detects/decrypts, Bob's cannot.
    #[tokio::test]
    async fn test_end_to_end_detection_and_decryption() {
        use penumbra_sdk_keys::keys::{Diversifier, MasterComplianceKey};
        use penumbra_sdk_num::Amount;
        use penumbra_sdk_proto::core::component::shielded_pool::v1::{Output, OutputBody};
        use penumbra_sdk_proto::core::transaction::v1::{
            action::Action, Action as ActionProto, Transaction as ProtoTransaction, TransactionBody,
        };
        use rand_core::{OsRng, RngCore};

        let mut rng = OsRng;

        // Helper to create address from diversifier
        fn make_address(rng: &mut OsRng, div: [u8; 16]) -> Address {
            let scalar = decaf377::Fr::rand(rng);
            let point = decaf377::Element::GENERATOR * scalar;
            let pk_d = decaf377_ka::Public(point.vartime_compress().0);
            let mut ck_bytes = [0u8; 32];
            rng.fill_bytes(&mut ck_bytes);
            let ck = decaf377_fmd::ClueKey(ck_bytes);
            Address::from_components(Diversifier(div), pk_d, ck).unwrap()
        }

        // Create Alice and Bob with separate MCKs
        let alice_mck = MasterComplianceKey::new(decaf377::Fr::rand(&mut rng));
        let alice_diversifier = Diversifier([1u8; 16]);
        let alice_ack = alice_mck.derive_address_key(&alice_diversifier);
        let alice_address = make_address(&mut rng, [1u8; 16]);

        let bob_mck = MasterComplianceKey::new(decaf377::Fr::rand(&mut rng));
        let bob_address = make_address(&mut rng, [2u8; 16]);

        let usdc_asset_id = asset::Id(decaf377::Fq::from(999999u64));
        let amount = Amount::from(1000u128);
        let date = penumbra_sdk_keys::keys::day_index(1700000000);

        // Encrypt and create mock transaction
        let (alice_ciphertext, _) = encrypt_compliance_details(
            &mut rng,
            &alice_ack,
            &alice_address,
            date,
            usdc_asset_id,
            amount,
            bob_address.clone(),
        )
        .unwrap();

        let tx = ProtoTransaction {
            body: Some(TransactionBody {
                actions: vec![ActionProto {
                    action: Some(Action::Output(Output {
                        body: Some(OutputBody {
                            compliance_ciphertext: alice_ciphertext.to_bytes(),
                            ..Default::default()
                        }),
                        ..Default::default()
                    })),
                }],
                ..Default::default()
            }),
            ..Default::default()
        };

        // Alice's detection key can scan and detect
        let alice_detection_key =
            alice_mck.derive_daily_key(penumbra_sdk_keys::keys::KeyType::Detection, date);
        let mut detected_ciphertexts = Vec::new();
        let matches = scan_transaction(&alice_detection_key, usdc_asset_id, &tx, 100, 0, |d| {
            detected_ciphertexts.push(d.ciphertext);
            Ok(())
        })
        .unwrap();
        assert_eq!(matches, 1);
        assert_eq!(detected_ciphertexts.len(), 1);

        // Detection key only reveals asset_id
        let detection_result = alice_detection_key.try_detect_asset(
            &detected_ciphertexts[0].epk,
            &detected_ciphertexts[0].detection_tag,
        );
        assert_eq!(detection_result.unwrap(), usdc_asset_id);

        // Bob's detection key cannot detect Alice's transaction
        let bob_detection_key =
            bob_mck.derive_daily_key(penumbra_sdk_keys::keys::KeyType::Detection, date);
        let mut bob_detected = 0;
        scan_transaction(&bob_detection_key, usdc_asset_id, &tx, 100, 0, |_| {
            bob_detected += 1;
            Ok(())
        })
        .unwrap();
        assert_eq!(bob_detected, 0);

        // Full MCK can decrypt all fields
        let decrypted = decrypt_with_mck(&alice_mck, date, &detected_ciphertexts[0]).unwrap();
        assert_eq!(decrypted.asset_id, usdc_asset_id);
        assert_eq!(decrypted.amount, amount);
        assert_eq!(
            decrypted.self_diversified_generator,
            *alice_address.diversified_generator()
        );
        assert_eq!(
            decrypted.self_transmission_key,
            alice_address.transmission_key().0
        );
        assert_eq!(
            decrypted.counterparty_diversified_generator,
            *bob_address.diversified_generator()
        );
        assert_eq!(
            decrypted.counterparty_transmission_key,
            bob_address.transmission_key().0
        );

        // Wrong MCK produces garbage
        if let Ok(wrong_data) = decrypt_with_mck(&bob_mck, date, &detected_ciphertexts[0]) {
            assert_ne!(wrong_data.asset_id, usdc_asset_id);
            assert_ne!(wrong_data.amount, amount);
        }
    }
}
