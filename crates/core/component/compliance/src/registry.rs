use anyhow::Result;
use async_trait::async_trait;
use cnidarium::{StateRead, StateWrite};
use decaf377::Fq;
use penumbra_sdk_asset::asset;
use penumbra_sdk_proto::{StateReadProto, StateWriteProto};
use penumbra_sdk_tct::StateCommitment;

use crate::{state_key, structs::ComplianceLeaf, tree::QuadTree};

// Re-export bincode for serialization
use bincode;

/// Extension trait for reading compliance registry state.
#[async_trait]
pub trait ComplianceRegistryRead: StateRead {
    /// Get the user compliance tree from state.
    async fn get_user_tree(&self) -> Result<QuadTree> {
        match self.get_raw(state_key::user_tree()).await? {
            Some(bytes) => Ok(bincode::deserialize(&bytes)?),
            None => Ok(QuadTree::new()),
        }
    }

    /// Get the asset regulation tree from state.
    async fn get_asset_tree(&self) -> Result<QuadTree> {
        match self.get_raw(state_key::asset_tree()).await? {
            Some(bytes) => Ok(bincode::deserialize(&bytes)?),
            None => Ok(QuadTree::new()),
        }
    }

    /// Get the current user count (number of registered users).
    async fn get_user_count(&self) -> Result<u64> {
        Ok(self
            .get_proto(state_key::user_count())
            .await?
            .unwrap_or(0u64))
    }

    /// Get the current asset count (number of registered assets).
    async fn get_asset_count(&self) -> Result<u64> {
        Ok(self
            .get_proto(state_key::asset_count())
            .await?
            .unwrap_or(0u64))
    }

    /// Get the user tree root hash.
    async fn get_user_tree_root(&self) -> Result<StateCommitment> {
        let tree = self.get_user_tree().await?;
        Ok(tree.root())
    }

    /// Get the asset tree root hash.
    async fn get_asset_tree_root(&self) -> Result<StateCommitment> {
        let tree = self.get_asset_tree().await?;
        Ok(tree.root())
    }

    /// Get the position index for an asset in the asset tree.
    ///
    /// Returns the position where the asset's regulation status is stored,
    /// allowing clients to generate proofs.
    async fn get_asset_index(&self, asset_id: asset::Id) -> Result<Option<u64>> {
        self.get_proto(&state_key::asset_index(&asset_id)).await
    }

    /// Get the public regulation status of an asset.
    ///
    /// Returns whether the asset is regulated (true) or not (false).
    async fn get_asset_status(&self, asset_id: asset::Id) -> Result<Option<bool>> {
        match self.get_raw(&state_key::asset_status(&asset_id)).await? {
            Some(bytes) if bytes.len() == 1 => Ok(Some(bytes[0] != 0)),
            Some(_) => Err(anyhow::anyhow!("invalid asset status format")),
            None => Ok(None),
        }
    }

    /// Get an authentication path for a user at the given position.
    async fn get_user_auth_path(&self, position: u64) -> Result<Vec<[StateCommitment; 3]>> {
        let tree = self.get_user_tree().await?;
        Ok(tree.auth_path(position))
    }

    /// Get an authentication path for an asset at the given position.
    async fn get_asset_auth_path(&self, position: u64) -> Result<Vec<[StateCommitment; 3]>> {
        let tree = self.get_asset_tree().await?;
        Ok(tree.auth_path(position))
    }

    /// Get the position of a user's leaf in the user tree.
    ///
    /// This enables O(1) lookup for generating merkle paths during transaction planning.
    ///
    /// # Arguments
    /// * `address` - The wallet address
    /// * `asset_id` - The asset ID
    ///
    /// # Returns
    /// Returns `Some(position)` if the user is registered for this asset, `None` otherwise.
    async fn get_user_leaf_position(
        &self,
        address: &penumbra_sdk_keys::Address,
        asset_id: asset::Id,
    ) -> Result<Option<u64>> {
        let lookup_key = state_key::user_leaf_position(address, &asset_id);
        self.get_proto(&lookup_key).await
    }

    /// Get the full ComplianceLeaf for a user.
    ///
    /// This retrieves the complete leaf data (including the ACK) that was registered
    /// on-chain. This is needed for proof generation to ensure the leaf used in the
    /// proof matches what was registered.
    ///
    /// # Arguments
    /// * `address` - The wallet address
    /// * `asset_id` - The asset ID
    ///
    /// # Returns
    /// Returns `Some(ComplianceLeaf)` if the user is registered for this asset, `None` otherwise.
    async fn get_user_leaf(
        &self,
        address: &penumbra_sdk_keys::Address,
        asset_id: asset::Id,
    ) -> Result<Option<ComplianceLeaf>> {
        use penumbra_sdk_proto::DomainType;

        let lookup_key = state_key::user_leaf_data(address, &asset_id);
        match self.get_raw(&lookup_key).await? {
            Some(bytes) => {
                // Use proto decoding (ComplianceLeaf implements DomainType)
                let leaf = ComplianceLeaf::decode(bytes.as_slice())?;
                Ok(Some(leaf))
            }
            None => Ok(None),
        }
    }

    /// Verify that a compliance leaf exists on-chain by checking if its commitment
    /// is in the user tree.
    ///
    /// This function is used to verify that a leaf shared off-chain actually exists
    /// in the on-chain registry.
    ///
    /// # Arguments
    /// * `leaf` - The compliance leaf to verify
    ///
    /// # Returns
    /// Returns `Ok(true)` if the leaf's commitment is found in the tree at any position,
    /// `Ok(false)` if not found.
    ///
    /// # Note
    /// This is a linear scan through all user positions. For production, consider
    /// adding a reverse mapping from commitment to position.
    async fn verify_compliance_leaf(&self, leaf: &ComplianceLeaf) -> Result<bool> {
        let tree = self.get_user_tree().await?;
        let user_count = self.get_user_count().await?;
        let target_commitment = leaf.commit();

        // Scan through all positions to find matching commitment
        for position in 0..user_count {
            if let Some(commitment) = tree.get_leaf(position) {
                if commitment.0 == target_commitment.0 {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }
}

impl<T: StateRead + ?Sized> ComplianceRegistryRead for T {}

/// Extension trait for writing compliance registry state.
#[async_trait]
pub trait ComplianceRegistryWrite: StateWrite {
    /// Add a compliance leaf for a user.
    ///
    /// This registers a user's compliance viewing key for a regulated asset.
    /// The leaf is committed and added to the user tree at the next available position.
    ///
    /// # Arguments
    /// * `leaf` - The compliance leaf containing address, CVK, and asset_id
    async fn add_compliance_leaf(&mut self, leaf: ComplianceLeaf) -> Result<()> {
        // Load the current user tree (or create new)
        let mut tree = self.get_user_tree().await?;

        // Load the current user count (this will be our position)
        let position = self.get_user_count().await?;

        // Calculate the leaf commitment
        let commitment = leaf.commit();

        // Update the tree at the next available position
        tree.update(position, commitment);

        // Increment the user count
        let new_count = position + 1;

        // Save the updated tree and count
        let tree_bytes = bincode::serialize(&tree)?;
        self.put_raw(state_key::user_tree().to_string(), tree_bytes);
        self.put_proto(state_key::user_count().to_string(), new_count);

        // Store the reverse lookup index for O(1) position retrieval
        let lookup_key = state_key::user_leaf_position(&leaf.address, &leaf.asset_id);
        self.put_proto(lookup_key, position);

        // Store the full leaf data for later retrieval during proof generation
        // Use proto encoding since ComplianceLeaf has serde(try_from/into proto) attributes
        use penumbra_sdk_proto::DomainType;
        let leaf_data_key = state_key::user_leaf_data(&leaf.address, &leaf.asset_id);
        let leaf_bytes = leaf.encode_to_vec();
        self.put_raw(leaf_data_key, leaf_bytes);

        Ok(())
    }

    /// Update an asset's regulation status.
    ///
    /// This registers whether an asset is regulated (requires compliance) or not.
    /// The status is committed and added to the asset tree.
    ///
    /// # Arguments
    /// * `asset_id` - The asset ID to register
    /// * `is_regulated` - Whether the asset is regulated
    ///
    /// # Note
    /// For demo purposes, this function does not allow updating existing assets.
    /// Once an asset is registered, attempting to re-register it will return an error.
    /// TODO: Implement proper update logic that modifies the existing tree entry instead
    /// of creating duplicates.
    async fn update_asset_regulation(
        &mut self,
        asset_id: asset::Id,
        is_regulated: bool,
    ) -> Result<()> {
        // Check if asset is already registered
        if let Some(_existing_index) = self.get_asset_index(asset_id).await? {
            anyhow::bail!(
                "Asset {} is already registered in compliance registry. \
                Re-registration is not supported in this demo version.",
                asset_id
            );
        }

        // Load the current asset tree (or create new)
        let mut tree = self.get_asset_tree().await?;

        // Load the current asset count
        let asset_count = self.get_asset_count().await?;

        // Calculate the leaf commitment for this asset regulation status
        // Hash: poseidon377::hash_2(domain_sep, (asset_id, is_regulated))
        let regulation_flag = if is_regulated {
            Fq::from(1u64)
        } else {
            Fq::from(0u64)
        };
        let domain_sep = Fq::from(0u64); // Use zero as domain separator
        let commitment_hash = poseidon377::hash_2(&domain_sep, (asset_id.0, regulation_flag));
        let commitment = StateCommitment(commitment_hash);

        // Update the tree at the next available position
        tree.update(asset_count, commitment);

        // Increment the asset count
        let new_count = asset_count + 1;

        // Save the updated tree and count
        let tree_bytes = bincode::serialize(&tree)?;
        self.put_raw(state_key::asset_tree().to_string(), tree_bytes);
        self.put_proto(state_key::asset_count().to_string(), new_count);

        // Store the index mapping so clients can look up the position
        self.put_proto(state_key::asset_index(&asset_id), asset_count);

        // Store the public regulation status as a single byte (0 = not regulated, 1 = regulated)
        self.put_raw(
            state_key::asset_status(&asset_id),
            vec![u8::from(is_regulated)],
        );

        Ok(())
    }
}

impl<T: StateWrite + ?Sized> ComplianceRegistryWrite for T {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree::QuadTree;
    use cnidarium::TempStorage;
    use penumbra_sdk_keys::{keys::AddressComplianceKey, Address};

    #[tokio::test]
    async fn test_add_compliance_leaf() {
        let storage = TempStorage::new().await.unwrap();
        let snapshot = storage.latest_snapshot();
        let mut state = cnidarium::StateDelta::new(snapshot);

        // Create a dummy compliance leaf
        let leaf = ComplianceLeaf {
            address: Address::dummy(&mut rand::thread_rng()),
            key: AddressComplianceKey::new(decaf377::Element::GENERATOR),
            asset_id: asset::Id(Fq::from(1u64)),
        };

        // Add the leaf
        state.add_compliance_leaf(leaf.clone()).await.unwrap();

        // Check that user count increased
        let count = state.get_user_count().await.unwrap();
        assert_eq!(count, 1);

        // Check that the tree root changed
        let root = state.get_user_tree_root().await.unwrap();
        assert_ne!(root.0, Fq::from(0u64));
    }

    #[tokio::test]
    async fn test_update_asset_regulation() {
        let storage = TempStorage::new().await.unwrap();
        let snapshot = storage.latest_snapshot();
        let mut state = cnidarium::StateDelta::new(snapshot);

        let asset_id = asset::Id(Fq::from(123u64));

        // Register an asset as regulated
        state.update_asset_regulation(asset_id, true).await.unwrap();

        // Check that asset count increased
        let count = state.get_asset_count().await.unwrap();
        assert_eq!(count, 1);

        // Check that the tree root changed
        let root = state.get_asset_tree_root().await.unwrap();
        assert_ne!(root.0, Fq::from(0u64));

        // Verify the index mapping was stored
        let index = state.get_asset_index(asset_id).await.unwrap();
        assert_eq!(index, Some(0), "First asset should be at index 0");

        // Verify the public status was stored
        let status = state.get_asset_status(asset_id).await.unwrap();
        assert_eq!(status, Some(true), "Asset should be marked as regulated");

        // Test with a second asset (unregulated)
        let asset_id_2 = asset::Id(Fq::from(456u64));
        state
            .update_asset_regulation(asset_id_2, false)
            .await
            .unwrap();

        let index_2 = state.get_asset_index(asset_id_2).await.unwrap();
        assert_eq!(index_2, Some(1), "Second asset should be at index 1");

        let status_2 = state.get_asset_status(asset_id_2).await.unwrap();
        assert_eq!(
            status_2,
            Some(false),
            "Second asset should be marked as unregulated"
        );
    }

    #[tokio::test]
    async fn test_multiple_leaves() {
        let storage = TempStorage::new().await.unwrap();
        let snapshot = storage.latest_snapshot();
        let mut state = cnidarium::StateDelta::new(snapshot);

        let mut rng = rand::thread_rng();

        // Add multiple leaves
        for i in 0..5 {
            let leaf = ComplianceLeaf {
                address: Address::dummy(&mut rng),
                key: AddressComplianceKey::new(
                    decaf377::Element::GENERATOR * decaf377::Fr::from(i as u64 + 1),
                ),
                asset_id: asset::Id(Fq::from(i as u64)),
            };
            state.add_compliance_leaf(leaf).await.unwrap();
        }

        // Check that user count is correct
        let count = state.get_user_count().await.unwrap();
        assert_eq!(count, 5);

        // Verify we can get auth paths for each position
        for pos in 0..5 {
            let path = state.get_user_auth_path(pos).await.unwrap();
            assert_eq!(path.len(), 16); // DEFAULT_DEPTH
        }
    }

    #[tokio::test]
    async fn test_asset_duplicate_prevention() {
        let storage = TempStorage::new().await.unwrap();
        let snapshot = storage.latest_snapshot();
        let mut state = cnidarium::StateDelta::new(snapshot);

        let asset_id = asset::Id(Fq::from(789u64));

        // First registration should succeed
        state
            .update_asset_regulation(asset_id, true)
            .await
            .expect("First registration should succeed");

        // Verify asset was registered
        let status = state.get_asset_status(asset_id).await.unwrap();
        assert_eq!(status, Some(true));

        // Second registration of same asset should fail
        let result = state.update_asset_regulation(asset_id, false).await;
        assert!(result.is_err(), "Duplicate registration should fail");

        // Verify error message mentions duplicate
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("already registered"),
            "Error should mention asset is already registered, got: {}",
            err_msg
        );

        // Verify asset count didn't increase
        let count = state.get_asset_count().await.unwrap();
        assert_eq!(
            count, 1,
            "Asset count should remain 1 after failed duplicate registration"
        );

        // Verify original status unchanged
        let status = state.get_asset_status(asset_id).await.unwrap();
        assert_eq!(status, Some(true), "Original status should be unchanged");
    }

    #[tokio::test]
    async fn test_verify_compliance_leaf() {
        let storage = TempStorage::new().await.unwrap();
        let snapshot = storage.latest_snapshot();
        let mut state = cnidarium::StateDelta::new(snapshot);

        let mut rng = rand::thread_rng();

        // Create a compliance leaf
        let leaf = ComplianceLeaf {
            address: Address::dummy(&mut rng),
            key: AddressComplianceKey::new(decaf377::Element::GENERATOR),
            asset_id: asset::Id(Fq::from(100u64)),
        };

        // Before adding, verification should fail
        let verified = state.verify_compliance_leaf(&leaf).await.unwrap();
        assert!(!verified, "Leaf should not be verified before being added");

        // Add the leaf to the registry
        state.add_compliance_leaf(leaf.clone()).await.unwrap();

        // After adding, verification should succeed
        let verified = state.verify_compliance_leaf(&leaf).await.unwrap();
        assert!(verified, "Leaf should be verified after being added");

        // Create a different leaf with same asset but different wallet
        let different_leaf = ComplianceLeaf {
            address: Address::dummy(&mut rng),
            key: AddressComplianceKey::new(decaf377::Element::GENERATOR * decaf377::Fr::from(2u64)),
            asset_id: asset::Id(Fq::from(100u64)),
        };

        // Different leaf should not verify
        let verified = state.verify_compliance_leaf(&different_leaf).await.unwrap();
        assert!(!verified, "Different leaf should not be verified");
    }

    #[tokio::test]
    async fn test_leaf_json_serialization() {
        let mut rng = rand::thread_rng();

        // Create a compliance leaf
        let original_leaf = ComplianceLeaf {
            address: Address::dummy(&mut rng),
            key: AddressComplianceKey::new(decaf377::Element::GENERATOR),
            asset_id: asset::Id(Fq::from(200u64)),
        };

        // Export to JSON
        let json = original_leaf
            .to_json()
            .expect("JSON serialization should succeed");

        // JSON should not be empty
        assert!(!json.is_empty(), "JSON should not be empty");

        // Import from JSON
        let imported_leaf =
            ComplianceLeaf::from_json(&json).expect("JSON deserialization should succeed");

        // Should be equal to original
        assert_eq!(
            original_leaf, imported_leaf,
            "Imported leaf should match original"
        );

        // Commitments should also match
        assert_eq!(
            original_leaf.commit().0,
            imported_leaf.commit().0,
            "Commitments should match"
        );
    }

    #[tokio::test]
    async fn test_share_and_verify_workflow() {
        let storage = TempStorage::new().await.unwrap();
        let snapshot = storage.latest_snapshot();
        let mut state = cnidarium::StateDelta::new(snapshot);

        let mut rng = rand::thread_rng();

        // User creates their compliance leaf (private)
        let user_leaf = ComplianceLeaf {
            address: Address::dummy(&mut rng),
            key: AddressComplianceKey::new(decaf377::Element::GENERATOR),
            asset_id: asset::Id(Fq::from(300u64)),
        };

        // User registers on-chain
        state.add_compliance_leaf(user_leaf.clone()).await.unwrap();

        // User exports their leaf to share off-chain with issuer
        let shared_json = user_leaf.to_json().expect("Export should succeed");

        // Issuer receives the JSON and imports it
        let received_leaf = ComplianceLeaf::from_json(&shared_json).expect("Import should succeed");

        // Issuer verifies that this leaf exists on-chain
        let is_valid = state.verify_compliance_leaf(&received_leaf).await.unwrap();
        assert!(
            is_valid,
            "Issuer should be able to verify the shared leaf exists on-chain"
        );

        // Issuer can now use the received_leaf.key (ACK) to encrypt compliance data
        assert_eq!(
            user_leaf.key.inner(),
            received_leaf.key.inner(),
            "ACK should be preserved through sharing"
        );
    }

    #[tokio::test]
    async fn test_verify_with_multiple_leaves() {
        let storage = TempStorage::new().await.unwrap();
        let snapshot = storage.latest_snapshot();
        let mut state = cnidarium::StateDelta::new(snapshot);

        let mut rng = rand::thread_rng();

        // Add multiple leaves
        let mut leaves = Vec::new();
        for i in 0..5u64 {
            let leaf = ComplianceLeaf {
                address: Address::dummy(&mut rng),
                key: AddressComplianceKey::new(
                    decaf377::Element::GENERATOR * decaf377::Fr::from(i + 1),
                ),
                asset_id: asset::Id(Fq::from(i)),
            };
            state.add_compliance_leaf(leaf.clone()).await.unwrap();
            leaves.push(leaf);
        }

        // All added leaves should verify
        for leaf in &leaves {
            let verified = state.verify_compliance_leaf(leaf).await.unwrap();
            assert!(verified, "All added leaves should verify");
        }

        // A new leaf not in the tree should not verify
        let new_leaf = ComplianceLeaf {
            address: Address::dummy(&mut rng),
            key: AddressComplianceKey::new(
                decaf377::Element::GENERATOR * decaf377::Fr::from(999u64),
            ),
            asset_id: asset::Id(Fq::from(999u64)),
        };
        let verified = state.verify_compliance_leaf(&new_leaf).await.unwrap();
        assert!(!verified, "Non-registered leaf should not verify");
    }

    #[tokio::test]
    async fn test_phase1_comprehensive_integration() {
        let storage = TempStorage::new().await.unwrap();
        let snapshot = storage.latest_snapshot();
        let mut state = cnidarium::StateDelta::new(snapshot);
        let mut rng = rand::thread_rng();

        // Bridged asset (USDC) - regulated
        let usdc_asset_id = asset::Id(Fq::from(12345u64));
        state
            .update_asset_regulation(usdc_asset_id, true)
            .await
            .unwrap();
        assert_eq!(
            state.get_asset_status(usdc_asset_id).await.unwrap(),
            Some(true)
        );
        assert_eq!(state.get_asset_index(usdc_asset_id).await.unwrap(), Some(0));
        assert_ne!(state.get_asset_tree_root().await.unwrap().0, Fq::from(0u64));

        // Native asset (penumbra) - unregulated
        let penumbra_asset_id = asset::Id(Fq::from(1u64));
        state
            .update_asset_regulation(penumbra_asset_id, false)
            .await
            .unwrap();
        assert_eq!(
            state.get_asset_status(penumbra_asset_id).await.unwrap(),
            Some(false)
        );
        assert_eq!(
            state.get_asset_index(penumbra_asset_id).await.unwrap(),
            Some(1)
        );

        // Multiple wallets for same user
        let wallet1 = Address::dummy(&mut rng);
        let ack1 =
            AddressComplianceKey::new(decaf377::Element::GENERATOR * decaf377::Fr::from(101u64));
        let leaf1 = ComplianceLeaf {
            address: wallet1.clone(),
            key: ack1,
            asset_id: usdc_asset_id,
        };

        let leaf2 = ComplianceLeaf {
            address: Address::dummy(&mut rng),
            key: AddressComplianceKey::new(
                decaf377::Element::GENERATOR * decaf377::Fr::from(102u64),
            ),
            asset_id: usdc_asset_id,
        };
        let leaf3 = ComplianceLeaf {
            address: Address::dummy(&mut rng),
            key: AddressComplianceKey::new(
                decaf377::Element::GENERATOR * decaf377::Fr::from(103u64),
            ),
            asset_id: usdc_asset_id,
        };

        state.add_compliance_leaf(leaf1.clone()).await.unwrap();
        state.add_compliance_leaf(leaf2.clone()).await.unwrap();
        state.add_compliance_leaf(leaf3.clone()).await.unwrap();
        assert_eq!(state.get_user_count().await.unwrap(), 3);
        assert!(state.verify_compliance_leaf(&leaf1).await.unwrap());
        assert!(state.verify_compliance_leaf(&leaf2).await.unwrap());
        assert!(state.verify_compliance_leaf(&leaf3).await.unwrap());

        // Share and verify workflow
        let shared_json = leaf1.to_json().unwrap();
        let received_leaf = ComplianceLeaf::from_json(&shared_json).unwrap();
        assert!(state.verify_compliance_leaf(&received_leaf).await.unwrap());
        assert_eq!(received_leaf.key.inner(), ack1.inner());

        // Query non-existent asset
        let unknown_asset = asset::Id(Fq::from(99999u64));
        assert_eq!(state.get_asset_status(unknown_asset).await.unwrap(), None);
        assert_eq!(state.get_asset_index(unknown_asset).await.unwrap(), None);

        // Authentication paths
        let path = state.get_user_auth_path(0).await.unwrap();
        assert_eq!(path.len(), 16);
        let user_root = state.get_user_tree_root().await.unwrap();
        let tree = state.get_user_tree().await.unwrap();
        assert!(QuadTree::verify_auth_path(
            0,
            leaf1.commit(),
            &path,
            user_root,
            tree.depth()
        ));
        assert_eq!(state.get_asset_count().await.unwrap(), 2);

        // Same wallet registered for multiple assets
        let dai_asset_id = asset::Id(Fq::from(67890u64));
        state
            .update_asset_regulation(dai_asset_id, true)
            .await
            .unwrap();
        let leaf1_dai = ComplianceLeaf {
            address: wallet1,
            key: AddressComplianceKey::new(
                decaf377::Element::GENERATOR * decaf377::Fr::from(201u64),
            ),
            asset_id: dai_asset_id,
        };
        state.add_compliance_leaf(leaf1_dai.clone()).await.unwrap();
        assert!(state.verify_compliance_leaf(&leaf1).await.unwrap());
        assert!(state.verify_compliance_leaf(&leaf1_dai).await.unwrap());
        assert_eq!(state.get_user_count().await.unwrap(), 4);
    }

    #[tokio::test]
    async fn test_user_leaf_position_lookup() {
        let storage = TempStorage::new().await.unwrap();
        let snapshot = storage.latest_snapshot();
        let mut state = cnidarium::StateDelta::new(snapshot);
        let mut rng = rand::thread_rng();

        let wallet1 = Address::dummy(&mut rng);
        let wallet2 = Address::dummy(&mut rng);
        let usdc = asset::Id(Fq::from(12345u64));
        let dai = asset::Id(Fq::from(67890u64));

        let leaf1 = ComplianceLeaf {
            address: wallet1.clone(),
            key: AddressComplianceKey::new(
                decaf377::Element::GENERATOR * decaf377::Fr::from(101u64),
            ),
            asset_id: usdc,
        };
        let leaf2 = ComplianceLeaf {
            address: wallet1.clone(),
            key: AddressComplianceKey::new(
                decaf377::Element::GENERATOR * decaf377::Fr::from(102u64),
            ),
            asset_id: dai,
        };
        let leaf3 = ComplianceLeaf {
            address: wallet2.clone(),
            key: AddressComplianceKey::new(
                decaf377::Element::GENERATOR * decaf377::Fr::from(103u64),
            ),
            asset_id: usdc,
        };

        state.add_compliance_leaf(leaf1.clone()).await.unwrap();
        state.add_compliance_leaf(leaf2.clone()).await.unwrap();
        state.add_compliance_leaf(leaf3.clone()).await.unwrap();

        // Position lookups
        assert_eq!(
            state.get_user_leaf_position(&wallet1, usdc).await.unwrap(),
            Some(0)
        );
        assert_eq!(
            state.get_user_leaf_position(&wallet1, dai).await.unwrap(),
            Some(1)
        );
        assert_eq!(
            state.get_user_leaf_position(&wallet2, usdc).await.unwrap(),
            Some(2)
        );
        assert_eq!(
            state
                .get_user_leaf_position(&Address::dummy(&mut rng), usdc)
                .await
                .unwrap(),
            None
        );

        // Auth paths verify correctly
        let tree = state.get_user_tree().await.unwrap();
        let root = tree.root();
        let path0 = state.get_user_auth_path(0).await.unwrap();
        let path1 = state.get_user_auth_path(1).await.unwrap();
        assert_eq!(path0.len(), 16);
        assert!(QuadTree::verify_auth_path(
            0,
            leaf1.commit(),
            &path0,
            root,
            tree.depth()
        ));
        assert!(QuadTree::verify_auth_path(
            1,
            leaf2.commit(),
            &path1,
            root,
            tree.depth()
        ));
    }

    /// Tests that get_user_leaf() returns the exact registered leaf (catches ACK mismatch bugs).
    #[tokio::test]
    async fn test_user_leaf_roundtrip() {
        let storage = TempStorage::new().await.unwrap();
        let snapshot = storage.latest_snapshot();
        let mut state = cnidarium::StateDelta::new(snapshot);
        let mut rng = rand::thread_rng();

        let wallet = Address::dummy(&mut rng);
        let ack =
            AddressComplianceKey::new(decaf377::Element::GENERATOR * decaf377::Fr::from(42u64));
        let asset_id = asset::Id(Fq::from(12345u64));

        let original_leaf = ComplianceLeaf {
            address: wallet.clone(),
            key: ack,
            asset_id,
        };
        state
            .add_compliance_leaf(original_leaf.clone())
            .await
            .unwrap();

        let fetched_leaf = state
            .get_user_leaf(&wallet, asset_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(original_leaf.key.inner(), fetched_leaf.key.inner());
        assert_eq!(original_leaf.address, fetched_leaf.address);
        assert_eq!(original_leaf.asset_id, fetched_leaf.asset_id);
        assert_eq!(original_leaf.commit().0, fetched_leaf.commit().0);
        assert!(state
            .get_user_leaf(&Address::dummy(&mut rng), asset_id)
            .await
            .unwrap()
            .is_none());
    }
}
