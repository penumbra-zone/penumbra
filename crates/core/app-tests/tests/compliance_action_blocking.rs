//! Tests that regulated assets are blocked in non-transfer actions.

mod common;

use {
    anyhow::Result,
    cnidarium::{ArcStateDeltaExt, StateDelta, TempStorage},
    cnidarium_component::ActionHandler,
    common::TempStorageExt,
    penumbra_sdk_asset::{asset, Value},
    penumbra_sdk_community_pool::CommunityPoolDeposit,
    penumbra_sdk_compliance::registry::ComplianceRegistryWrite,
    penumbra_sdk_dex::{
        lp::{position::Position, Reserves},
        swap::{SwapPlaintext, SwapPlan},
        DirectedTradingPair, TradingPair,
    },
    penumbra_sdk_fee::Fee,
    penumbra_sdk_keys::{test_keys, Address},
    penumbra_sdk_num::Amount,
    penumbra_sdk_sct::component::source::SourceContext,
    rand_core::SeedableRng,
    std::{ops::Deref, sync::Arc},
};

// ============================================================================
// Swap Tests
// ============================================================================

#[tokio::test]
async fn swap_rejects_regulated_asset() -> Result<()> {
    let mut rng = rand_chacha::ChaChaRng::seed_from_u64(1234);
    let storage = TempStorage::new_with_penumbra_prefixes()
        .await?
        .apply_default_genesis()
        .await?;

    let gm = asset::Cache::with_known_assets().get_unit("gm").unwrap();
    let gn = asset::Cache::with_known_assets().get_unit("gn").unwrap();

    // Register gm as regulated
    {
        let mut state = StateDelta::new(storage.latest_snapshot());
        state.update_asset_regulation(gm.id(), true).await?;
        storage.commit(state).await?;
    }

    // Create swap with regulated asset
    let trading_pair = TradingPair::new(gm.id(), gn.id());
    let delta_1 = Amount::from(100_000u64);
    let delta_2 = Amount::from(0u64);
    let fee = Fee::default();
    let claim_address: Address = test_keys::ADDRESS_0.deref().clone();
    let plaintext =
        SwapPlaintext::new(&mut rng, trading_pair, delta_1, delta_2, fee, claim_address);
    let swap_plan = SwapPlan::new(&mut rng, plaintext);
    let swap = swap_plan.swap(&test_keys::FULL_VIEWING_KEY);

    // Execute should fail
    let mut state = Arc::new(StateDelta::new(storage.latest_snapshot()));
    let mut state_tx = state.try_begin_transaction().unwrap();
    state_tx.put_mock_source(1u8);
    let result = swap.check_and_execute(&mut state_tx).await;

    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("egulated") || err_msg.contains("Swap"),
        "Error should mention regulated assets or Swap action: {}",
        err_msg
    );

    Ok(())
}

#[tokio::test]
async fn swap_allows_unregulated_asset() -> Result<()> {
    let mut rng = rand_chacha::ChaChaRng::seed_from_u64(1234);
    let storage = TempStorage::new_with_penumbra_prefixes()
        .await?
        .apply_default_genesis()
        .await?;

    // gm and gn are unregulated by default
    let gm = asset::Cache::with_known_assets().get_unit("gm").unwrap();
    let gn = asset::Cache::with_known_assets().get_unit("gn").unwrap();

    let trading_pair = TradingPair::new(gm.id(), gn.id());
    let delta_1 = Amount::from(100_000u64);
    let delta_2 = Amount::from(0u64);
    let fee = Fee::default();
    let claim_address: Address = test_keys::ADDRESS_0.deref().clone();
    let plaintext =
        SwapPlaintext::new(&mut rng, trading_pair, delta_1, delta_2, fee, claim_address);
    let swap_plan = SwapPlan::new(&mut rng, plaintext);
    let swap = swap_plan.swap(&test_keys::FULL_VIEWING_KEY);

    // Execute should succeed
    let mut state = Arc::new(StateDelta::new(storage.latest_snapshot()));
    let mut state_tx = state.try_begin_transaction().unwrap();
    state_tx.put_mock_source(1u8);
    swap.check_and_execute(&mut state_tx).await?;

    Ok(())
}

// ============================================================================
// PositionOpen Tests
// ============================================================================

#[tokio::test]
async fn position_open_rejects_regulated_asset() -> Result<()> {
    use penumbra_sdk_dex::lp::action::PositionOpen;

    let mut rng = rand_chacha::ChaChaRng::seed_from_u64(5678);
    let storage = TempStorage::new_with_penumbra_prefixes()
        .await?
        .apply_default_genesis()
        .await?;

    let gm = asset::Cache::with_known_assets().get_unit("gm").unwrap();
    let gn = asset::Cache::with_known_assets().get_unit("gn").unwrap();

    // Register gm as regulated
    {
        let mut state = StateDelta::new(storage.latest_snapshot());
        state.update_asset_regulation(gm.id(), true).await?;
        storage.commit(state).await?;
    }

    // Create position with regulated asset
    let pair = DirectedTradingPair::new(gm.id(), gn.id());
    let position = Position::new(
        &mut rng,
        pair,
        0,                        // fee
        1u64.into(),              // p
        1u64.into(),              // q
        Reserves {
            r1: 100_000u64.into(),
            r2: 100_000u64.into(),
        },
    );
    let action = PositionOpen {
        position,
        encrypted_metadata: None,
    };

    // Execute should fail
    let mut state = Arc::new(StateDelta::new(storage.latest_snapshot()));
    let mut state_tx = state.try_begin_transaction().unwrap();
    state_tx.put_mock_source(1u8);
    let result = action.check_and_execute(&mut state_tx).await;

    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("egulated") || err_msg.contains("PositionOpen"),
        "Error should mention regulated assets or PositionOpen action: {}",
        err_msg
    );

    Ok(())
}

#[tokio::test]
async fn position_open_allows_unregulated_asset() -> Result<()> {
    use penumbra_sdk_dex::lp::action::PositionOpen;

    let mut rng = rand_chacha::ChaChaRng::seed_from_u64(5678);
    let storage = TempStorage::new_with_penumbra_prefixes()
        .await?
        .apply_default_genesis()
        .await?;

    // gm and gn are unregulated by default
    let gm = asset::Cache::with_known_assets().get_unit("gm").unwrap();
    let gn = asset::Cache::with_known_assets().get_unit("gn").unwrap();

    let pair = DirectedTradingPair::new(gm.id(), gn.id());
    let position = Position::new(
        &mut rng,
        pair,
        0,                        // fee
        1u64.into(),              // p
        1u64.into(),              // q
        Reserves {
            r1: 100_000u64.into(),
            r2: 100_000u64.into(),
        },
    );
    let action = PositionOpen {
        position,
        encrypted_metadata: None,
    };

    // Execute should succeed
    let mut state = Arc::new(StateDelta::new(storage.latest_snapshot()));
    let mut state_tx = state.try_begin_transaction().unwrap();
    state_tx.put_mock_source(1u8);
    action.check_and_execute(&mut state_tx).await?;

    Ok(())
}

// ============================================================================
// CommunityPoolDeposit Tests
// ============================================================================

#[tokio::test]
async fn community_pool_deposit_rejects_regulated_asset() -> Result<()> {
    let storage = TempStorage::new_with_penumbra_prefixes()
        .await?
        .apply_default_genesis()
        .await?;

    let gm = asset::Cache::with_known_assets().get_unit("gm").unwrap();

    // Register gm as regulated
    {
        let mut state = StateDelta::new(storage.latest_snapshot());
        state.update_asset_regulation(gm.id(), true).await?;
        storage.commit(state).await?;
    }

    // Create deposit with regulated asset
    let value = Value {
        amount: 100_000u64.into(),
        asset_id: gm.id(),
    };
    let action = CommunityPoolDeposit { value };

    // Execute should fail
    let mut state = Arc::new(StateDelta::new(storage.latest_snapshot()));
    let mut state_tx = state.try_begin_transaction().unwrap();
    state_tx.put_mock_source(1u8);
    let result = action.check_and_execute(&mut state_tx).await;

    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("egulated") || err_msg.contains("CommunityPoolDeposit"),
        "Error should mention regulated assets or CommunityPoolDeposit action: {}",
        err_msg
    );

    Ok(())
}

#[tokio::test]
async fn community_pool_deposit_allows_unregulated_asset() -> Result<()> {
    let storage = TempStorage::new_with_penumbra_prefixes()
        .await?
        .apply_default_genesis()
        .await?;

    // gm is unregulated by default
    let gm = asset::Cache::with_known_assets().get_unit("gm").unwrap();

    let value = Value {
        amount: 100_000u64.into(),
        asset_id: gm.id(),
    };
    let action = CommunityPoolDeposit { value };

    // Execute should succeed
    let mut state = Arc::new(StateDelta::new(storage.latest_snapshot()));
    let mut state_tx = state.try_begin_transaction().unwrap();
    state_tx.put_mock_source(1u8);
    action.check_and_execute(&mut state_tx).await?;

    Ok(())
}
