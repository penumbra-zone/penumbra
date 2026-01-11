use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use cnidarium::StateWrite;
use cnidarium_component::{ActionHandler, Component};
use penumbra_sdk_asset::{STAKING_TOKEN_ASSET_ID, TEST_USD_ASSET_ID};
use penumbra_sdk_proto::StateWriteProto;
use tendermint::v0_37::abci;
use tracing::instrument;

use crate::{
    registry::{ComplianceRegistryRead, ComplianceRegistryWrite},
    structs::{MsgRegisterAsset, MsgRegisterUser},
    tree::QuadTree,
};

/// The Compliance component manages on-chain registries for regulated assets.
///
/// It maintains two Quad Merkle Trees:
/// - User tree: Maps users to their compliance viewing keys for regulated assets
/// - Asset tree: Tracks which assets are regulated
pub struct Compliance {}

#[async_trait]
impl Component for Compliance {
    type AppState = ();

    #[instrument(name = "compliance", skip(state, _app_state))]
    async fn init_chain<S: StateWrite>(mut state: S, _app_state: Option<&Self::AppState>) {
        // Initialize empty trees if they don't exist
        // This ensures the trees are properly set up at genesis

        // Check and initialize user tree
        if state.get_user_tree().await.ok().is_none() {
            let user_tree = QuadTree::new();
            let tree_bytes = bincode::serialize(&user_tree).expect("serialization should not fail");
            state.put_raw(crate::state_key::user_tree().to_string(), tree_bytes);
            state.put_proto(crate::state_key::user_count().to_string(), 0u64);
        }

        // Check and initialize asset tree
        if state.get_asset_tree().await.ok().is_none() {
            let asset_tree = QuadTree::new();
            let tree_bytes =
                bincode::serialize(&asset_tree).expect("serialization should not fail");
            state.put_raw(crate::state_key::asset_tree().to_string(), tree_bytes);
            state.put_proto(crate::state_key::asset_count().to_string(), 0u64);
        }

        // Auto-register essential assets as unregulated at genesis.
        // The staking token is required because fee payments use it, and without
        // this registration, no transactions (including asset registration txs)
        // could be submitted - a bootstrapping problem.
        // test_usd is registered for integration tests.
        for (asset_id, name) in [
            (*STAKING_TOKEN_ASSET_ID, "staking token"),
            (*TEST_USD_ASSET_ID, "test_usd"),
        ] {
            if state
                .get_asset_status(asset_id)
                .await
                .ok()
                .flatten()
                .is_none()
            {
                state
                    .update_asset_regulation(asset_id, false)
                    .await
                    .expect("must be able to register asset at genesis");
                tracing::info!("registered {} as unregulated at genesis", name);
            }
        }
    }

    #[instrument(name = "compliance", skip(_state, _begin_block))]
    async fn begin_block<S: StateWrite + 'static>(
        _state: &mut Arc<S>,
        _begin_block: &abci::request::BeginBlock,
    ) {
        // No-op for compliance component
    }

    #[instrument(name = "compliance", skip(_state, _end_block))]
    async fn end_block<S: StateWrite + 'static>(
        _state: &mut Arc<S>,
        _end_block: &abci::request::EndBlock,
    ) {
        // No-op for compliance component
    }

    async fn end_epoch<S: StateWrite + 'static>(_state: &mut Arc<S>) -> Result<()> {
        // No-op for compliance component
        Ok(())
    }
}

/// ActionHandler implementation for MsgRegisterUser.
///
/// This handler registers a user's compliance viewing key for a regulated asset.
#[async_trait]
impl ActionHandler for MsgRegisterUser {
    type CheckStatelessContext = ();

    async fn check_stateless(&self, _context: ()) -> Result<()> {
        // TODO(compliance): Verify signature proving ownership of address address.
        // The signature should be over a canonical message including the leaf commitment.
        Ok(())
    }

    async fn check_and_execute<S: StateWrite>(&self, mut state: S) -> Result<()> {
        // TODO(compliance): Check if user is already registered (idempotent but wasteful).
        // Could add check_stateful() to detect and skip duplicate registrations.
        state.add_compliance_leaf(self.leaf.clone()).await?;
        // TODO(compliance): Emit EventUserRegistered { address, asset_id, position }
        Ok(())
    }
}

/// ActionHandler implementation for MsgRegisterAsset.
///
/// This handler registers an asset's regulation status in the compliance registry.
#[async_trait]
impl ActionHandler for MsgRegisterAsset {
    type CheckStatelessContext = ();

    async fn check_stateless(&self, _context: ()) -> Result<()> {
        // TODO(compliance): Add governance authorization check.
        // Only authorized parties (e.g., asset issuer) should be able to register assets.
        Ok(())
    }

    async fn check_and_execute<S: StateWrite>(&self, mut state: S) -> Result<()> {
        // TODO(compliance): Check if asset already registered and reject re-registration.
        // Currently idempotent but may want to prevent status changes.
        state
            .update_asset_regulation(self.asset_id, self.is_regulated)
            .await?;
        // TODO(compliance): Emit EventAssetRegistered { asset_id, is_regulated, position }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cnidarium::TempStorage;
    use decaf377::Fq;
    use penumbra_sdk_asset::asset;
    use penumbra_sdk_keys::{keys::AddressComplianceKey, Address};

    use crate::structs::ComplianceLeaf;

    #[tokio::test]
    async fn test_init_chain() {
        let storage = TempStorage::new().await.unwrap();
        let snapshot = storage.latest_snapshot();
        let mut state = cnidarium::StateDelta::new(snapshot);

        // Initialize the component
        Compliance::init_chain(&mut state, None).await;

        // Verify trees were initialized
        let user_tree = state.get_user_tree().await.unwrap();
        let asset_tree = state.get_asset_tree().await.unwrap();

        assert_eq!(user_tree.depth(), 16);
        assert_eq!(asset_tree.depth(), 16);

        // Verify staking token was auto-registered as unregulated
        let staking_status = state
            .get_asset_status(*STAKING_TOKEN_ASSET_ID)
            .await
            .unwrap();
        assert_eq!(
            staking_status,
            Some(false),
            "staking token should be registered as unregulated"
        );

        // Verify test_usd was auto-registered as unregulated
        let test_usd_status = state.get_asset_status(*TEST_USD_ASSET_ID).await.unwrap();
        assert_eq!(
            test_usd_status,
            Some(false),
            "test_usd should be registered as unregulated"
        );
    }

    #[tokio::test]
    async fn test_msg_register_user() {
        let storage = TempStorage::new().await.unwrap();
        let snapshot = storage.latest_snapshot();
        let mut state = cnidarium::StateDelta::new(snapshot);

        // Initialize component
        Compliance::init_chain(&mut state, None).await;

        // Create a register user message
        let msg = MsgRegisterUser {
            leaf: ComplianceLeaf {
                address: Address::dummy(&mut rand::thread_rng()),
                key: AddressComplianceKey::new(decaf377::Element::GENERATOR),
                asset_id: asset::Id(Fq::from(1u64)),
            },
            signature: vec![0u8; 64], // Dummy signature
        };

        // Execute the action directly on state
        msg.check_and_execute(&mut state).await.unwrap();

        // Verify user was registered
        let user_count = state.get_user_count().await.unwrap();
        assert_eq!(user_count, 1);
    }

    #[tokio::test]
    async fn test_msg_register_asset() {
        let storage = TempStorage::new().await.unwrap();
        let snapshot = storage.latest_snapshot();
        let mut state = cnidarium::StateDelta::new(snapshot);

        // Initialize component (this auto-registers the staking token and test_usd)
        Compliance::init_chain(&mut state, None).await;

        // Get initial asset count (should be 2 after staking token and test_usd auto-registration)
        let initial_count = state.get_asset_count().await.unwrap();
        assert_eq!(
            initial_count, 2,
            "staking token and test_usd should be auto-registered"
        );

        // Create a register asset message
        let msg = MsgRegisterAsset {
            asset_id: asset::Id(Fq::from(123u64)),
            is_regulated: true,
        };

        // Execute the action directly on state
        msg.check_and_execute(&mut state).await.unwrap();

        // Verify new asset was registered (count should be +1)
        let asset_count = state.get_asset_count().await.unwrap();
        assert_eq!(asset_count, initial_count + 1);
    }
}
