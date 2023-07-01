use std::sync::Arc;

use anyhow::Ok;
use async_trait::async_trait;
use futures::StreamExt;
use penumbra_asset::{asset, Value};
use penumbra_num::Amount;
use penumbra_storage::{ArcStateDeltaExt, StateDelta, TempStorage};
use rand_core::OsRng;

//use crate::TempStorageExt;

use crate::lp::action::PositionOpen;
use crate::{
    component::{
        router::FillRoute,
        router::{limit_buy, limit_sell, HandleBatchSwaps, RoutingParams},
        Arbitrage, PositionManager, PositionRead, StateReadExt, StateWriteExt,
    },
    lp::{position::Position, Reserves},
    BatchSwapOutputData, DirectedTradingPair, DirectedUnitPair,
};

// TODO: what's the right way to mock genesis? if component A needs component B,
// do we need a way to mock B's genesis in A's tests? or should we only do unit
// tests for A, integration tests for A+B?

#[async_trait]
pub trait TempStorageExt: Sized {
    async fn apply_minimal_genesis(self) -> anyhow::Result<Self>;
}

#[async_trait]
impl TempStorageExt for TempStorage {
    async fn apply_minimal_genesis(self) -> anyhow::Result<Self> {
        use penumbra_chain::component::StateWriteExt;

        let mut state = StateDelta::new(self.latest_snapshot());

        // TODO: this corresponds to code in App that should be part of
        // penumbra_chain or something (TBD: how to split up penumbra-chain?
        // params should be at the top, stuff like this should be at the bottom)

        state.put_block_height(0);
        state.put_epoch_by_height(
            0,
            penumbra_chain::Epoch {
                index: 0,
                start_height: 0,
            },
        );

        self.commit(state).await?;

        Ok(self)
    }
}

#[tokio::test]
/// Builds a simple order book with a single limit order, and tests different
/// market order execution against it.
async fn single_limit_order() -> anyhow::Result<()> {
    let storage = TempStorage::new().await?.apply_minimal_genesis().await?;
    let mut state = Arc::new(StateDelta::new(storage.latest_snapshot()));
    let mut state_tx = state.try_begin_transaction().unwrap();

    let gm = asset::Cache::with_known_assets().get_unit("gm").unwrap();
    let gn = asset::Cache::with_known_assets().get_unit("gn").unwrap();

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
    state_tx.put_position(position_1.clone()).await.unwrap();

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
    let execution = FillRoute::fill_route(&mut state_test_1, delta_gm, &route_to_gn, None).await?;

    let unfilled = delta_gm
        .amount
        .checked_sub(&execution.input.amount)
        .unwrap();
    let output = execution.output;

    assert_eq!(unfilled, lambda_gm.amount);
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

    let execution = state_test_1
        .fill_route(delta_gn, &route_to_gm, None)
        .await?;

    let unfilled = delta_gn
        .amount
        .checked_sub(&execution.input.amount)
        .unwrap();
    let output = execution.output;

    assert_eq!(unfilled, Amount::zero(),);
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
        let execution = state_tx.fill_route(delta_gm, &route_to_gn, None).await?;

        let unfilled = delta_gm
            .amount
            .checked_sub(&execution.input.amount)
            .unwrap();
        let output = execution.output;

        // We check that there are no unfilled `gm`s resulting from executing the order
        assert_eq!(unfilled, Amount::zero());
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
    let storage = TempStorage::new().await?.apply_minimal_genesis().await?;
    let mut state = Arc::new(StateDelta::new(storage.latest_snapshot()));
    let mut state_tx = state.try_begin_transaction().unwrap();

    let gm = asset::Cache::with_known_assets().get_unit("gm").unwrap();
    let gn = asset::Cache::with_known_assets().get_unit("gn").unwrap();

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
    state_tx.put_position(position_2.clone()).await.unwrap();
    state_tx.put_position(position_1.clone()).await.unwrap();
    state_tx.put_position(position_3.clone()).await.unwrap();

    let mut full_orderbook_state = state_tx;

    let mut state_test_1 = full_orderbook_state.fork();

    // Since we are looking to exhaust the order's reserves, Delta_1's amount
    // must account for the 9 bps fee, rounded up.
    let delta_gm = Value {
        amount: 100_091u64.into(),
        asset_id: gm.id(),
    };

    let route_to_gn = vec![gn.id()];
    let execution = state_test_1
        .fill_route(delta_gm, &route_to_gn, None)
        .await?;

    let unfilled = delta_gm
        .amount
        .checked_sub(&execution.input.amount)
        .unwrap();
    let output = execution.output;

    assert_eq!(execution.input.asset_id, gm.id());
    assert_eq!(unfilled, Amount::zero());
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

    let execution = state_test_2
        .fill_route(delta_gm, &route_to_gn, None)
        .await?;

    let unfilled = delta_gm
        .amount
        .checked_sub(&execution.input.amount)
        .unwrap();
    let output = execution.output;

    assert_eq!(execution.input.asset_id, gm.id());
    assert_eq!(output.asset_id, gn.id());
    assert_eq!(unfilled, Amount::zero());
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

    let execution = state_test_3
        .fill_route(delta_gm, &route_to_gn, None)
        .await?;

    let unfilled = delta_gm
        .amount
        .checked_sub(&execution.input.amount)
        .unwrap();
    let output = execution.output;

    assert_eq!(execution.input.asset_id, gm.id());
    assert_eq!(output.asset_id, gn.id());
    assert_eq!(unfilled, Amount::zero());
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
    use penumbra_component::ActionHandler;
    let gm = asset::Cache::with_known_assets().get_unit("gm").unwrap();
    let gn = asset::Cache::with_known_assets().get_unit("gn").unwrap();

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

    let position_action = PositionOpen {
        position: position_1,
    };

    assert!(position_action.check_stateless(()).await.is_err());

    Ok(())
}

#[tokio::test]
/// Test that positions are created and returned as expected.
async fn position_create_and_retrieve() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt::try_init();
    let storage = TempStorage::new().await?.apply_minimal_genesis().await?;
    let mut state = Arc::new(StateDelta::new(storage.latest_snapshot()));
    let mut state_tx = state.try_begin_transaction().unwrap();

    let gm = asset::Cache::with_known_assets().get_unit("gm").unwrap();
    let gn = asset::Cache::with_known_assets().get_unit("gn").unwrap();

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
    state_tx.put_position(buy_1.clone()).await.unwrap();
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
    state_tx.put_position(buy_2.clone()).await.unwrap();
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
    let storage = TempStorage::new().await?.apply_minimal_genesis().await?;
    let mut state = Arc::new(StateDelta::new(storage.latest_snapshot()));
    let mut state_tx = state.try_begin_transaction().unwrap();

    let penumbra = asset::Cache::with_known_assets()
        .get_unit("penumbra")
        .unwrap();
    let gn = asset::Cache::with_known_assets().get_unit("gn").unwrap();

    let pair_gn_penumbra = DirectedUnitPair::new(gn.clone(), penumbra.clone());

    // Create a single 1:1 gn:penumbra position (i.e. buy 1 gn at 1 penumbra).
    let buy_1 = limit_buy(pair_gn_penumbra.clone(), 1u64.into(), 1u64.into());
    state_tx.put_position(buy_1).await.unwrap();
    state_tx.apply();

    // Now we should be able to fill a 1:1 gn:penumbra swap.
    let trading_pair = pair_gn_penumbra.into_directed_trading_pair().into();

    let mut swap_flow = state.swap_flow(&trading_pair);

    assert!(trading_pair.asset_1() == penumbra.id());

    // Add the amount of each asset being swapped to the batch swap flow.
    swap_flow.0 += 0u32.into();
    swap_flow.1 += gn.value(1u32.into()).amount;

    // Set the batch swap flow for the trading pair.
    Arc::get_mut(&mut state)
        .unwrap()
        .put_swap_flow(&trading_pair, swap_flow.clone());
    state
        .handle_batch_swaps(trading_pair, swap_flow, 0, 0, RoutingParams::default())
        .await
        .expect("unable to process batch swaps");

    // Swap execution should have a single trace consisting of `[1gn, 1penumbra]`.
    let swap_execution = state
        .swap_execution(
            0,
            DirectedTradingPair::new(trading_pair.asset_2, trading_pair.asset_1),
        )
        .await?
        .unwrap();

    // TODO: check the other direction's swap execution too.

    assert_eq!(
        swap_execution.traces,
        vec![vec![gn.value(1u32.into()), penumbra.value(1u32.into()),]]
    );

    // Now do a more complicated swap execution through a few positions
    // and asset types.

    // Reset storage and state for this test.
    let storage = TempStorage::new().await?.apply_minimal_genesis().await?;
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
    let gm = asset::Cache::with_known_assets().get_unit("gm").unwrap();
    let pusd = asset::Cache::with_known_assets()
        .get_unit("test_usd")
        .unwrap();

    tracing::info!(gm_id = ?gm.id());
    tracing::info!(gn_id = ?gn.id());
    tracing::info!(pusd_id = ?pusd.id());
    tracing::info!(penumbra_id = ?penumbra.id());

    // Working backwards through the graph:

    // Sell 25 gn at 1 gm each.
    state_tx
        .put_position(limit_sell(
            DirectedUnitPair::new(gn.clone(), gm.clone()),
            25u64.into(),
            1u64.into(),
        ))
        .await
        .unwrap();
    // Buy 1 pusd at 20 gm each.
    state_tx
        .put_position(limit_buy(
            DirectedUnitPair::new(pusd.clone(), gm.clone()),
            1u64.into(),
            20u64.into(),
        ))
        .await
        .unwrap();
    // Buy 5 penumbra at 1 gm each.
    state_tx
        .put_position(limit_buy(
            DirectedUnitPair::new(penumbra.clone(), gm.clone()),
            5u64.into(),
            1u64.into(),
        ))
        .await
        .unwrap();
    // Sell 1pusd at 5 penumbra each.
    state_tx
        .put_position(limit_sell(
            DirectedUnitPair::new(pusd.clone(), penumbra.clone()),
            1u64.into(),
            5u64.into(),
        ))
        .await
        .unwrap();

    state_tx.apply();

    // Now we should be able to fill a 10penumbra into 25gn swap.
    let trading_pair = pair_gn_penumbra.into_directed_trading_pair().into();

    let mut swap_flow = state.swap_flow(&trading_pair);

    assert!(trading_pair.asset_1() == penumbra.id());

    // Add the amount of each asset being swapped to the batch swap flow.
    swap_flow.0 += Amount::from(10u32) * penumbra.unit_amount();
    swap_flow.1 += Amount::from(0u32) * gn.unit_amount();

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
            epoch_starting_height: 0,
            trading_pair,
        }
    );

    // Swap execution should have two traces.
    let swap_execution = state
        .swap_execution(
            0,
            DirectedTradingPair::new(trading_pair.asset_1, trading_pair.asset_2),
        )
        .await?
        .unwrap();

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
    let storage = TempStorage::new().await?.apply_minimal_genesis().await?;
    let mut state = Arc::new(StateDelta::new(storage.latest_snapshot()));
    let mut state_tx = state.try_begin_transaction().unwrap();

    let penumbra = asset::Cache::with_known_assets()
        .get_unit("penumbra")
        .unwrap();

    let gm = asset::Cache::with_known_assets().get_unit("gm").unwrap();

    let gn = asset::Cache::with_known_assets().get_unit("gn").unwrap();

    tracing::info!(gm_id = ?gm.id());
    tracing::info!(gn_id = ?gn.id());
    tracing::info!(penumbra_id = ?penumbra.id());

    // Sell 10 gn at 1 penumbra each.
    state_tx
        .put_position(limit_sell(
            DirectedUnitPair::new(gn.clone(), penumbra.clone()),
            10u64.into(),
            1u64.into(),
        ))
        .await
        .unwrap();
    // Buy 100 gn at 2 gm each.
    state_tx
        .put_position(limit_buy(
            DirectedUnitPair::new(gn.clone(), gm.clone()),
            100u64.into(),
            2u64.into(),
        ))
        .await
        .unwrap();
    // Sell 100 penumbra at 1 gm each.
    state_tx
        .put_position(limit_sell(
            DirectedUnitPair::new(penumbra.clone(), gm.clone()),
            100u64.into(),
            1u64.into(),
        ))
        .await
        .unwrap();
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

#[tokio::test]
/// Reproduce the arbitrage loop bug that caused testnet 53 to stall.
/// The issue was that we did not treat the spill price as a strict
/// upper bound, which is necessary to ensure that the arbitrage logic
/// terminates.
async fn reproduce_arbitrage_loop_testnet_53() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt::try_init();
    let storage = TempStorage::new().await?.apply_minimal_genesis().await?;
    let mut state = Arc::new(StateDelta::new(storage.latest_snapshot()));
    let mut state_tx = state.try_begin_transaction().unwrap();

    let penumbra = asset::Cache::with_known_assets()
        .get_unit("penumbra")
        .unwrap();
    let test_usd = asset::Cache::with_known_assets()
        .get_unit("test_usd")
        .unwrap();

    tracing::info!(penumbra_id= ?penumbra.id());
    tracing::info!(test_usd_id = ?test_usd.id());

    let penumbra_usd = DirectedUnitPair::new(penumbra.clone(), test_usd.clone());

    /*
     * INITIAL STATE:
     * Position A: Seeks to buy 1 penumbra for 110 test_usd
     * Position B: Seeks to buy 1 penumbra for 100 test_usd
     * Position C: Seeks to sell 10 penumbra for 100 test_usd
     *
     * The arbitrage logic should detect that it can sell 1 penumbra for 110 test_usd,
     * (Position A), and buy it back for 100 test_usd (Position B), and thus make a profit
     * of 0.1 penumbra. The execution price on the cycle is 0.909 penumbra/test_usd/penumbra, and the
     * spill price is 1 penumbra/test_usd/penumbra.
     *
     * So the arbitrage logic found a profitable cycle, and executed it. In this setup,
     * there are no more profitable cycles, so the arbitrage logic should terminate.
     *
     * It doesn't, because we did not treat the spill price as a strict upper bound.
     *
     * AFTER EXECUTING THE FIRST ARBITRAGE CYCLE:
     * Position A: Seeks to sell 1 penumbra for 110 test_usd
     * Position B: Seeks to sell 1 penumbra for 100 test_usd
     * Position C: Seeks to sell 10 penumbra for 100 test_usd
     *
     *
     * Notice that Position A and Position B are now both seeking to sell penumbra for usd,
     * because they were filled previously. The execution price on penumbra -> usd -> penumbra
     * is 1, and the spill price is 1. The fact that we don't treat the spill price as a strict
     * upper bound means that we will execute the arbitrage logic again even though there are no
     * profitable cycles (=surplus).
     *
     */

    let mut buy_1 = limit_buy(penumbra_usd.clone(), 1u64.into(), 110u64.into());
    buy_1.nonce = [1; 32];

    let mut buy_2 = limit_buy(penumbra_usd.clone(), 1u64.into(), 100u64.into());
    buy_2.nonce = [2; 32];

    let mut sell_1 = limit_sell(penumbra_usd.clone(), 10u64.into(), 100u64.into());
    sell_1.nonce = [0; 32];

    state_tx.put_position(buy_1).await.unwrap();
    state_tx.put_position(buy_2).await.unwrap();
    state_tx.put_position(sell_1).await.unwrap();

    state_tx.apply();

    tracing::info!("we posted the positions");

    tracing::info!("we are triggering the arbitrage logic");

    let arb_profit = tokio::time::timeout(
        tokio::time::Duration::from_secs(2),
        state.arbitrage(penumbra.id(), vec![penumbra.id(), test_usd.id()]),
    )
    .await??;

    tracing::info!(profit = ?arb_profit, "the arbitrage logic has concluded!");
    let profit: Value = "0.1penumbra".parse().unwrap();
    assert_eq!(arb_profit, profit);

    tracing::info!("fetching the `ArbExecution`");
    let arb_execution = state.arb_execution(0).await?.expect("arb was performed");
    tracing::info!(?arb_execution, "fetched arb execution!");
    Ok(())
}
