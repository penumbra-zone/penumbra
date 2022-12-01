use std::{ops::Deref, sync::Arc};

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
    Action, Transaction, TransactionBody,
};
use rand_core::SeedableRng;
use tendermint::abci;

use crate::{shielded_pool::ShieldedPool, ActionHandler, Component, MockClient, TempStorageExt};

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

    let swap_plan = SwapPlan::new(&mut rng, plaintext.clone());
    let swap = swap_plan.swap(&*test_keys::FULL_VIEWING_KEY);

    // 3. Simulate execution of the Swap action

    // TODO: this is slightly cursed: although exection of the Swap action is
    // supposed to happen in its ActionHandler, just calling the swap.execute()
    // won't cause us to actually record its note payload, because the logic
    // that records the note payload (for any action) is in the transaction-wide
    // ActionHandler impl.  Why? because with the current quarantine system, we
    // have to quarantine all outputs from any action if a quarantined output is
    // present, so we can't do the correct and simple thing of executing each
    // action in its own action handler. After tokenized unbonding, we should
    // fix this!
    //
    // Because of this, we have to construct a whole dummy tx that we can
    // .execute().  Otherwise, we won't actually record the swap nft for use in
    // testing the SwapClaim logic.

    let swap_transaction = Arc::new(Transaction {
        transaction_body: TransactionBody {
            actions: vec![Action::Swap(swap.clone())],
            ..Default::default()
        },
        ..Default::default()
    });

    swap.check_stateless(swap_transaction.clone()).await?;
    swap.check_stateful(state.clone()).await?;
    let mut state_tx = state.try_begin_transaction().unwrap();
    // This will call swap.execute()
    swap_transaction.execute(&mut state_tx).await?;
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

    // To do this, we need to have an auth path for the swap nft note, which
    // means we have to synchronize a client's view of the test chain's NCT
    // state.

    let epoch_duration = state.get_epoch_duration().await?;
    let mut client = MockClient::new(test_keys::FULL_VIEWING_KEY.clone(), epoch_duration);
    // TODO: generalize StateRead/StateWrite impls from impl for &S to impl for Deref<Target=S>
    client.sync_to(1, state.deref()).await?;

    let output_data = state.output_data(height, trading_pair).await?.unwrap();

    let swap_nft_note = client
        .note_by_commitment(&swap.body.swap_nft.note_commitment)
        .expect("client should have detected the swap nft note");

    let swap_nft_note_auth_path = client
        .witness(swap.body.swap_nft.note_commitment)
        .expect("client should have detected the swap nft note");

    let claim_plan = SwapClaimPlan::new(
        &mut rng,
        plaintext,
        swap_nft_note,
        swap_nft_note_auth_path.position(),
        epoch_duration,
        output_data,
    );
    let claim = claim_plan.swap_claim(&*test_keys::FULL_VIEWING_KEY, &swap_nft_note_auth_path);

    // 7. Execute the SwapClaim action

    // The SwapClaim ActionHandler uses the transaction's anchor to check proofs:
    let context = Arc::new(Transaction {
        anchor: client.latest_height_and_nct_root().1,
        ..Default::default()
    });

    claim.check_stateless(context).await?;
    claim.check_stateful(state.clone()).await?;
    let mut state_tx = state.try_begin_transaction().unwrap();
    claim.execute(&mut state_tx).await?;
    state_tx.apply();

    Ok(())
}
