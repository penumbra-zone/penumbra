use anyhow::Ok;
use futures::StreamExt;
use penumbra_crypto::{
    asset::{self},
    dex::{
        lp::{position::Position, Reserves},
        DirectedTradingPair, Market,
    },
    Amount, MockFlowCiphertext,
};
use penumbra_proto::core::dex::v1alpha1::swap_execution;
use penumbra_storage::{ArcStateDeltaExt, StateDelta, TempStorage};

use rand_core::OsRng;

use penumbra_crypto::Value;

use crate::dex::{
    position_manager::PositionManager,
    router::{limit_buy, RouteAndFill},
    StateWriteExt,
};
use crate::dex::{position_manager::PositionRead, StateReadExt};
use crate::TempStorageExt;
use std::sync::Arc;

#[tokio::test]
/// Builds a simple order book with a single limit order, and tests different
/// market order execution against it.
async fn single_limit_order() -> anyhow::Result<()> {
    let storage = TempStorage::new().await?.apply_default_genesis().await?;
    let mut state = Arc::new(StateDelta::new(storage.latest_snapshot()));
    let mut state_tx = state.try_begin_transaction().unwrap();

    let gm = asset::REGISTRY.parse_unit("gm");
    let gn = asset::REGISTRY.parse_unit("gn");

    let pair = DirectedTradingPair::new(gm.id(), gn.id());

    /* position_1: Limit Buy 100gm@1.2gn */
    let reserves = Reserves {
        r1: 0u64.into(),
        r2: 120_000u64.into(),
    };

    let position_1 = Position::new(
        OsRng,
        pair,
        0u32,
        1_200_000u64.into(),
        1_000_000u64.into(),
        reserves,
    );

    let position_1_id = position_1.id();
    state_tx.put_position(position_1.clone());

    let mut state_test_1 = state_tx.fork();

    // A single limit order quotes asset 2
    // We execute four swaps against this order and verify that reserves are accurately updated,
    // and that filled amounts all checkout.
    //      Test 1:
    //          * swap_1: fills the entire order
    //          -> reserves are updated correctly
    //      Test 2:
    //          * swap_1: fills entire order
    //          * swap_2: fills entire order (other direction)
    //          -> reserves are updated correctly
    //      Test 3:
    //          * swap_1: partial fill
    //          -> reserves are updated correctly
    //          * swap_2: clone of swap_1
    //          -> reserves are updated correctly
    //          * swap_3: fills what is left
    //          -> reserves are updated correctly

    // Test 1: We're trying to fill the entire order.
    let delta_1 = Value {
        amount: 100_000u64.into(),
        asset_id: gm.id(),
    };

    let lambda_1 = Value {
        amount: 0u64.into(),
        asset_id: gm.id(),
    };

    let lambda_2 = Value {
        amount: 120_000u64.into(),
        asset_id: gn.id(),
    };

    let (unfilled, output) = state_test_1.fill_against(delta_1, &position_1_id).await?;
    assert_eq!(unfilled, lambda_1);
    assert_eq!(output, lambda_2);

    let position = state_test_1
        .position_by_id(&position_1_id)
        .await
        .unwrap()
        .unwrap();

    // Check that the position is filled
    assert_eq!(position.reserves.r1, delta_1.amount);
    assert_eq!(position.reserves.r2, Amount::zero());

    // Now we try to fill the order a second time, this should leave the position untouched.
    let (unfilled, output) = state_test_1.fill_against(delta_1, &position_1_id).await?;
    assert_eq!(unfilled, delta_1);
    assert_eq!(
        output,
        Value {
            amount: Amount::zero(),
            asset_id: gn.id(),
        }
    );

    // Fetch the position, and assert that its reserves are unchanged.
    let position = state_test_1
        .position_by_id(&position_1_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(position.reserves.r1, delta_1.amount);
    assert_eq!(position.reserves.r2, Amount::zero());

    // Now let's try to do a "round-trip" i.e. fill the order in the other direction:
    let delta_2 = Value {
        amount: 120_000u64.into(),
        asset_id: gn.id(),
    };
    let (unfilled, output) = state_test_1.fill_against(delta_2, &position_1_id).await?;
    assert_eq!(
        unfilled,
        Value {
            amount: Amount::zero(),
            asset_id: gn.id(),
        }
    );
    assert_eq!(
        output,
        Value {
            amount: 100_000u64.into(),
            asset_id: gm.id(),
        }
    );

    let position = state_test_1
        .position_by_id(&position_1_id)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(position.reserves.r1, Amount::zero());
    assert_eq!(position.reserves.r2, 120_000u64.into());

    // Finally, let's test partial fills, rolling back to `state_tx`, which contains
    // a single limit order for 100_000gm@1.2gn.
    for _i in 1..=100 {
        let delta_1 = Value {
            amount: 1000u64.into(),
            asset_id: gm.id(),
        };
        // We are splitting a single large fill for a `100_000gm` into, 100 fills for `1000gm`.
        let (unfilled, output) = state_tx.fill_against(delta_1, &position_1_id).await?;

        // We check that there are no unfilled `gm`s resulting from executing the order
        assert_eq!(
            unfilled,
            Value {
                amount: Amount::zero(),
                asset_id: gm.id(),
            }
        );
        // And that for every `1000gm`, we get `1200gn` as desired.
        assert_eq!(
            output,
            Value {
                amount: 1200u64.into(),
                asset_id: gn.id(),
            }
        );
    }

    // Now, we want to verify that the position is updated with the correct reserves.
    // We should have depleted the order of all its `gn`s, and replaced it with `100_000gm`.
    let position = state_tx
        .position_by_id(&position_1_id)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(position.reserves.r1, 100_000u64.into());
    assert_eq!(position.reserves.r2, Amount::zero());

    Ok(())
}

#[tokio::test]
/// Try to execute against multiple positions, mainly testing that the order-book traversal
/// is done correctly.
async fn multiple_limit_orders() -> anyhow::Result<()> {
    let storage = TempStorage::new().await?.apply_default_genesis().await?;
    let mut state = Arc::new(StateDelta::new(storage.latest_snapshot()));
    let mut state_tx = state.try_begin_transaction().unwrap();

    let gm = asset::REGISTRY.parse_unit("gm");
    let gn = asset::REGISTRY.parse_unit("gn");

    let pair = DirectedTradingPair::new(gm.id(), gn.id());

    /*

    Setup 1:
        We post three identical orders (buy 100gm@1.2gn), with different fees.
        Order A: 9 bps fee
        Order B: 10 bps fee
        Order C: 11 bps fee

    We are first going to check that we can exhaust Order A, while leaving B and C intact.

    Then, we want to try to fill A and B. And finally, all three orders, ensuring that execution
    is well-ordered.
    */

    let reserves_1 = Reserves {
        r1: 0u64.into(),
        r2: 120_000u64.into(),
    };
    let reserves_2 = reserves_1.clone();
    let reserves_3 = reserves_1.clone();

    // Building positions:
    let position_1 = Position::new(
        OsRng,
        pair,
        9u32,
        1_200_000u64.into(),
        1_000_000u64.into(),
        reserves_1,
    );

    // Order B's trading function, with a 10 bps fee.
    let position_2 = Position::new(
        OsRng,
        pair,
        10u32,
        1_200_000u64.into(),
        1_000_000u64.into(),
        reserves_2,
    );

    // Order C's trading function with an 11 bps fee
    let position_3 = Position::new(
        OsRng,
        pair,
        11u32,
        1_200_000u64.into(),
        1_000_000u64.into(),
        reserves_3,
    );

    let position_1_id = position_1.id();
    let position_2_id = position_2.id();
    let position_3_id = position_3.id();

    // The insertion order shouldn't matter.
    state_tx.put_position(position_2.clone());
    state_tx.put_position(position_1.clone());
    state_tx.put_position(position_3.clone());

    let mut full_orderbook_state = state_tx;

    let mut state_test_1 = full_orderbook_state.fork();

    // Since we are looking to exhaust the order's reserves, Delta_1's amount
    // must account for the 9 bps fee, rounded up.
    let delta_1 = Value {
        amount: 100_091u64.into(),
        asset_id: gm.id(),
    };

    let (unfilled, output, positions_touched) = state_test_1
        .fill(delta_1, DirectedTradingPair::new(gm.id(), gn.id()))
        .await?;

    assert_eq!(positions_touched.len(), 1);
    assert_eq!(positions_touched[0], position_1_id);

    assert_eq!(unfilled.amount, Amount::zero());
    assert_eq!(output.amount, 120_000u64.into());

    // We fetch the entire order book, checking that only position 1 was filled against.
    let p_1 = state_test_1.position_by_id(&position_1_id).await?.unwrap();
    assert_eq!(p_1.reserves.r1, 100_091u64.into());
    assert_eq!(p_1.reserves.r2, Amount::zero());
    let p_2 = state_test_1.position_by_id(&position_2_id).await?.unwrap();
    assert_eq!(p_2.reserves.r1, Amount::zero());
    assert_eq!(p_2.reserves.r2, 120_000u64.into());
    let p_3 = state_test_1.position_by_id(&position_3_id).await?.unwrap();
    assert_eq!(p_3.reserves.r1, Amount::zero());
    assert_eq!(p_3.reserves.r2, 120_000u64.into());

    // Test 2: We're trying to exhaust order A and B:
    let mut state_test_2 = full_orderbook_state.fork();
    let delta_1 = Value {
        amount: delta_1.amount + 100_101u64.into(),
        asset_id: gm.id(),
    };

    let (unfilled, output, positions_touched) = state_test_2
        .fill(delta_1, DirectedTradingPair::new(gm.id(), gn.id()))
        .await?;

    assert_eq!(positions_touched.len(), 2);
    assert_eq!(positions_touched[0], position_1_id);
    assert_eq!(positions_touched[1], position_2_id);

    assert_eq!(unfilled.amount, Amount::zero());
    assert_eq!(output.amount, 240_000u64.into());

    // We fetch the entire order book, checking that only position 1 was filled against.
    let p_1 = state_test_2.position_by_id(&position_1_id).await?.unwrap();
    assert_eq!(p_1.reserves.r1, 100_091u64.into());
    assert_eq!(p_1.reserves.r2, Amount::zero());
    let p_2 = state_test_2.position_by_id(&position_2_id).await?.unwrap();
    assert_eq!(p_2.reserves.r1, 100_101u64.into());
    assert_eq!(p_2.reserves.r2, 0u64.into());
    let p_3 = state_test_2.position_by_id(&position_3_id).await?.unwrap();
    assert_eq!(p_3.reserves.r1, Amount::zero());
    assert_eq!(p_3.reserves.r2, 120_000u64.into());

    // Test 3: We're trying to fill all the orders.
    let mut state_test_3 = full_orderbook_state.fork();
    let delta_1 = Value {
        amount: delta_1.amount + 100_111u64.into(),
        asset_id: gm.id(),
    };

    let (unfilled, output, positions_touched) = state_test_3
        .fill(delta_1, DirectedTradingPair::new(gm.id(), gn.id()))
        .await?;

    assert_eq!(positions_touched.len(), 3);
    assert_eq!(positions_touched[0], position_1_id);
    assert_eq!(positions_touched[1], position_2_id);
    assert_eq!(positions_touched[2], position_3_id);

    assert_eq!(unfilled.amount, Amount::zero());
    assert_eq!(output.amount, 360_000u64.into());

    // We fetch the entire order book, checking that only position 1 was filled against.
    let p_1 = state_test_3.position_by_id(&position_1_id).await?.unwrap();
    assert_eq!(p_1.reserves.r1, 100_091u64.into());
    assert_eq!(p_1.reserves.r2, Amount::zero());
    let p_2 = state_test_3.position_by_id(&position_2_id).await?.unwrap();
    assert_eq!(p_2.reserves.r1, 100_101u64.into());
    assert_eq!(p_2.reserves.r2, Amount::zero());
    let p_3 = state_test_3.position_by_id(&position_3_id).await?.unwrap();
    assert_eq!(p_3.reserves.r1, 100_111u64.into());
    assert_eq!(p_3.reserves.r2, Amount::zero());

    Ok(())
}

#[tokio::test]
/// Test that submitting a position that provisions no inventory fails.
async fn empty_order_fails() -> anyhow::Result<()> {
    use crate::ActionHandler;
    let gm = asset::REGISTRY.parse_unit("gm");
    let gn = asset::REGISTRY.parse_unit("gn");

    let pair = DirectedTradingPair::new(gm.id(), gn.id());

    let reserves = Reserves {
        r1: 0u64.into(),
        r2: 0u64.into(),
    };
    let position_1 = Position::new(
        OsRng,
        pair,
        0u32,
        1_200_000u64.into(),
        1_000_000u64.into(),
        reserves,
    );

    let position_action = penumbra_transaction::action::PositionOpen {
        position: position_1,
    };

    assert!(position_action.check_stateless(()).await.is_err());

    Ok(())
}

#[tokio::test]
/// Test that positions are created and returned as expected.
async fn position_create_and_retrieve() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt::try_init();
    let storage = TempStorage::new().await?.apply_default_genesis().await?;
    let mut state = Arc::new(StateDelta::new(storage.latest_snapshot()));
    let mut state_tx = state.try_begin_transaction().unwrap();

    let gm = asset::REGISTRY.parse_unit("gm");
    let gn = asset::REGISTRY.parse_unit("gn");

    let price1: Amount = 1u64.into();
    let buy_1 = Position::new(
        OsRng,
        DirectedTradingPair {
            start: gm.clone().id(),
            end: gn.clone().id(),
        },
        0u32,
        price1 * gn.clone().unit_amount(),
        Amount::from(1u64) * gm.clone().unit_amount(),
        Reserves {
            r1: Amount::zero(),
            r2: Amount::from(1u64) * price1 * gn.clone().unit_amount(),
        },
    );
    state_tx.put_position(buy_1.clone());
    state_tx.apply();

    let stream = state.all_positions();
    let all_positions = stream.map(|p| p.unwrap()).collect::<Vec<_>>().await;
    assert!(all_positions.len() == 1);
    assert!(all_positions[0].id() == buy_1.id());

    let mut state_tx = state.try_begin_transaction().unwrap();

    let price2: Amount = 2u64.into();
    let buy_2 = Position::new(
        OsRng,
        DirectedTradingPair {
            start: gm.clone().id(),
            end: gn.clone().id(),
        },
        0u32,
        price2 * gn.clone().unit_amount(),
        Amount::from(1u64) * gm.clone().unit_amount(),
        Reserves {
            r1: Amount::zero(),
            r2: Amount::from(1u64) * price2 * gn.clone().unit_amount(),
        },
    );
    state_tx.put_position(buy_2.clone());
    state_tx.apply();

    let stream = state.all_positions();
    let all_positions = stream.map(|p| p.unwrap()).collect::<Vec<_>>().await;
    assert!(all_positions.len() == 2);
    assert!(all_positions
        .iter()
        .map(|p| p.id())
        .collect::<Vec<_>>()
        .contains(&buy_1.id()));
    assert!(all_positions
        .iter()
        .map(|p| p.id())
        .collect::<Vec<_>>()
        .contains(&buy_2.id()));

    Ok(())
}

#[tokio::test]
/// Test that swap executions are created and recorded as expected.
async fn swap_execution_tests() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt::try_init();
    let storage = TempStorage::new().await?.apply_default_genesis().await?;
    let mut state = Arc::new(StateDelta::new(storage.latest_snapshot()));
    let mut state_tx = state.try_begin_transaction().unwrap();

    let gn = asset::REGISTRY.parse_unit("gn");
    let penumbra = asset::REGISTRY.parse_unit("penumbra");

    let pair_gn_penumbra = Market::new(gn.clone(), penumbra.clone());
    let pair_penumbra_gn = Market::new(gn.clone(), penumbra.clone());

    // Create a single 1:1 gn:penumbra position (i.e. buy 1 gn at 1 penumbra).
    let buy_1 = limit_buy(pair_gn_penumbra.clone(), 1u64.into(), 1u64.into());
    state_tx.put_position(buy_1);
    state_tx.apply();

    // Now we should be able to fill a 1:1 gn:penumbra swap.
    let trading_pair = pair_gn_penumbra.into_directed_trading_pair().into();

    let mut swap_flow = state.swap_flow(&trading_pair);

    assert!(trading_pair.asset_1() == penumbra.id());

    // Add the amount of each asset being swapped to the batch swap flow.
    swap_flow.0 += MockFlowCiphertext::new(0u32.into());
    swap_flow.1 += MockFlowCiphertext::new(1u32.into());

    // Set the batch swap flow for the trading pair.
    Arc::get_mut(&mut state)
        .unwrap()
        .put_swap_flow(&trading_pair, swap_flow.clone());
    state
        .handle_batch_swaps(trading_pair, swap_flow, 0u32.into(), 0)
        .await
        .expect("unable to process batch swaps");

    // Swap execution should have a single trace consisting of `[1gn, 1penumbra]`.
    let swap_execution = state.swap_execution(0, trading_pair).await?.unwrap();

    assert_eq!(swap_execution.traces.len(), 1);
    assert_eq!(swap_execution.traces[0].len(), 2);

    assert_eq!(swap_execution.traces[0][0].asset_id, gn.id());
    assert_eq!(swap_execution.traces[0][1].asset_id, penumbra.id());
    assert_eq!(swap_execution.traces[0][0].amount, 1u32.into());
    assert_eq!(swap_execution.traces[0][1].amount, 1u32.into());

    // Now do a more complicated swap execution through a few positions
    // and asset types.

    // Reset storage and state for this test.
    let storage = TempStorage::new().await?.apply_default_genesis().await?;
    let mut state = Arc::new(StateDelta::new(storage.latest_snapshot()));
    let mut state_tx = state.try_begin_transaction().unwrap();

    // Flow:
    //
    //                       ┌───────┐       ┌─────┐
    //                  ┌───▶│  5gm  │──────▶│ 5gn │
    // ┌────────────┐   │    └───────┘       └─────┘
    // │ 10penumbra │───┤
    // └────────────┘   │
    //                  │    ┌───────┐      ┌──────┐       ┌──────┐
    //                  └───▶│ 1pusd │─────▶│ 20gm │──────▶│ 20gn │
    //                       └───────┘      └──────┘       └──────┘
    let gm = asset::REGISTRY.parse_unit("gm");
    let pusd = asset::REGISTRY.parse_unit("pusd");

    let pair_gm_gn = Market::new(gm.clone(), gn.clone());
    let pair_gm_pusd = Market::new(gm.clone(), pusd.clone());
    let pair_penumbra_gm = Market::new(penumbra.clone(), gm.clone());
    let pair_penumbra_pusd = Market::new(penumbra.clone(), pusd.clone());

    // Create a 1:1 gm:penumbra position (i.e. sell 5 gm at 1 penumbra each).
    let buy_1 = limit_buy(pair_penumbra_gm.clone(), 5u64.into(), 1u64.into());
    state_tx.put_position(buy_1);

    // Create a 5:1 pusd:penumbra position (i.e. sell 1 pusd at 5 penumbra each).
    let buy_2 = limit_buy(pair_penumbra_pusd.clone(), 1u64.into(), 5u64.into());
    state_tx.put_position(buy_2);

    // Create a 1:1 gm:gn position (i.e. sell 25 gn at 1 gm each).
    let buy_3 = limit_buy(pair_gm_gn.clone(), 25u64.into(), 1u64.into());
    state_tx.put_position(buy_3);

    // Create a 1:20 pusd:gm position (i.e. sell 1 pusd at 20 gm each).
    let buy_4 = limit_buy(pair_gm_pusd.clone(), 20u64.into(), 1u64.into());
    state_tx.put_position(buy_4);

    state_tx.apply();

    // Now we should be able to fill a 10penumbra into 25gn swap.
    let trading_pair = pair_penumbra_gn.into_directed_trading_pair().into();

    let mut swap_flow = state.swap_flow(&trading_pair);

    assert!(trading_pair.asset_1() == penumbra.id());

    // Add the amount of each asset being swapped to the batch swap flow.
    swap_flow.0 += MockFlowCiphertext::new(10u32.into());
    swap_flow.1 += MockFlowCiphertext::new(0u32.into());

    // Set the batch swap flow for the trading pair.
    Arc::get_mut(&mut state)
        .unwrap()
        .put_swap_flow(&trading_pair, swap_flow.clone());
    state
        .handle_batch_swaps(trading_pair, swap_flow, 0u32.into(), 0)
        .await
        .expect("unable to process batch swaps");

    let output_data = state.output_data(0, trading_pair).await?.unwrap();

    // Output data should have 10 penumbra in and 25gn out
    assert_eq!(output_data.delta_1, 10u32.into());
    assert_eq!(output_data.lambda_2_1, 25u32.into());

    // Swap execution should have two traces.
    let swap_execution = state.swap_execution(0, trading_pair).await?.unwrap();

    assert_eq!(swap_execution.traces.len(), 2);
    assert_eq!(swap_execution.traces[0].len(), 2);

    // assert_eq!(swap_execution.traces[0][0].asset_id, gn.id());
    // assert_eq!(swap_execution.traces[0][1].asset_id, penumbra.id());
    // assert_eq!(swap_execution.traces[0][0].amount, 1u32.into());
    // assert_eq!(swap_execution.traces[0][1].amount, 1u32.into());

    Ok(())
}
