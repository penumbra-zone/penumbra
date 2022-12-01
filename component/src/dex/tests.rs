use std::sync::Arc;

use penumbra_chain::{test_keys, StateReadExt, StateWriteExt};
use penumbra_crypto::{
    asset,
    dex::{swap::SwapPlaintext, TradingPair},
    transaction::Fee,
    Address, Amount,
};
use penumbra_storage::{ArcStateExt, TempStorage};
use penumbra_transaction::{
    plan::{SwapClaimPlan, SwapPlan},
    Transaction,
};
use rand_core::SeedableRng;
use tendermint::abci;

use crate::{shielded_pool::ShieldedPool, ActionHandler, Component, TempStorageExt};

use super::{Dex, StateReadExt as _};

#[tokio::test]
async fn swap_and_swap_claim() -> anyhow::Result<()> {
    let mut rng = rand_chacha::ChaChaRng::seed_from_u64(1312);

    let storage = TempStorage::new().await?.apply_default_genesis().await?;
    let mut state = Arc::new(storage.latest_state());

    let height = 1;

    // 1. Simulate BeginBlock

    let mut state_tx = state.try_begin_transaction().unwrap();
    state_tx.put_block_height(height);
    state_tx.apply();

    // 2. Create a Swap action

    let gm = asset::REGISTRY.parse_unit("gm");
    let gn = asset::REGISTRY.parse_unit("gn");
    let trading_pair = TradingPair::new(gm.id(), gn.id()).expect("distinct assets");

    let delta_1 = Amount::from(100_000u64);
    let delta_2 = Amount::from(0u64);
    let fee = Fee::default();
    let claim_address: Address = test_keys::ADDRESS_0.clone();

    let plaintext =
        SwapPlaintext::new(&mut rng, trading_pair, delta_1, delta_2, fee, claim_address);

    let swap_plan = SwapPlan::new(&mut rng, plaintext);
    let swap = swap_plan.swap(&*test_keys::FULL_VIEWING_KEY);

    // 3. Simulate execution of the Swap action

    // The Swap ActionHandler doesn't use the context, so pass a dummy one.
    let context = Arc::new(Transaction::default());

    swap.check_stateless(context).await?;
    swap.check_stateful(state.clone()).await?;
    let mut state_tx = state.try_begin_transaction().unwrap();
    swap.execute(&mut state_tx).await?;
    state_tx.apply();

    // 4. Execute EndBlock (where the swap is actually executed)

    let end_block = abci::request::EndBlock {
        height: height.try_into().unwrap(),
    };
    let mut state_tx = state.try_begin_transaction().unwrap();
    // Execute EndBlock for the Dex, to actually execute the swaps...
    Dex::end_block(&mut state_tx, &end_block).await;
    // ... and for the ShieldedPool, to correctly write out the NCT with the data we'll use next.
    ShieldedPool::end_block(&mut state_tx, &end_block).await;
    state_tx.apply();

    // 6. Create a SwapClaim action

    /*

    // To do this, we need to have an auth path for the swap nft note,
    // which means we have

    let output_data = state.output_data(height, trading_pair).await?.unwrap();
    let epoch_duration = state.get_epoch_duration().await?;

    let claim_plan = SwapClaimPlan::new(
        &mut rng,
        plaintext,
        // swap_nft_note,
        // swap_nft_position,
        // epoch_duration (why?)
        output_data,
    );
    */

    // 7. Execute the SwapClaim action

    Ok(())
}
