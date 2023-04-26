use std::{ops::Deref, sync::Arc};

use penumbra_chain::{test_keys, StateReadExt, StateWriteExt};
use penumbra_crypto::{
    asset,
    dex::{swap::SwapPlaintext, TradingPair},
    transaction::Fee,
    Address, Amount,
};
use penumbra_storage::{ArcStateDeltaExt, StateDelta, TempStorage};
use penumbra_transaction::{
    plan::{SwapClaimPlan, SwapPlan},
    Transaction,
};
use rand_core::SeedableRng;
use tendermint::abci;

use crate::app::App;
use crate::{shielded_pool::ShieldedPool, ActionHandler, Component, MockClient, TempStorageExt};

use super::{StateReadExt as _, StubDex};

#[tokio::test]
async fn swap_and_swap_claim() -> anyhow::Result<()> {
    let mut rng = rand_chacha::ChaChaRng::seed_from_u64(1312);

    let storage = TempStorage::new().await?.apply_default_genesis().await?;
    let mut state = Arc::new(StateDelta::new(storage.latest_snapshot()));

    let height = 1;

    // 1. Simulate BeginBlock

    let mut state_tx = state.try_begin_transaction().unwrap();
    state_tx.put_epoch_by_height(
        height,
        penumbra_chain::Epoch {
            index: 0,
            start_height: 0,
        },
    );
    state_tx.put_block_height(height);
    state_tx.apply();

    // 2. Create a Swap action

    let gm = asset::REGISTRY.parse_unit("gm");
    let gn = asset::REGISTRY.parse_unit("gn");
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

    // We don't use the context in the Swap::check_stateless impl, so use a dummy one.
    let dummy_context = Arc::new(Transaction::default());
    swap.check_stateless(dummy_context.clone()).await?;
    swap.check_stateful(state.clone()).await?;
    let mut state_tx = state.try_begin_transaction().unwrap();
    swap.execute(&mut state_tx).await?;
    state_tx.apply();

    // 4. Execute EndBlock (where the swap is actually executed)

    let end_block = abci::request::EndBlock {
        height: height.try_into().unwrap(),
    };
    // Execute EndBlock for the Dex, to actually execute the swaps...
    StubDex::end_block(&mut state, &end_block).await;
    ShieldedPool::end_block(&mut state, &end_block).await;
    let mut state_tx = state.try_begin_transaction().unwrap();
    // ... and for the App, to correctly write out the SCT with the data we'll use next.
    App::finish_sct_block(&mut state_tx).await;

    state_tx.apply();

    // 6. Create a SwapClaim action

    // To do this, we need to have an auth path for the swap nft note, which
    // means we have to synchronize a client's view of the test chain's SCT
    // state.

    let epoch_duration = state.get_epoch_duration().await?;
    let mut client = MockClient::new(test_keys::FULL_VIEWING_KEY.clone());
    // TODO: generalize StateRead/StateWrite impls from impl for &S to impl for Deref<Target=S>
    client.sync_to(1, state.deref()).await?;

    let output_data = state.output_data(height, trading_pair).await?.unwrap();

    let commitment = swap.body.payload.commitment;
    let swap_auth_path = client.witness(commitment).unwrap();
    let detected_plaintext = client.swap_by_commitment(&commitment).unwrap();
    assert_eq!(plaintext, detected_plaintext);

    let claim_plan = SwapClaimPlan {
        swap_plaintext: plaintext,
        position: swap_auth_path.position(),
        output_data,
        epoch_duration,
    };
    let claim = claim_plan.swap_claim(&test_keys::FULL_VIEWING_KEY, &swap_auth_path);

    // 7. Execute the SwapClaim action

    // The SwapClaim ActionHandler uses the transaction's anchor to check proofs:
    let context = Arc::new(Transaction {
        anchor: client.latest_height_and_sct_root().1,
        ..Default::default()
    });

    claim.check_stateless(context).await?;
    claim.check_stateful(state.clone()).await?;
    let mut state_tx = state.try_begin_transaction().unwrap();
    claim.execute(&mut state_tx).await?;
    state_tx.apply();

    Ok(())
}

#[tokio::test]
async fn swap_with_nonzero_fee() -> anyhow::Result<()> {
    let mut rng = rand_chacha::ChaChaRng::seed_from_u64(1312);

    let storage = TempStorage::new().await?.apply_default_genesis().await?;
    let mut state = Arc::new(StateDelta::new(storage.latest_snapshot()));

    let height = 1;

    // 1. Simulate BeginBlock

    let mut state_tx = state.try_begin_transaction().unwrap();
    state_tx.put_block_height(height);
    state_tx.put_epoch_by_height(
        height,
        penumbra_chain::Epoch {
            index: 0,
            start_height: 0,
        },
    );
    state_tx.apply();

    // 2. Create a Swap action

    let gm = asset::REGISTRY.parse_unit("gm");
    let gn = asset::REGISTRY.parse_unit("gn");
    let trading_pair = TradingPair::new(gm.id(), gn.id());

    let delta_1 = Amount::from(100_000u64);
    let delta_2 = Amount::from(0u64);
    let fee = Fee::from_staking_token_amount(Amount::from(1u64));
    let claim_address: Address = *test_keys::ADDRESS_0;

    let plaintext =
        SwapPlaintext::new(&mut rng, trading_pair, delta_1, delta_2, fee, claim_address);

    let swap_plan = SwapPlan::new(&mut rng, plaintext.clone());
    let swap = swap_plan.swap(&test_keys::FULL_VIEWING_KEY);

    // 3. Simulate execution of the Swap action

    // We don't use the context in the Swap::check_stateless impl, so use a dummy one.
    let dummy_context = Arc::new(Transaction::default());
    swap.check_stateless(dummy_context.clone()).await?;
    swap.check_stateful(state.clone()).await?;
    let mut state_tx = state.try_begin_transaction().unwrap();
    swap.execute(&mut state_tx).await?;
    state_tx.apply();

    // 4. Execute EndBlock (where the swap is actually executed)

    let end_block = abci::request::EndBlock {
        height: height.try_into().unwrap(),
    };
    // Execute EndBlock for the Dex, to actually execute the swaps...
    StubDex::end_block(&mut state, &end_block).await;
    ShieldedPool::end_block(&mut state, &end_block).await;
    let mut state_tx = state.try_begin_transaction().unwrap();
    App::finish_sct_block(&mut state_tx).await;
    state_tx.apply();

    Ok(())
}
