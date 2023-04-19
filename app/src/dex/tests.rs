mod test {
    use anyhow::Ok;
    use penumbra_crypto::{
    asset::{self, Unit},
    dex::{
        lp::{
            position::{self, Position},
            Reserves, TradingFunction,
        },
        DirectedTradingPair,
    },
    fixpoint::U128x128,
    Amount,
};
use penumbra_storage::{ArcStateDeltaExt, StateDelta, TempStorage};

use rand_core::OsRng;

use penumbra_crypto::Value;

use crate::dex::position_manager::PositionRead;
use crate::dex::{position_manager::PositionManager, router::FillRoute};
use crate::TempStorageExt;
use futures::StreamExt;
use std::{collections::BTreeMap, sync::Arc};

use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

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

#[derive(Clone, Debug)]
struct Market {
    start: Unit,
    end: Unit,
}

impl Market {
    fn new(start: Unit, end: Unit) -> Self {
        Self { start, end }
    }
    fn into_directed_trading_pair(&self) -> DirectedTradingPair {
        DirectedTradingPair {
            start: self.start.id(),
            end: self.end.id(),
        }
    }
}

/// Create a `Position` that seeks to acquire `asset_1` at a `price` denominated in
/// numeraire (`asset_2`), by provisioning enough numeraire.
fn limit_buy(market: Market, quantity: Amount, price_in_numeraire: Amount) -> Position {
    Position::new(
        OsRng,
        market.into_directed_trading_pair(),
        0u32,
        price_in_numeraire * market.end.unit_amount(),
        Amount::from(1u64) * market.start.unit_amount(),
        Reserves {
            r1: Amount::zero(),
            r2: quantity * market.end.unit_amount() * price_in_numeraire,
        },
    )
}

/// Create a `Position` that seeks to shed `asset_1` at a `price` denominated in
/// numeraire (`asset_2`).
fn limit_sell(market: Market, quantity: Amount, price_in_numeraire: Amount) -> Position {
    Position::new(
        OsRng,
        market.into_directed_trading_pair(),
        0u32,
        Amount::from(1u64) * market.end.unit_amount(),
        price_in_numeraire * market.start.unit_amount(),
        Reserves {
            r1: quantity * market.start.unit_amount(),
            r2: Amount::zero(),
        },
    )
}

#[tokio::test]
async fn put_position_get_best_price() -> anyhow::Result<()> {
    let subscriber = FmtSubscriber::builder()
        // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
        // will be written to stdout.
        .with_max_level(Level::TRACE)
        // completes the builder.
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let storage = TempStorage::new().await?.apply_default_genesis().await?;
    let mut state = Arc::new(StateDelta::new(storage.latest_snapshot()));
    let mut state_tx = state.try_begin_transaction().unwrap();
    let gn = asset::REGISTRY.parse_unit("gn");
    let penumbra = asset::REGISTRY.parse_unit("penumbra");

    let mut pair = DirectedTradingPair::new(gn.id(), penumbra.id());
    let position_1 = Position::new(
        OsRng,
        pair,
        1,
        1u64.into(),
        100u64.into(),
        Reserves {
            r1: 0u64.into(),
            r2: 10_000u64.into(),
        },
    );

    let position_2 = Position::new(
        OsRng,
        pair,
        0,
        1u64.into(),
        101u64.into(),
        Reserves {
            r1: 10_001u64.into(),
            r2: 0u64.into(),
        },
    );
    state_tx.put_position(position_1.clone());
    state_tx.put_position(position_2.clone());

    let positions = state_tx
        .positions_by_price(&pair)
        .then(|result| async {
            let id = result.unwrap();
            let position = state_tx.position_by_id(&id).await.unwrap().unwrap();
            position
        })
        .collect::<Vec<position::Position>>()
        .await;

    assert_eq!(positions.len(), 1);
    assert_eq!(positions[0].reserves.r1, position_1.reserves.r1);
    assert_eq!(positions[0].reserves.r2, position_1.reserves.r2);
    assert_eq!(positions[0].phi, position_1.phi);
    assert_eq!(positions[0].nonce, position_1.nonce);

    let pair = pair.flip();

    let positions = state_tx
        .positions_by_price(&pair)
        .then(|result| async {
            let id = result.unwrap();
            let position = state_tx.position_by_id(&id).await.unwrap().unwrap();
            position
        })
        .collect::<Vec<position::Position>>()
        .await;

    assert_eq!(positions.len(), 1);
    assert_eq!(positions[0].reserves.r1, position_2.reserves.r1);
    assert_eq!(positions[0].reserves.r2, position_2.reserves.r2);
    assert_eq!(positions[0].phi, position_2.phi);
    assert_eq!(positions[0].nonce, position_2.nonce);
    Ok(())
}

#[tokio::test]
async fn test_multiple_similar_position() -> anyhow::Result<()> {
    let storage = TempStorage::new().await?.apply_default_genesis().await?;
    let mut state = Arc::new(StateDelta::new(storage.latest_snapshot()));
    let mut state_tx = state.try_begin_transaction().unwrap();

    let gm = asset::REGISTRY.parse_unit("gm");
    let gn = asset::REGISTRY.parse_unit("gn");

    let pair_1 = Market::new(gm.clone(), gn.clone());

    let one = 1u64.into();
    let mut buy_1 = limit_buy(pair_1.clone(), 1u64.into(), one);
    let mut buy_2 = limit_buy(pair_1.clone(), 1u64.into(), one);
    buy_1.nonce = [1u8; 32];
    buy_2.nonce = [2u8; 32];
    state_tx.put_position(buy_1.clone());
    state_tx.put_position(buy_2.clone());

    let mut p_1 = state_tx
        .best_position(&pair_1.into_directed_trading_pair())
        .await
        .unwrap()
        .expect("we just posted two positions");
    assert_eq!(p_1.nonce, buy_1.nonce);
    p_1.reserves = p_1.reserves.flip();
    state_tx.put_position(p_1);

    let mut p_2 = state_tx
        .best_position(&pair_1.into_directed_trading_pair())
        .await
        .unwrap()
        .expect("there is one position remaining");
    assert_eq!(p_2.nonce, buy_2.nonce);
    p_2.reserves = p_2.reserves.flip();
    state_tx.put_position(p_2);

    let p_3 = assert!(state_tx
        .best_position(&pair_1.into_directed_trading_pair())
        .await
        .unwrap()
        .is_none());
    Ok(())
}

#[tokio::test]
async fn fill_route_constraint_1() -> anyhow::Result<()> {
    // tracing_subscriber::fmt().try_init().unwrap();

    use tracing::{info, Level};
    use tracing_subscriber::FmtSubscriber;
    let subscriber = FmtSubscriber::builder()
        // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
        // will be written to stdout.
        .with_max_level(Level::TRACE)
        // completes the builder.
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let storage = TempStorage::new().await?.apply_default_genesis().await?;
    let mut state = Arc::new(StateDelta::new(storage.latest_snapshot()));
    let mut state_tx = state.try_begin_transaction().unwrap();
    /*
            ------------------------------------------------------------------------------------------------------------
            |       Pair 1: gm <> gn       |       Pair 2: gn <> penumbra        |       Pair 3: penumbra <> pusd      |
            ------------------------------------------------------------------------------------------------------------
            |       100gm@1                |       5300gn@92                     |       15penumbra@1445             |
            |       120gm@1                |       54gn@100                      |       1penumbra@1450               |
            |       50gm@1                 |       55gn@100                      |                                     |
            | ^-bids---------asks-v        |   ^-bids---------asks-v             |   ^-bids---------asks-v             |
            |       10gn@1                 |       54gn@101                      |       5penumbra@1500             |
            |       100gn@1                |       1000gn@102                    |       1penumbra@1550                |
            |       50gn@1                 |                                     |                                     |
            ------------------------------------------------------------------------------------------------------------
    */

    let gm = asset::REGISTRY.parse_unit("gm");
    let gn = asset::REGISTRY.parse_unit("gn");
    let penumbra = asset::REGISTRY.parse_unit("penumbra");
    let pusd = asset::REGISTRY.parse_unit("pusd");

    let pair_1 = Market::new(gm.clone(), gn.clone());
    let pair_2 = Market::new(gn.clone(), penumbra.clone());
    let pair_3 = Market::new(penumbra.clone(), pusd.clone());

    /*
     * pair 1: gm <> gn
                100gm@1
                120gm@1
                50gm@1
            ^-bids---------asks-v
                10gn@1
                100gn@1
                50gn@1
    */
    let one = 1u64.into();

    let buy_1 = limit_buy(pair_1.clone(), 50u64.into(), one);
    let buy_2 = limit_buy(pair_1.clone(), 120u64.into(), one);
    let buy_3 = limit_buy(pair_1.clone(), 100u64.into(), one);

    let sell_1 = limit_sell(pair_1.clone(), 10u64.into(), one);
    let sell_2 = limit_sell(pair_1.clone(), 100u64.into(), one);
    let sell_3 = limit_sell(pair_1.clone(), 50u64.into(), one);

    state_tx.put_position(buy_1);
    state_tx.put_position(buy_2);
    state_tx.put_position(buy_3);

    state_tx.put_position(sell_1);
    state_tx.put_position(sell_2);
    state_tx.put_position(sell_3);

    /*

    * pair 2: gn <> penumbra
         5300gn@92
         54gn@100
         55gn@100
     ^-bids---------asks-v
          54gn@101
          1000gn@102

    */
    let price100 = 100u64.into();
    let price101 = 101u64.into();
    let price102 = 102u64.into();
    let price92 = 92u64.into();

    let buy_1 = limit_buy(pair_2.clone(), 55u64.into(), price100);
    let buy_2 = limit_buy(pair_2.clone(), 54u64.into(), price100);
    let buy_3 = limit_buy(pair_2.clone(), 5300u64.into(), price92);

    let sell_1 = limit_sell(pair_2.clone(), 54u64.into(), price101);
    let sell_2 = limit_sell(pair_2.clone(), 1000u64.into(), price102);

    state_tx.put_position(buy_1);
    state_tx.put_position(buy_2);
    state_tx.put_position(buy_3);

    state_tx.put_position(sell_1);
    state_tx.put_position(sell_2);

    /*
    * pair 3: penumbra <> pusd
            1500penumbra@1445
            10penumbra@1450
        ^-bids---------asks-v
            1penumbra@1500
            10penumbra@1550
            100penumbra@1800
    */
    let price1445 = 1445u64.into();
    let price1450 = 1450u64.into();
    let price1500 = 1500u64.into();
    let price1550 = 1550u64.into();
    let price1800 = 1800u64.into();

    let buy_1 = limit_buy(pair_3.clone(), 1u64.into(), price1450);
    let buy_2 = limit_buy(pair_3.clone(), 1u64.into(), price1445);

    let sell_1 = limit_sell(pair_3.clone(), 1u64.into(), price1500);
    let sell_2 = limit_sell(pair_3.clone(), 1u64.into(), price1550);
    let sell_3 = limit_sell(pair_3.clone(), 1u64.into(), price1800);

    state_tx.put_position(buy_1);
    state_tx.put_position(buy_2);

    state_tx.put_position(sell_1);
    state_tx.put_position(sell_2);
    state_tx.put_position(sell_3);

    /*

       Fill route scratchpad:

    */

    let delta_1 = Value {
        asset_id: gm.id(),
        amount: Amount::from(1u64) * gm.unit_amount(),
    };

    let route = vec![gm.id(), gn.id(), penumbra.id(), pusd.id()];

    let spill_price = U128x128::from(1_000_000u64);

    let (unfilled, output) = FillRoute::fill_route(&mut state_tx, delta_1, &route, spill_price)
        .await
        .unwrap();

    assert_eq!(unfilled.asset_id, gm.id());
    assert_eq!(unfilled.amount, Amount::zero());

    assert_eq!(output.asset_id, pusd.id());
    assert_eq!(output.amount, 3u64.into());

    Ok(())
}

#[tokio::test]
async fn fill_route_unconstrained() -> anyhow::Result<()> {
    // tracing_subscriber::fmt().try_init().unwrap();

    use tracing::{info, Level};
    use tracing_subscriber::FmtSubscriber;
    let subscriber = FmtSubscriber::builder()
        // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
        // will be written to stdout.
        .with_max_level(Level::TRACE)
        // completes the builder.
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let storage = TempStorage::new().await?.apply_default_genesis().await?;
    let mut state = Arc::new(StateDelta::new(storage.latest_snapshot()));
    let mut state_tx = state.try_begin_transaction().unwrap();
    /*
            ------------------------------------------------------------------------------------------------------------
            |       Pair 1: gm <> gn       |       Pair 2: gn <> penumbra        |       Pair 3: penumbra <> pusd      |
            ------------------------------------------------------------------------------------------------------------
            |                              |                                     |                                     |
            | ^-bids---------asks-v        |   ^-bids---------asks-v             |   ^-bids---------asks-v             |
            |        1gm@1                 |          1gn@2                      |         1penumbra@1500              |
            |        1gm@1                 |          1gn@2                      |         1penumbra@1500              |
            |                              |                                     |         1penumbra@1500              |
            |                              |                                     |         1penumbra@1500              |
            |                              |                                     |         1penumbra@1500              |
            ------------------------------------------------------------------------------------------------------------
    */

    let gm = asset::REGISTRY.parse_unit("gm");
    let gn = asset::REGISTRY.parse_unit("gn");
    let penumbra = asset::REGISTRY.parse_unit("penumbra");
    let pusd = asset::REGISTRY.parse_unit("pusd");

    let pair_1 = Market::new(gm.clone(), gn.clone());
    let pair_2 = Market::new(gn.clone(), penumbra.clone());
    let pair_3 = Market::new(penumbra.clone(), pusd.clone());

    let one = 1u64.into();
    let buy_1 = limit_buy(pair_1.clone(), 1u64.into(), one);
    let buy_2 = limit_buy(pair_1.clone(), 1u64.into(), one);
    state_tx.put_position(buy_1);
    state_tx.put_position(buy_2);

    let price2 = 2u64.into();
    let buy_1 = limit_buy(pair_2.clone(), 1u64.into(), price2);
    let buy_2 = limit_buy(pair_2.clone(), 1u64.into(), price2);
    state_tx.put_position(buy_1);
    state_tx.put_position(buy_2);

    let price1500 = 1500u64.into();
    let buy_1 = limit_buy(pair_3.clone(), 1u64.into(), price1500);
    let buy_2 = limit_buy(pair_3.clone(), 1u64.into(), price1500);
    let buy_3 = limit_buy(pair_3.clone(), 1u64.into(), price1500);
    let buy_4 = limit_buy(pair_3.clone(), 1u64.into(), price1500);
    let buy_5 = limit_buy(pair_3.clone(), 1u64.into(), price1500);
    state_tx.put_position(buy_1);
    state_tx.put_position(buy_2);
    state_tx.put_position(buy_3);
    state_tx.put_position(buy_4);
    state_tx.put_position(buy_5);

    let delta_1 = Value {
        asset_id: gm.id(),
        amount: Amount::from(1u64) * gm.unit_amount(),
    };

    let route = vec![gm.id(), gn.id(), penumbra.id(), pusd.id()];

    let spill_price =
        (U128x128::from(1_000_000_000_000u64) * U128x128::from(pusd.unit_amount())).unwrap();

    let (unfilled, output) = FillRoute::fill_route(&mut state_tx, delta_1, &route, spill_price)
        .await
        .unwrap();

    let desired_output = Amount::from(3000u64) * pusd.unit_amount();

    assert_eq!(unfilled.amount, Amount::zero());
    assert_eq!(unfilled.asset_id, gm.id());
    assert_eq!(output.amount, desired_output);
    assert_eq!(output.asset_id, pusd.id());

    Ok(())
}
}
