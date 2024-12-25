mod common;

use self::common::TempStorageExt;
use cnidarium::{ArcStateDeltaExt, StateDelta, TempStorage};
use cnidarium_component::{ActionHandler, Component};
use decaf377::Fq;
use penumbra_sdk_asset::asset;
use penumbra_sdk_compact_block::component::CompactBlockManager as _;
use penumbra_sdk_dex::{
    component::{Dex, StateReadExt as _},
    swap::{SwapPlaintext, SwapPlan},
    swap_claim::SwapClaimPlan,
    TradingPair,
};
use penumbra_sdk_fee::Fee;
use penumbra_sdk_keys::{test_keys, Address};
use penumbra_sdk_mock_client::MockClient;
use penumbra_sdk_num::Amount;
use penumbra_sdk_sct::{
    component::{clock::EpochManager, source::SourceContext as _, StateReadExt as _},
    epoch::Epoch,
};
use penumbra_sdk_shielded_pool::component::ShieldedPool;
use penumbra_sdk_transaction::Transaction;
use rand_core::SeedableRng;
use std::{ops::Deref, sync::Arc};
use tendermint::abci;

#[tokio::test]
async fn swap_and_swap_claim() -> anyhow::Result<()> {
    let mut rng = rand_chacha::ChaChaRng::seed_from_u64(1312);

    let storage = TempStorage::new_with_penumbra_prefixes()
        .await?
        .apply_default_genesis()
        .await?;
    let mut state = Arc::new(StateDelta::new(storage.latest_snapshot()));

    let height = 1;

    // 1. Simulate BeginBlock

    let mut state_tx = state.try_begin_transaction().unwrap();
    state_tx.put_epoch_by_height(
        height,
        Epoch {
            index: 0,
            start_height: 0,
        },
    );
    state_tx.put_block_height(height);
    state_tx.apply();

    // 2. Create a Swap action

    let gm = asset::Cache::with_known_assets().get_unit("gm").unwrap();
    let gn = asset::Cache::with_known_assets().get_unit("gn").unwrap();
    let trading_pair = TradingPair::new(gm.id(), gn.id());

    let delta_1 = Amount::from(100_000u64);
    let delta_2 = Amount::from(0u64);
    let fee = Fee::default();
    let claim_address: Address = test_keys::ADDRESS_0.deref().clone();

    let plaintext =
        SwapPlaintext::new(&mut rng, trading_pair, delta_1, delta_2, fee, claim_address);

    let swap_plan = SwapPlan::new(&mut rng, plaintext.clone());
    let swap = swap_plan.swap(&test_keys::FULL_VIEWING_KEY);

    // 3. Simulate execution of the Swap action

    swap.check_stateless(()).await?;
    swap.check_historical(state.clone()).await?;
    let mut state_tx = state.try_begin_transaction().unwrap();
    state_tx.put_mock_source(1u8);
    swap.check_and_execute(&mut state_tx).await?;
    state_tx.apply();

    // 4. Execute EndBlock (where the swap is actually executed)

    let end_block = abci::request::EndBlock {
        height: height.try_into().unwrap(),
    };
    // Execute EndBlock for the Dex, to actually execute the swaps...
    Dex::end_block(&mut state, &end_block).await;
    ShieldedPool::end_block(&mut state, &end_block).await;

    let mut state_tx = state.try_begin_transaction().unwrap();
    // ... and for the App, call `finish_block` to correctly write out the SCT with the data we'll use next.
    state_tx.finish_block().await.unwrap();

    state_tx.apply();

    // 6. Create a SwapClaim action

    // To do this, we need to have an auth path for the swap, which
    // means we have to synchronize a client's view of the test chain's SCT
    // state.
    let epoch_duration = state.get_epoch_duration_parameter().await?;
    let mut client = MockClient::new(test_keys::SPEND_KEY.clone());
    // TODO: generalize StateRead/StateWrite impls from impl for &S to impl for Deref<Target=S>
    client.sync_to(1, state.deref()).await?;

    let output_data = state.output_data(height, trading_pair).await?.unwrap();

    let commitment = swap.body.payload.commitment;
    let swap_auth_path = client.witness_commitment(commitment).unwrap();
    let detected_plaintext = client.swap_by_commitment(&commitment).unwrap();
    assert_eq!(plaintext, detected_plaintext);

    let claim_plan = SwapClaimPlan {
        swap_plaintext: plaintext,
        position: swap_auth_path.position(),
        output_data,
        epoch_duration,
        proof_blinding_r: Fq::rand(&mut rng),
        proof_blinding_s: Fq::rand(&mut rng),
    };
    let claim = claim_plan.swap_claim(&test_keys::FULL_VIEWING_KEY, &swap_auth_path);

    // 7. Execute the SwapClaim action

    // The SwapClaim ActionHandler uses the transaction's anchor to check proofs:
    let context = &Transaction {
        anchor: client.latest_height_and_sct_root().1,
        ..Default::default()
    }
    .context();

    claim.check_stateless(context.clone()).await?;
    claim.check_historical(state.clone()).await?;
    let mut state_tx = state.try_begin_transaction().unwrap();
    state_tx.put_mock_source(2u8);
    claim.check_and_execute(&mut state_tx).await?;
    state_tx.apply();

    Ok(())
}

/*
#[tokio::test]
#[should_panic(expected = "was already spent")]
async fn swap_claim_duplicate_nullifier_previous_transaction() {
    let mut rng = rand_chacha::ChaChaRng::seed_from_u64(1312);

    let storage = TempStorage::new_with_penumbra_prefixes()
        .await
        .unwrap()
        .apply_default_genesis()
        .await
        .unwrap();
    let mut state = Arc::new(StateDelta::new(storage.latest_snapshot()));

    let height = 1;

    // 1. Simulate BeginBlock

    let mut state_tx = state.try_begin_transaction().unwrap();
    state_tx.put_epoch_by_height(
        height,
        Epoch {
            index: 0,
            start_height: 0,
        },
    );
    state_tx.put_block_height(height);
    state_tx.apply();

    // 2. Create a Swap action

    let gm = asset::Cache::with_known_assets().get_unit("gm").unwrap();
    let gn = asset::Cache::with_known_assets().get_unit("gn").unwrap();
    let trading_pair = TradingPair::new(gm.id(), gn.id());

    let delta_1 = Amount::from(100_000u64);
    let delta_2 = Amount::from(0u64);
    let fee = Fee::default();
    let claim_address: Address = *test_keys::ADDRESS_0;

    let plaintext =
        SwapPlaintext::new(&mut rng, trading_pair, delta_1, delta_2, fee, claim_address);

    let swap_plan = SwapPlan::new(&mut rng, plaintext.clone());
    let swap = swap_plan.swap(&test_keys::FULL_VIEWING_KEY);

    // 3. Simulate execution of the Swap action

    swap.check_stateless(()).await.unwrap();
    swap.check_historical(state.clone()).await.unwrap();
    let mut state_tx = state.try_begin_transaction().unwrap();
    state_tx.put_mock_source(1u8);
    swap.check_and_execute(&mut state_tx).await.unwrap();
    state_tx.apply();

    // 4. Execute EndBlock (where the swap is actually executed)

    let end_block = abci::request::EndBlock {
        height: height.try_into().unwrap(),
    };
    // Execute EndBlock for the Dex, to actually execute the swaps...
    Dex::end_block(&mut state, &end_block).await;
    ShieldedPool::end_block(&mut state, &end_block).await;

    let mut state_tx = state.try_begin_transaction().unwrap();
    // ... and for the App, call `finish_block` to correctly write out the SCT with the data we'll use next.
    state_tx.finish_block(false).await.unwrap();

    state_tx.apply();

    // 6. Create a SwapClaim action
    let epoch_duration = state.get_epoch_duration_parameter().await.unwrap();
    let mut client = MockClient::new(test_keys::SPEND_KEY.clone());
    client.sync_to(1, state.deref()).await.unwrap();

    let output_data = state
        .output_data(height, trading_pair)
        .await
        .unwrap()
        .unwrap();

    let commitment = swap.body.payload.commitment;
    let swap_auth_path = client.witness_commitment(commitment).unwrap();
    let detected_plaintext = client.swap_by_commitment(&commitment).unwrap();
    assert_eq!(plaintext, detected_plaintext);

    let claim_plan = SwapClaimPlan {
        swap_plaintext: plaintext.clone(),
        position: swap_auth_path.position(),
        output_data,
        epoch_duration,
        proof_blinding_r: Fq::rand(&mut rng),
        proof_blinding_s: Fq::rand(&mut rng),
    };
    let claim = claim_plan.swap_claim(&test_keys::FULL_VIEWING_KEY, &swap_auth_path);

    // 7. Execute the SwapClaim action

    // The SwapClaim ActionHandler uses the transaction's anchor to check proofs:
    let context = &Transaction {
        anchor: client.latest_height_and_sct_root().1,
        ..Default::default()
    }
    .context();

    claim.check_stateless(context.clone()).await.unwrap();
    claim.check_historical(state.clone()).await.unwrap();
    let mut state_tx = state.try_begin_transaction().unwrap();
    state_tx.put_mock_source(2u8);
    claim.check_and_execute(&mut state_tx).await.unwrap();
    state_tx.apply();

    // 8. Now form a second SwapClaim action attempting to claim the outputs again.
    let claim_plan = SwapClaimPlan {
        swap_plaintext: plaintext,
        position: swap_auth_path.position(),
        output_data,
        epoch_duration,
        proof_blinding_r: Fq::rand(&mut rng),
        proof_blinding_s: Fq::rand(&mut rng),
    };
    let claim = claim_plan.swap_claim(&test_keys::FULL_VIEWING_KEY, &swap_auth_path);

    // 9. Execute the second SwapClaim action - the test should panic here
    claim.check_historical(state.clone()).await.unwrap();
}
 */

#[tokio::test]
async fn swap_with_nonzero_fee() -> anyhow::Result<()> {
    let mut rng = rand_chacha::ChaChaRng::seed_from_u64(1312);

    let storage = TempStorage::new_with_penumbra_prefixes()
        .await?
        .apply_default_genesis()
        .await?;
    let mut state = Arc::new(StateDelta::new(storage.latest_snapshot()));

    let height = 1;

    // 1. Simulate BeginBlock

    let mut state_tx = state.try_begin_transaction().unwrap();
    state_tx.put_block_height(height);
    state_tx.put_epoch_by_height(
        height,
        Epoch {
            index: 0,
            start_height: 0,
        },
    );
    state_tx.apply();

    // 2. Create a Swap action

    let gm = asset::Cache::with_known_assets().get_unit("gm").unwrap();
    let gn = asset::Cache::with_known_assets().get_unit("gn").unwrap();
    let trading_pair = TradingPair::new(gm.id(), gn.id());

    let delta_1 = Amount::from(100_000u64);
    let delta_2 = Amount::from(0u64);
    let fee = Fee::from_staking_token_amount(Amount::from(1u64));
    let claim_address: Address = test_keys::ADDRESS_0.deref().clone();

    let plaintext =
        SwapPlaintext::new(&mut rng, trading_pair, delta_1, delta_2, fee, claim_address);

    let swap_plan = SwapPlan::new(&mut rng, plaintext.clone());
    let swap = swap_plan.swap(&test_keys::FULL_VIEWING_KEY);

    // 3. Simulate execution of the Swap action

    swap.check_stateless(()).await?;
    swap.check_historical(state.clone()).await?;
    let mut state_tx = state.try_begin_transaction().unwrap();
    state_tx.put_mock_source(1u8);
    swap.check_and_execute(&mut state_tx).await?;
    state_tx.apply();

    // 4. Execute EndBlock (where the swap is actually executed)

    let end_block = abci::request::EndBlock {
        height: height.try_into().unwrap(),
    };
    // Execute EndBlock for the Dex, to actually execute the swaps...
    Dex::end_block(&mut state, &end_block).await;
    ShieldedPool::end_block(&mut state, &end_block).await;

    let mut state_tx = state.try_begin_transaction().unwrap();
    // ... and for the App, call `finish_block` to correctly write out the SCT with the data we'll use next.
    state_tx.finish_block().await.unwrap();

    state_tx.apply();

    Ok(())
}
