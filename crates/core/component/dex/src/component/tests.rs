use std::sync::Arc;

use anyhow::Ok;
use async_trait::async_trait;
use cnidarium::{ArcStateDeltaExt, StateDelta, TempStorage};
use futures::StreamExt;
use penumbra_sdk_asset::{asset, Value};
use penumbra_sdk_num::Amount;
use rand_core::OsRng;

use crate::component::{SwapDataRead, SwapDataWrite};
use crate::lp::action::PositionOpen;
use crate::lp::{position, SellOrder};
use crate::DexParameters;
use crate::{
    component::{
        router::FillRoute,
        router::{create_buy, create_sell, HandleBatchSwaps, RoutingParams},
        Arbitrage, PositionManager, PositionRead, StateReadExt, StateWriteExt,
    },
    lp::{position::Position, Reserves},
    BatchSwapOutputData, DirectedTradingPair, DirectedUnitPair,
};

#[async_trait]
pub trait TempStorageExt: Sized {
    async fn apply_minimal_genesis(self) -> anyhow::Result<Self>;
}

#[async_trait]
impl TempStorageExt for TempStorage {
    async fn apply_minimal_genesis(self) -> anyhow::Result<Self> {
        use penumbra_sdk_sct::component::clock::EpochManager as _;
        let mut state = StateDelta::new(self.latest_snapshot());

        state.put_block_height(0);
        state.put_epoch_by_height(
            0,
            penumbra_sdk_sct::epoch::Epoch {
                index: 0,
                start_height: 0,
            },
        );
        state.put_dex_params(DexParameters::default());

        self.commit(state).await?;

        Ok(self)
    }
}

#[tokio::test]
/// Builds a simple order book with a single limit order, and tests different
/// market order execution against it.
async fn single_close_on_fill() -> anyhow::Result<()> {
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
    state_tx.open_position(position_1.clone()).await.unwrap();

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

    // Test 1: we exhaust the entire position.
    // In theory, we would only need to swap 100_000gm, to get back
    // 120_000gn, with no unfilled amount and resulting reserves of 100_000gm and 0gn.
    // However, since we have to account for rounding, we need to swap 100_001gm which
    // gets us back 120_000gn. The unfilled amount is 1gm, and the resulting reserves are
    // 100_000gm and 0gn.
    let delta_gm = Value {
        amount: 100_001u64.into(),
        asset_id: gm.id(),
    };

    let expected_lambda_gm = Value {
        amount: 1u64.into(),
        asset_id: gm.id(),
    };

    let expected_lambda_gn = Value {
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

    assert_eq!(unfilled, expected_lambda_gm.amount);
    assert_eq!(output, expected_lambda_gn);

    let position = state_test_1
        .position_by_id(&position_1_id)
        .await
        .unwrap()
        .unwrap();

    // Check that the position is filled
    assert_eq!(position.reserves.r2, Amount::zero());
    // We got back 1gm of unfilled amount
    let expected_r1 = delta_gm.amount.checked_sub(&unfilled).unwrap();
    assert_eq!(position.reserves.r1, expected_r1);

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
    assert_eq!(position.reserves.r1, expected_r1);
    assert_eq!(position.reserves.r2, Amount::zero());

    // Now let's try to do a "round-trip" i.e. fill the order in the other direction:
    let delta_gn = Value {
        amount: 120_001u64.into(),
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

    assert_eq!(unfilled, Amount::from(1u64));
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
        assert_eq!(unfilled, Amount::from(0u64));
        // And that for every `1000gm`, we get `1199gn`.
        assert_eq!(
            output,
            Value {
                amount: 1199u64.into(),
                asset_id: gn.id(),
            }
        );
    }

    // After executing 100 swaps of `1000gm` into `gn`. We should have acquired `1199gn*100` or `119900gn`.
    // We consume the last `100gn` in the next swap.
    let delta_gm = Value {
        amount: 84u64.into(),
        asset_id: gm.id(),
    };
    let execution = state_tx.fill_route(delta_gm, &route_to_gn, None).await?;
    let unfilled = delta_gm
        .amount
        .checked_sub(&execution.input.amount)
        .unwrap();
    let output = execution.output;
    assert_eq!(unfilled, Amount::from(0u64));
    assert_eq!(
        output,
        Value {
            amount: 100u64.into(),
            asset_id: gn.id(),
        }
    );

    // Now, we want to verify that the position is updated with the correct reserves.
    // We should have depleted the order of all its `gn`s, and replaced it with `100_084gm`
    // This accounts for the rounding behavior that arises when swapping smaller quantities.
    let position = state_tx
        .position_by_id(&position_1_id)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(position.reserves.r1, 100_084u64.into());
    assert_eq!(position.reserves.r2, Amount::zero());

    Ok(())
}

#[tokio::test]
/// Builds a simple order book with a two orders, fills against them both,
/// and checks that one of the orders is auto-closed.
async fn check_close_on_fill() -> anyhow::Result<()> {
    let storage = TempStorage::new().await?.apply_minimal_genesis().await?;
    let mut state = Arc::new(StateDelta::new(storage.latest_snapshot()));
    let mut state_tx = state.try_begin_transaction().unwrap();

    let gm = asset::Cache::with_known_assets().get_unit("gm").unwrap();

    let mut position_1 = SellOrder::parse_str("100gm@1gn")?.into_position(OsRng);
    position_1.close_on_fill = true;
    let position_2 = SellOrder::parse_str("100gm@1.1gn")?.into_position(OsRng);

    let position_1_id = position_1.id();
    let position_2_id = position_2.id();

    state_tx.open_position(position_1.clone()).await.unwrap();
    state_tx.open_position(position_2.clone()).await.unwrap();

    // Now we have the following liquidity:
    //
    // 100gm@1gn (auto-closing)
    // 100gm@1.1gn
    //
    // We therefore expect that trading 100gn + 110gn will exhaust both positions.
    // Attempting to trade a bit more than that ensures we completely fill both,
    // without worrying about rounding.
    // Because we're just testing the DEX internals, we need to trigger fill_route manually.
    let input = "220gn".parse::<Value>().unwrap();
    let route = [gm.id()];
    let execution = FillRoute::fill_route(&mut state_tx, input, &route, None).await?;

    let unfilled = input.amount.checked_sub(&execution.input.amount).unwrap();

    // Check that we got the execution we expected.
    assert_eq!(unfilled, "10gn".parse::<Value>().unwrap().amount);
    assert_eq!(execution.output, "200gm".parse::<Value>().unwrap());

    // Now grab both position states:
    let position_1_post_exec = state_tx.position_by_id(&position_1_id).await?.unwrap();
    let position_2_post_exec = state_tx.position_by_id(&position_2_id).await?.unwrap();

    dbg!(&position_1_post_exec);
    dbg!(&position_2_post_exec);

    // Check that position 1 was auto-closed but position 2 wasn't:
    assert_eq!(position_1_post_exec.state, position::State::Closed);
    assert_eq!(position_2_post_exec.state, position::State::Opened);

    Ok(())
}

#[tokio::test]
/// Try to execute against multiple positions, mainly testing that the order-book traversal
/// is done correctly.
async fn multiple_close_on_fills() -> anyhow::Result<()> {
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
    state_tx.open_position(position_2.clone()).await.unwrap();
    state_tx.open_position(position_1.clone()).await.unwrap();
    state_tx.open_position(position_3.clone()).await.unwrap();

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
    use cnidarium_component::ActionHandler;
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
        encrypted_metadata: None,
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
    state_tx.open_position(buy_1.clone()).await.unwrap();
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
    state_tx.open_position(buy_2.clone()).await.unwrap();
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
    let buy_1 = create_buy(pair_gn_penumbra.clone(), 1u64.into(), 1u64.into());
    state_tx.open_position(buy_1).await.unwrap();
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
        .accumulate_swap_flow(&trading_pair, swap_flow.clone())
        .await
        .unwrap();
    let routing_params = state.routing_params().await.unwrap();
    state
        .handle_batch_swaps(trading_pair, swap_flow, 0, routing_params, 64)
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
    tracing::info!(penumbra_sdk_id = ?penumbra.id());

    // Working backwards through the graph:

    // Sell 25 gn at 1 gm each.
    state_tx
        .open_position(create_sell(
            DirectedUnitPair::new(gn.clone(), gm.clone()),
            25u64.into(),
            1u64.into(),
        ))
        .await
        .unwrap();
    // Buy 1 pusd at 20 gm each.
    state_tx
        .open_position(create_buy(
            DirectedUnitPair::new(pusd.clone(), gm.clone()),
            1u64.into(),
            20u64.into(),
        ))
        .await
        .unwrap();
    // Buy 5 penumbra at 1 gm each.
    state_tx
        .open_position(create_buy(
            DirectedUnitPair::new(penumbra.clone(), gm.clone()),
            5u64.into(),
            1u64.into(),
        ))
        .await
        .unwrap();
    // Sell 1pusd at 5 penumbra each.
    state_tx
        .open_position(create_sell(
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
        .accumulate_swap_flow(&trading_pair, swap_flow.clone())
        .await
        .unwrap();
    let routing_params = state.routing_params().await.unwrap();
    state
        .handle_batch_swaps(trading_pair, swap_flow, 0u32.into(), routing_params, 64)
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
            trading_pair,
            sct_position_prefix: Default::default(),
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
    tracing::info!(penumbra_sdk_id = ?penumbra.id());

    // Sell 10 gn at 1 penumbra each.
    state_tx
        .open_position(create_sell(
            DirectedUnitPair::new(gn.clone(), penumbra.clone()),
            10u64.into(),
            1u64.into(),
        ))
        .await
        .unwrap();
    // Buy 100 gn at 2 gm each.
    state_tx
        .open_position(create_buy(
            DirectedUnitPair::new(gn.clone(), gm.clone()),
            100u64.into(),
            2u64.into(),
        ))
        .await
        .unwrap();
    // Sell 100 penumbra at 1 gm each.
    state_tx
        .open_position(create_sell(
            DirectedUnitPair::new(penumbra.clone(), gm.clone()),
            100u64.into(),
            1u64.into(),
        ))
        .await
        .unwrap();
    state_tx.apply();

    // Now we should be able to arb 10penumbra => 10gn => 20gm => 20penumbra.
    let routing_params = RoutingParams {
        max_hops: 4 + 2,
        price_limit: Some(1u64.into()),
        fixed_candidates: Arc::new(vec![penumbra.id(), gm.id(), gn.id()]),
    };
    state.arbitrage(penumbra.id(), routing_params).await?;

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
///
/// This test also ensures that the created `SwapExecution` has the
///
/// *Arbitrage execution record bug:*
/// This test also ensures that the created `SwapExecution` has the
/// correct data. (See #3790).
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

    tracing::info!(penumbra_sdk_id= ?penumbra.id());
    tracing::info!(test_usd_id = ?test_usd.id());

    let penumbra_sdk_usd = DirectedUnitPair::new(penumbra.clone(), test_usd.clone());

    /*
     * INITIAL STATE:
     * Position A: Seeks to buy 1 penumbra for 110 test_usd
     * Position B: Seeks to buy 1 penumbra for 100 test_usd
     * Position C: Seeks to sell 10 penumbra for 100 test_usd
     *
     * The arbitrage logic should detect that it can sell 1 penumbra for 110 test_usd,
     * (Position A), and buy it back for 100 test_usd (Position B), and thus make a profit
     * of ~0.1 penumbra. The execution price on the cycle is 0.909 penumbra/test_usd/penumbra, and the
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

    let mut buy_1 = create_buy(penumbra_sdk_usd.clone(), 1u64.into(), 110u64.into());
    buy_1.nonce = [1; 32];

    let mut buy_2 = create_buy(penumbra_sdk_usd.clone(), 1u64.into(), 100u64.into());
    buy_2.nonce = [2; 32];

    let mut sell_1 = create_sell(penumbra_sdk_usd.clone(), 10u64.into(), 100u64.into());
    sell_1.nonce = [0; 32];

    state_tx.open_position(buy_1).await.unwrap();
    state_tx.open_position(buy_2).await.unwrap();
    state_tx.open_position(sell_1).await.unwrap();

    state_tx.apply();

    tracing::info!("we posted the positions");

    tracing::info!("we are triggering the arbitrage logic");

    let routing_params = RoutingParams {
        max_hops: 4 + 2,
        price_limit: Some(1u64.into()),
        fixed_candidates: Arc::new(vec![penumbra.id(), test_usd.id()]),
    };

    let arb_profit = tokio::time::timeout(
        tokio::time::Duration::from_secs(2),
        state.arbitrage(penumbra.id(), routing_params),
    )
    .await??;

    tracing::info!(profit = ?arb_profit, "the arbitrage logic has concluded!");
    // we should have made a profit of 0.01penumbra, with precision loss of 0.000002penumbra.
    let profit: Value = "0.099998penumbra".parse().unwrap();
    assert_eq!(arb_profit, Some(profit));

    tracing::info!("fetching the `ArbExecution`");
    let arb_execution = state.arb_execution(0).await?.expect("arb was performed");
    tracing::info!(?arb_execution, "fetched arb execution!");

    // Validate that the arb execution has the correct data:
    // Validate the traces.
    assert_eq!(
        arb_execution.traces,
        vec![
            vec![
                penumbra.value(1u32.into()),
                test_usd.value(110u32.into()),
                Value {
                    amount: 1099999u64.into(),
                    asset_id: penumbra.id()
                }
            ],
            vec![
                penumbra.value(1u32.into()),
                test_usd.value(100u32.into()),
                Value {
                    amount: 999999u64.into(),
                    asset_id: penumbra.id()
                }
            ]
        ]
    );

    // Validate the input/output of the arb execution:
    assert_eq!(
        arb_execution.input,
        Value {
            amount: 2000000u64.into(),
            asset_id: penumbra.id(),
        }
    );
    assert_eq!(
        arb_execution.output,
        Value {
            amount: 2099998u64.into(),
            asset_id: penumbra.id(),
        }
    );

    Ok(())
}

#[tokio::test]
/// Confirms the ordering of routable assets returns the assets
/// with the most liquidity first, as discovered in https://github.com/penumbra-zone/penumbra/issues/4189
/// For the purposes of this test, it is important to remember
/// that for a trade routing from A -> *, candidate liquidity is
/// the amount of A purchaseable with the candidate assets, i.e. the amount of
/// A in the reserves for any A <-> * positions.
async fn check_routable_asset_ordering() -> anyhow::Result<()> {
    let storage = TempStorage::new().await?.apply_minimal_genesis().await?;
    let mut state = Arc::new(StateDelta::new(storage.latest_snapshot()));
    let mut state_tx = state.try_begin_transaction().unwrap();

    let penumbra = asset::Cache::with_known_assets()
        .get_unit("penumbra")
        .unwrap();
    let test_usd = asset::Cache::with_known_assets()
        .get_unit("test_usd")
        .unwrap();
    let test_btc = asset::Cache::with_known_assets()
        .get_unit("test_btc")
        .unwrap();
    let gn = asset::Cache::with_known_assets().get_unit("gn").unwrap();
    let gm = asset::Cache::with_known_assets().get_unit("gm").unwrap();

    let penumbra_sdk_usd = DirectedTradingPair::new(penumbra.id(), test_usd.id());

    let reserves_1 = Reserves {
        // 0 penumbra
        r1: 0u64.into(),
        // 120,000 test_usd
        r2: 120_000u64.into(),
    };

    let position_1 = Position::new(
        OsRng,
        penumbra_sdk_usd,
        0u32,
        1_200_000u64.into(),
        1_000_000u64.into(),
        reserves_1,
    );

    state_tx.open_position(position_1).await.unwrap();

    let penumbra_sdk_gn = DirectedTradingPair::new(penumbra.id(), gn.id());

    let reserves_2 = Reserves {
        // 130,000 penumbra
        r1: 130_000u64.into(),
        // 0 gn
        r2: 0u64.into(),
    };

    let position_2 = Position::new(
        OsRng,
        penumbra_sdk_gn,
        0u32,
        1_200_000u64.into(),
        1_000_000u64.into(),
        reserves_2,
    );

    state_tx.open_position(position_2).await.unwrap();

    let penumbra_sdk_btc = DirectedTradingPair::new(penumbra.id(), test_btc.id());

    let reserves_3 = Reserves {
        // 100,000 penumbra
        r1: 100_000u64.into(),
        // 50,000 test_btc
        r2: 50_000u64.into(),
    };

    let position_3 = Position::new(
        OsRng,
        penumbra_sdk_btc,
        0u32,
        1_200_000u64.into(),
        1_000_000u64.into(),
        reserves_3,
    );

    state_tx.open_position(position_3).await.unwrap();

    let btc_gm = DirectedTradingPair::new(test_btc.id(), gm.id());

    let reserves_4 = Reserves {
        // 100,000 test_btc
        r1: 100_000u64.into(),
        // 100,000 gm
        r2: 100_000u64.into(),
    };

    let position_4 = Position::new(
        OsRng,
        btc_gm,
        0u32,
        1_200_000u64.into(),
        1_000_000u64.into(),
        reserves_4,
    );

    state_tx.open_position(position_4).await.unwrap();
    state_tx.apply();

    // Expected: GN reserves > BTC reserves, and USD/gm should not appear

    // Find routable assets starting at the Penumbra asset.
    let routable_assets: Vec<_> = state
        .ordered_routable_assets(&penumbra.id())
        .collect::<Vec<_>>()
        .await;
    let routable_assets = routable_assets
        .into_iter()
        .collect::<anyhow::Result<Vec<_>>>()?;

    assert!(
        routable_assets.len() == 2,
        "expected 2 routable assets, got {}",
        routable_assets.len()
    );

    let first = routable_assets[0];
    let second = routable_assets[1];
    assert!(
        first == gn.id(),
        "expected GN ({}) to be the first routable asset, got {}",
        gn.id(),
        first.clone()
    );

    assert!(
        second == test_btc.id(),
        "expected BTC ({}) to be the second routable asset, got {}",
        test_btc.id(),
        second.clone()
    );

    Ok(())
}
