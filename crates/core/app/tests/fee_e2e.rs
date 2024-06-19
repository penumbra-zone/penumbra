mod common;

use std::{ops::Deref, sync::Arc};

use self::common::TempStorageExt;
use cnidarium::{ArcStateDeltaExt as _, StateDelta, TempStorage};
use cnidarium_component::{ActionHandler, Component as _};
use penumbra_app::AppActionHandler as _;
use penumbra_asset::{asset, Value, STAKING_TOKEN_ASSET_ID};
use penumbra_compact_block::component::CompactBlockManager as _;
use penumbra_dex::{
    component::{router::FillRoute, Dex, PositionManager, PositionRead, StateWriteExt as _},
    lp::{position::Position, Reserves},
    DexParameters, DirectedTradingPair,
};
use penumbra_fee::{
    component::{FeeComponent, StateWriteExt as _},
    Fee, FeeParameters,
};
use penumbra_funding::component::Funding;
use penumbra_keys::test_keys;
use penumbra_mock_client::MockClient;
use penumbra_num::Amount;
use penumbra_sct::{
    component::{clock::EpochManager as _, sct::Sct, source::SourceContext as _},
    epoch::Epoch,
};
use penumbra_shielded_pool::{
    component::{ShieldedPool, StateWriteExt as _},
    params::ShieldedPoolParameters,
    Note, OutputPlan, SpendPlan,
};
use penumbra_tct as tct;
use penumbra_transaction::{
    plan::{CluePlan, DetectionDataPlan, TransactionPlan},
    TransactionParameters, WitnessData,
};
use penumbra_txhash::{EffectHash, TransactionContext};
use penumbra_view::Planner;
use rand::SeedableRng as _;
use rand_core::OsRng;
use tendermint::abci;
use tonic::async_trait;

#[tokio::test]
/// Submit transactions with fees (both in the staking token and in other tokens)
/// and ensure fees are swapped and accounted correctly.
async fn tx_fee_e2e() -> anyhow::Result<()> {
    let mut rng = rand_chacha::ChaChaRng::seed_from_u64(1312);

    let storage = TempStorage::new().await?.apply_default_genesis().await?;

    let mut state = Arc::new(StateDelta::new(storage.latest_snapshot()));

    let height = 1;

    // 1: a transaction paid in the staking token.

    // 1.1: Simulate BeginBlock
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

    // Precondition: This test uses the default genesis which has existing notes for the test keys.
    let mut client = MockClient::new(test_keys::SPEND_KEY.clone());
    let sk = test_keys::SPEND_KEY.clone();
    let fvk = &test_keys::FULL_VIEWING_KEY;
    client.sync_to(0, state.deref()).await?;
    let notes = client
        .notes
        .values()
        .map(|note| note.clone())
        .collect::<Vec<_>>();
    let spend_note = notes[0].clone();
    let spend_note_commitment = spend_note.commit();
    let spend_proof = client.witness_commitment(spend_note_commitment).unwrap();
    let tct_position = spend_proof.position();

    // 1.2: Create a Spend action
    // We spend a non-staking token note, and pay the fee in the staking token.
    assert_ne!(spend_note.asset_id(), *STAKING_TOKEN_ASSET_ID);
    let fee_note = notes[1].clone();
    assert_eq!(fee_note.asset_id(), *STAKING_TOKEN_ASSET_ID);
    let fee_note_commitment = fee_note.commit();
    let fee_proof = client.witness_commitment(fee_note_commitment).unwrap();
    let fee_tct_position = fee_proof.position();

    let spend_plan = SpendPlan::new(&mut rng, spend_note, tct_position);
    let fee_spend_plan = SpendPlan::new(&mut rng, fee_note, fee_tct_position);

    // 1.3: Insert the Spend action for both notes into a TransactionPlan
    let plan = TransactionPlan {
        transaction_parameters: TransactionParameters {
            expiry_height: 0,
            // Set the fee paid to 5 units of the staking token.
            fee: Fee::from_staking_token_amount(5u32.into()),
            chain_id: "".into(),
        },
        actions: vec![
            spend_plan.into(),
            fee_spend_plan.into(),
            // TODO: two output plans _should_ exist to balance tx
            // OutputPlan::new(&mut OsRng, value, test_keys::ADDRESS_1.deref().clone()).into(),
        ],
        detection_data: Some(DetectionDataPlan {
            clue_plans: vec![CluePlan::new(
                &mut OsRng,
                test_keys::ADDRESS_1.deref().clone(),
                1.try_into().unwrap(),
            )],
        }),
        memo: None,
    };

    // 1.4: Build the transaction.
    let auth_data = plan.authorize(OsRng, &sk)?;
    let witness_data = client.witness_plan(&plan)?;
    let tx = plan
        .build_concurrent(fvk, &witness_data, &auth_data)
        .await
        .expect("can build transaction");

    // 1.5: Simulate execution of the transaction
    tx.check_stateless(()).await?;
    tx.check_historical(state.clone()).await?;
    let mut state_tx = state.try_begin_transaction().unwrap();
    state_tx.put_mock_source(1u8);
    tx.check_and_execute(&mut state_tx).await?;
    state_tx.apply();

    // 1.6: Execute EndBlock for the relevant components
    let end_block = abci::request::EndBlock {
        height: height.try_into().unwrap(),
    };

    let state_tx = StateDelta::new(state.clone());

    tracing::debug!("running app components' `end_block` hooks");
    let mut arc_state_tx = Arc::new(state_tx);

    Sct::end_block(&mut arc_state_tx, &end_block).await;
    ShieldedPool::end_block(&mut arc_state_tx, &end_block).await;
    Dex::end_block(&mut arc_state_tx, &end_block).await;
    FeeComponent::end_block(&mut arc_state_tx, &end_block).await;
    Funding::end_block(&mut arc_state_tx, &end_block).await;

    let mut state_tx =
        Arc::try_unwrap(arc_state_tx).expect("components did not retain copies of shared state");

    // ... and for the App, call `finish_block` to correctly write out the SCT with the data we'll use next.
    state_tx.finish_block().await.unwrap();

    let (state2, cache) = state_tx.flatten();
    std::mem::drop(state2);
    // Now there is only one reference to the inter-block state: state

    cache.apply_to(Arc::get_mut(&mut state).expect("no other references to inter-block state"));

    // // Generate two notes controlled by the test address.
    // let value = Value {
    //     amount: 100u64.into(),
    //     asset_id: *STAKING_TOKEN_ASSET_ID,
    // };
    // let note = Note::generate(&mut OsRng, &test_keys::ADDRESS_0, value);
    // let value2 = Value {
    //     amount: 50u64.into(),
    //     asset_id: *STAKING_TOKEN_ASSET_ID,
    // };
    // let note2 = Note::generate(&mut OsRng, &test_keys::ADDRESS_0, value2);

    // // Record that note in an SCT, where we can generate an auth path.
    // let mut sct = tct::Tree::new();
    // // Assume there's a bunch of stuff already in the SCT.
    // for _ in 0..5 {
    //     let random_note = Note::generate(&mut OsRng, &test_keys::ADDRESS_0, value);
    //     sct.insert(tct::Witness::Keep, random_note.commit())
    //         .unwrap();
    // }
    // sct.insert(tct::Witness::Keep, note.commit()).unwrap();
    // sct.insert(tct::Witness::Keep, note2.commit()).unwrap();
    // // Do we want to seal the SCT block here?
    // let auth_path = sct.witness(note.commit()).unwrap();
    // let auth_path2 = sct.witness(note2.commit()).unwrap();

    // // Add a single spend and output to the transaction plan such that the
    // // transaction balances.
    // let plan = TransactionPlan {
    //     transaction_parameters: TransactionParameters {
    //         expiry_height: 0,
    //         fee: Fee::default(),
    //         chain_id: "".into(),
    //     },
    //     actions: vec![
    //         SpendPlan::new(&mut OsRng, note, auth_path.position()).into(),
    //         SpendPlan::new(&mut OsRng, note2, auth_path2.position()).into(),
    //         OutputPlan::new(&mut OsRng, value, test_keys::ADDRESS_1.deref().clone()).into(),
    //     ],
    //     detection_data: Some(DetectionDataPlan {
    //         clue_plans: vec![CluePlan::new(
    //             &mut OsRng,
    //             test_keys::ADDRESS_1.deref().clone(),
    //             1.try_into().unwrap(),
    //         )],
    //     }),
    //     memo: None,
    // };

    // // Build the transaction.
    // let sk = &test_keys::SPEND_KEY;
    // let auth_data = plan.authorize(OsRng, sk)?;
    // let witness_data = WitnessData {
    //     anchor: sct.root(),
    //     state_commitment_proofs: plan
    //         .spend_plans()
    //         .map(|spend| {
    //             (
    //                 spend.note.commit(),
    //                 sct.witness(spend.note.commit()).unwrap(),
    //             )
    //         })
    //         .collect(),
    // };
    // let tx = plan
    //     .build_concurrent(fvk, &witness_data, &auth_data)
    //     .await
    //     .expect("can build transaction");

    // let context = tx.context();

    // // On the verifier side, perform stateless verification.
    // for action in tx.transaction_body().actions {
    //     let result = action.check_stateless(context.clone()).await;
    //     assert!(result.is_ok())
    // }

    // // Without opening liquidity positions, we can't swap anything.
    // // Fees paid in non-native assets will be burned.

    // let gm = asset::Cache::with_known_assets().get_unit("gm").unwrap();
    // let gn = asset::Cache::with_known_assets().get_unit("gn").unwrap();

    // let pair = DirectedTradingPair::new(gm.id(), gn.id());

    // /* position_1: Limit Buy 100gm@1.2gn */
    // let reserves = Reserves {
    //     r1: 0u64.into(),
    //     r2: 120_000u64.into(),
    // };

    // let position_1 = Position::new(
    //     OsRng,
    //     pair,
    //     0u32,
    //     1_200_000u64.into(),
    //     1_000_000u64.into(),
    //     reserves,
    // );

    // let mut state_tx: StateDelta<&mut StateDelta<cnidarium::Snapshot>> =
    //     state.try_begin_transaction().unwrap();
    // let position_1_id = position_1.id();
    // state_tx.open_position(position_1.clone()).await.unwrap();

    // let mut state_test_1 = state_tx.fork();

    // // A single limit order quotes asset 2
    // // We execute four swaps against this order and verify that reserves are accurately updated,
    // // and that filled amounts all checkout.
    // //      Test 1:
    // //          * swap_1: fills the entire order
    // //          -> reserves are updated correctly
    // //      Test 2:
    // //          * swap_1: fills entire order
    // //          * swap_2: fills entire order (other direction)
    // //          -> reserves are updated correctly
    // //      Test 3:
    // //          * swap_1: partial fill
    // //          -> reserves are updated correctly
    // //          * swap_2: clone of swap_1
    // //          -> reserves are updated correctly
    // //          * swap_3: fills what is left
    // //          -> reserves are updated correctly

    // // Test 1: we exhaust the entire position.
    // // In theory, we would only need to swap 100_000gm, to get back
    // // 120_000gn, with no unfilled amount and resulting reserves of 100_000gm and 0gn.
    // // However, since we have to account for rounding, we need to swap 100_001gm which
    // // gets us back 120_000gn. The unfilled amount is 1gm, and the resulting reserves are
    // // 100_000gm and 0gn.
    // let delta_gm = Value {
    //     amount: 100_001u64.into(),
    //     asset_id: gm.id(),
    // };

    // let expected_lambda_gm = Value {
    //     amount: 1u64.into(),
    //     asset_id: gm.id(),
    // };

    // let expected_lambda_gn = Value {
    //     amount: 120_000u64.into(),
    //     asset_id: gn.id(),
    // };

    // let route_to_gn = vec![gn.id()];
    // let execution = FillRoute::fill_route(&mut state_test_1, delta_gm, &route_to_gn, None).await?;

    // let unfilled = delta_gm
    //     .amount
    //     .checked_sub(&execution.input.amount)
    //     .unwrap();
    // let output = execution.output;

    // assert_eq!(unfilled, expected_lambda_gm.amount);
    // assert_eq!(output, expected_lambda_gn);

    // let position = state_test_1
    //     .position_by_id(&position_1_id)
    //     .await
    //     .unwrap()
    //     .unwrap();

    // // Check that the position is filled
    // assert_eq!(position.reserves.r2, Amount::zero());
    // // We got back 1gm of unfilled amount
    // let expected_r1 = delta_gm.amount.checked_sub(&unfilled).unwrap();
    // assert_eq!(position.reserves.r1, expected_r1);

    // let route_to_gm = vec![gm.id()];
    // // Now we try to fill the order a second time, this should leave the position untouched.
    // assert!(state_test_1
    //     .fill_route(delta_gm, &route_to_gn, None)
    //     .await
    //     .is_err());

    // // Fetch the position, and assert that its reserves are unchanged.
    // let position = state_test_1
    //     .position_by_id(&position_1_id)
    //     .await
    //     .unwrap()
    //     .unwrap();
    // assert_eq!(position.reserves.r1, expected_r1);
    // assert_eq!(position.reserves.r2, Amount::zero());

    // // Now let's try to do a "round-trip" i.e. fill the order in the other direction:
    // let delta_gn = Value {
    //     amount: 120_001u64.into(),
    //     asset_id: gn.id(),
    // };

    // let execution = state_test_1
    //     .fill_route(delta_gn, &route_to_gm, None)
    //     .await?;

    // let unfilled = delta_gn
    //     .amount
    //     .checked_sub(&execution.input.amount)
    //     .unwrap();
    // let output = execution.output;

    // assert_eq!(unfilled, Amount::from(1u64));
    // assert_eq!(
    //     output,
    //     Value {
    //         amount: 100_000u64.into(),
    //         asset_id: gm.id(),
    //     }
    // );

    // let position = state_test_1
    //     .position_by_id(&position_1_id)
    //     .await
    //     .unwrap()
    //     .unwrap();

    // assert_eq!(position.reserves.r1, Amount::zero());
    // assert_eq!(position.reserves.r2, 120_000u64.into());

    // // Finally, let's test partial fills, rolling back to `state_tx`, which contains
    // // a single limit order for 100_000gm@1.2gn.
    // for _i in 1..=100 {
    //     let delta_gm = Value {
    //         amount: 1000u64.into(),
    //         asset_id: gm.id(),
    //     };
    //     // We are splitting a single large fill for a `100_000gm` into, 100 fills for `1000gm`.
    //     let execution = state_tx.fill_route(delta_gm, &route_to_gn, None).await?;

    //     let unfilled = delta_gm
    //         .amount
    //         .checked_sub(&execution.input.amount)
    //         .unwrap();
    //     let output = execution.output;

    //     // We check that there are no unfilled `gm`s resulting from executing the order
    //     assert_eq!(unfilled, Amount::from(0u64));
    //     // And that for every `1000gm`, we get `1199gn`.
    //     assert_eq!(
    //         output,
    //         Value {
    //             amount: 1199u64.into(),
    //             asset_id: gn.id(),
    //         }
    //     );
    // }

    // // After executing 100 swaps of `1000gm` into `gn`. We should have acquired `1199gn*100` or `119900gn`.
    // // We consume the last `100gn` in the next swap.
    // let delta_gm = Value {
    //     amount: 84u64.into(),
    //     asset_id: gm.id(),
    // };
    // let execution = state_tx.fill_route(delta_gm, &route_to_gn, None).await?;
    // let unfilled = delta_gm
    //     .amount
    //     .checked_sub(&execution.input.amount)
    //     .unwrap();
    // let output = execution.output;
    // assert_eq!(unfilled, Amount::from(0u64));
    // assert_eq!(
    //     output,
    //     Value {
    //         amount: 100u64.into(),
    //         asset_id: gn.id(),
    //     }
    // );

    // // Now, we want to verify that the position is updated with the correct reserves.
    // // We should have depleted the order of all its `gn`s, and replaced it with `100_084gm`
    // // This accounts for the rounding behavior that arises when swapping smaller quantities.
    // let position = state_tx
    //     .position_by_id(&position_1_id)
    //     .await
    //     .unwrap()
    //     .unwrap();

    // assert_eq!(position.reserves.r1, 100_084u64.into());
    // assert_eq!(position.reserves.r2, Amount::zero());

    Ok(())
}
