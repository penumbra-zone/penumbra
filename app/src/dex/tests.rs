use anyhow::Ok;
use futures::StreamExt;
use penumbra_crypto::{
    asset,
    dex::{
        lp::{position::Position, Reserves},
        BatchSwapOutputData, DirectedTradingPair, DirectedUnitPair,
    },
    Amount, MockFlowCiphertext,
};
use penumbra_storage::{ArcStateDeltaExt, StateDelta, TempStorage};

use rand_core::OsRng;

use penumbra_crypto::Value;

use crate::dex::{
    position_manager::PositionManager,
    router::FillRoute,
    router::{limit_buy, limit_sell, HandleBatchSwaps, RoutingParams},
    Arbitrage, StateWriteExt,
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
    let delta_gm = Value {
        amount: 100_000u64.into(),
        asset_id: gm.id(),
    };

    let lambda_gm = Value {
        amount: 0u64.into(),
        asset_id: gm.id(),
    };

    let lambda_gn = Value {
        amount: 120_000u64.into(),
        asset_id: gn.id(),
    };

    let route_to_gn = vec![gn.id()];
    let (unfilled, output) =
        FillRoute::fill_route(&mut state_test_1, delta_gm, &route_to_gn, None).await?;

    assert_eq!(unfilled, lambda_gm);
    assert_eq!(output, lambda_gn);

    let position = state_test_1
        .position_by_id(&position_1_id)
        .await
        .unwrap()
        .unwrap();

    // Check that the position is filled
    assert_eq!(position.reserves.r1, delta_gm.amount);
    assert_eq!(position.reserves.r2, Amount::zero());

    let route_to_gm = vec![gm.id()];
    // Now we try to fill the order a second time, this should leave the position untouched.
    assert!(state_test_1
        .fill_route(delta_gm, &route_to_gn, None)
        .await
        .is_err());

    // Fetch the position, and assert that its reserves are unchanged.
    let position = state_test_1
        .position_by_id(&position_1_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(position.reserves.r1, delta_gm.amount);
    assert_eq!(position.reserves.r2, Amount::zero());

    // Now let's try to do a "round-trip" i.e. fill the order in the other direction:
    let delta_gn = Value {
        amount: 120_000u64.into(),
        asset_id: gn.id(),
    };

    let (unfilled, output) = state_test_1
        .fill_route(delta_gn, &route_to_gm, None)
        .await?;
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
        let delta_gm = Value {
            amount: 1000u64.into(),
            asset_id: gm.id(),
        };
        // We are splitting a single large fill for a `100_000gm` into, 100 fills for `1000gm`.
        let (unfilled, output) = state_tx.fill_route(delta_gm, &route_to_gn, None).await?;

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
    let delta_gm = Value {
        amount: 100_091u64.into(),
        asset_id: gm.id(),
    };

    let route_to_gn = vec![gn.id()];
    let (unfilled, output) = state_test_1
        .fill_route(delta_gm, &route_to_gn, None)
        .await?;

    assert_eq!(unfilled.asset_id, gm.id());
    assert_eq!(unfilled.amount, Amount::zero());
    assert_eq!(output.amount, 120_000u64.into());
    assert_eq!(output.asset_id, gn.id());

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
    let delta_gm = Value {
        amount: delta_gm.amount + 100_101u64.into(),
        asset_id: gm.id(),
    };

    let (unfilled, output) = state_test_2
        .fill_route(delta_gm, &route_to_gn, None)
        .await?;
    assert_eq!(unfilled.asset_id, gm.id());
    assert_eq!(output.asset_id, gn.id());
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
    let delta_gm = Value {
        amount: delta_gm.amount + 100_111u64.into(),
        asset_id: gm.id(),
    };

    let (unfilled, output) = state_test_3
        .fill_route(delta_gm, &route_to_gn, None)
        .await?;
    assert_eq!(unfilled.asset_id, gm.id());
    assert_eq!(output.asset_id, gn.id());
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

    let pair_gn_penumbra = DirectedUnitPair::new(gn.clone(), penumbra.clone());

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
    swap_flow.1 += MockFlowCiphertext::new(gn.value(1u32.into()).amount);

    // Set the batch swap flow for the trading pair.
    Arc::get_mut(&mut state)
        .unwrap()
        .put_swap_flow(&trading_pair, swap_flow.clone());
    state
        .handle_batch_swaps(trading_pair, swap_flow, 0, 0, RoutingParams::default())
        .await
        .expect("unable to process batch swaps");

    // Swap execution should have a single trace consisting of `[1gn, 1penumbra]`.
    let swap_execution = state.swap_execution(0, trading_pair).await?.unwrap();

    assert_eq!(
        swap_execution.traces,
        vec![vec![gn.value(1u32.into()), penumbra.value(1u32.into()),]]
    );

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
    let pusd = asset::REGISTRY.parse_unit("test_usd");

    tracing::info!(gm_id = ?gm.id());
    tracing::info!(gn_id = ?gn.id());
    tracing::info!(pusd_id = ?pusd.id());
    tracing::info!(penumbra_id = ?penumbra.id());

    // Working backwards through the graph:

    // Sell 25 gn at 1 gm each.
    state_tx.put_position(limit_sell(
        DirectedUnitPair::new(gn.clone(), gm.clone()),
        25u64.into(),
        1u64.into(),
    ));
    // Buy 1 pusd at 20 gm each.
    state_tx.put_position(limit_buy(
        DirectedUnitPair::new(pusd.clone(), gm.clone()),
        1u64.into(),
        20u64.into(),
    ));
    // Buy 5 penumbra at 1 gm each.
    state_tx.put_position(limit_buy(
        DirectedUnitPair::new(penumbra.clone(), gm.clone()),
        5u64.into(),
        1u64.into(),
    ));
    // Sell 1pusd at 5 penumbra each.
    state_tx.put_position(limit_sell(
        DirectedUnitPair::new(pusd.clone(), penumbra.clone()),
        1u64.into(),
        5u64.into(),
    ));

    state_tx.apply();

    // Now we should be able to fill a 10penumbra into 25gn swap.
    let trading_pair = pair_gn_penumbra.into_directed_trading_pair().into();

    let mut swap_flow = state.swap_flow(&trading_pair);

    assert!(trading_pair.asset_1() == penumbra.id());

    // Add the amount of each asset being swapped to the batch swap flow.
    swap_flow.0 += MockFlowCiphertext::new(Amount::from(10u32) * penumbra.unit_amount());
    swap_flow.1 += MockFlowCiphertext::new(Amount::from(0u32) * gn.unit_amount());

    // Set the batch swap flow for the trading pair.
    Arc::get_mut(&mut state)
        .unwrap()
        .put_swap_flow(&trading_pair, swap_flow.clone());
    state
        .handle_batch_swaps(
            trading_pair,
            swap_flow,
            0u32.into(),
            0,
            RoutingParams::default(),
        )
        .await
        .expect("unable to process batch swaps");

    let output_data = state.output_data(0, trading_pair).await?.unwrap();

    assert_eq!(
        output_data,
        BatchSwapOutputData {
            delta_1: penumbra.value(10u32.into()).amount,
            delta_2: 0u32.into(),
            lambda_1: 0u32.into(),
            lambda_2: gn.value(25u32.into()).amount,
            unfilled_1: 0u32.into(),
            unfilled_2: 0u32.into(),
            height: 0,
            epoch_height: 0,
            trading_pair,
        }
    );

    // Swap execution should have two traces.
    let swap_execution = state.swap_execution(0, trading_pair).await?.unwrap();

    assert_eq!(
        swap_execution.traces,
        vec![
            vec![
                penumbra.value(5u32.into()),
                pusd.value(1u32.into()),
                gm.value(20u32.into()),
                gn.value(20u32.into()),
            ],
            vec![
                penumbra.value(5u32.into()),
                gm.value(5u32.into()),
                gn.value(5u32.into()),
            ],
        ]
    );

    Ok(())
}

#[tokio::test]
/// Test that a basic cycle arb is detected and filled.
async fn basic_cycle_arb() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt::try_init();
    let storage = TempStorage::new().await?.apply_default_genesis().await?;
    let mut state = Arc::new(StateDelta::new(storage.latest_snapshot()));
    let mut state_tx = state.try_begin_transaction().unwrap();

    let penumbra = asset::REGISTRY.parse_unit("penumbra");
    let gm = asset::REGISTRY.parse_unit("gm");
    let gn = asset::REGISTRY.parse_unit("gn");

    tracing::info!(gm_id = ?gm.id());
    tracing::info!(gn_id = ?gn.id());
    tracing::info!(penumbra_id = ?penumbra.id());

    // Sell 10 gn at 1 penumbra each.
    state_tx.put_position(limit_sell(
        DirectedUnitPair::new(gn.clone(), penumbra.clone()),
        10u64.into(),
        1u64.into(),
    ));
    // Buy 100 gn at 2 gm each.
    state_tx.put_position(limit_buy(
        DirectedUnitPair::new(gn.clone(), gm.clone()),
        100u64.into(),
        2u64.into(),
    ));
    // Sell 100 penumbra at 1 gm each.
    state_tx.put_position(limit_sell(
        DirectedUnitPair::new(penumbra.clone(), gm.clone()),
        100u64.into(),
        1u64.into(),
    ));
    state_tx.apply();

    // Now we should be able to arb 10penumbra => 10gn => 20gm => 20penumbra.
    state
        .arbitrage(penumbra.id(), vec![penumbra.id(), gm.id(), gn.id()])
        .await?;

    let arb_execution = state.arb_execution(0).await?.expect("arb was performed");
    assert_eq!(
        arb_execution.traces,
        vec![vec![
            penumbra.value(10u32.into()),
            gn.value(10u32.into()),
            gm.value(20u32.into()),
            penumbra.value(20u32.into()),
        ],]
    );

    Ok(())
}
