use futures::StreamExt;
use penumbra_crypto::MockFlowCiphertext;
use penumbra_crypto::{asset, fixpoint::U128x128, Amount, Value};
use penumbra_storage::ArcStateDeltaExt;
use penumbra_storage::TempStorage;
use penumbra_storage::{StateDelta, StateWrite};
use rand_core::OsRng;
use std::sync::Arc;

//use crate::temp_storage_ext::TempStorageExt;

use crate::{
    component::{
        router::{FillRoute, HandleBatchSwaps, Path},
        tests::TempStorageExt,
        PositionManager, PositionRead, StateReadExt, StateWriteExt,
    },
    lp::{
        position::{self, Position},
        Reserves,
    },
    DirectedTradingPair, DirectedUnitPair,
};

use super::{PathSearch, RoutingParams};

#[tokio::test(flavor = "multi_thread")]
async fn path_search_basic() {
    let _ = tracing_subscriber::fmt::try_init();
    let mut state = StateDelta::new(());
    create_test_positions_basic(&mut state, true);
    let state = Arc::new(state);

    // Try routing from "gm" to "penumbra".
    let gm = asset::REGISTRY.parse_unit("gm");
    let penumbra = asset::REGISTRY.parse_unit("penumbra");

    tracing::info!(src = %gm, dst = %penumbra, "searching for path");
    let (_path, _spill) = state
        .path_search(
            gm.id(),
            penumbra.id(),
            RoutingParams {
                max_hops: 4,
                ..Default::default()
            },
        )
        .await
        .unwrap();

    // Now try routing from "penumbra" to "penumbra".
    tracing::info!(src = %penumbra, dst = %penumbra, "searching for path");
    let (_path, _spill) = state
        .path_search(
            penumbra.id(),
            penumbra.id(),
            RoutingParams {
                max_hops: 8,
                ..Default::default()
            },
        )
        .await
        .unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn path_extension_basic() {
    let _ = tracing_subscriber::fmt::try_init();
    let mut state = StateDelta::new(());

    // Write some test positions with a mispriced gn:pusd pair.
    create_test_positions_basic(&mut state, true);

    // Create a new path starting at "gm".
    let gm = asset::REGISTRY.parse_unit("gm");
    let path = Path::begin(gm.id(), state);

    // Extend the path to "gn".
    let gn = asset::REGISTRY.parse_unit("gn");
    let mut path = path
        .extend_to(gn.id())
        .await
        .expect("extend_to failed")
        .expect("path to gn not found");

    assert_eq!(path.end(), &gn.id(), "path ends on gn");
    assert_eq!(path.start, gm.id(), "path starts on gm");

    // Extending directly to "penumbra" should fail as there
    // are no positions from GN <-> Penumbra.
    let penumbra = asset::REGISTRY.parse_unit("penumbra");
    assert!(
        path.fork()
            .extend_to(penumbra.id())
            .await
            .expect("extend_to failed")
            .is_none(),
        "path to penumbra should not exist"
    );

    // Extend further to "pusd".
    let pusd = asset::REGISTRY.parse_unit("test_usd");
    let path = path
        .extend_to(pusd.id())
        .await
        .expect("extend_to failed")
        .expect("path to test_usd not found");

    assert_eq!(path.end(), &pusd.id(), "path ends on test_usd");
    assert_eq!(path.start, gm.id(), "path starts on gm");

    // Extend further to "penumbra".
    let penumbra = asset::REGISTRY.parse_unit("penumbra");
    let path = path
        .extend_to(penumbra.id())
        .await
        .expect("extend_to failed")
        .expect("path to penumbra not found");

    assert_eq!(path.end(), &penumbra.id(), "path ends on penumbra");
    assert_eq!(path.start, gm.id(), "path starts on gm");

    // This price should have taken the cheaper path along the mispriced gn:pusd position.
    let cheap_price = path.price;

    // Reset the state.
    let mut state = StateDelta::new(());

    // Write some test positions without the mispriced position.
    create_test_positions_basic(&mut state, false);

    let path = Path::begin(gm.id(), state)
        .extend_to(gn.id())
        .await
        .expect("extend_to failed")
        .expect("path to gn not found")
        .extend_to(pusd.id())
        .await
        .expect("extend_to failed")
        .expect("path to test_usd not found")
        .extend_to(penumbra.id())
        .await
        .expect("extend_to failed")
        .expect("path to penumbra not found");

    // This price should be more expensive since the the cheaper path along the mispriced gn:pusd position no longer exists.
    let expensive_price = path.price;

    assert!(
        cheap_price < expensive_price,
        "price should be cheaper with mispriced position"
    );

    // TODO: ensure best-valued path is taken
    // TODO: test synthetic liquidity
}

fn create_test_positions_basic<S: StateWrite>(s: &mut S, misprice: bool) {
    let gm = asset::REGISTRY.parse_unit("gm");
    let gn = asset::REGISTRY.parse_unit("gn");
    let penumbra = asset::REGISTRY.parse_unit("penumbra");
    let pusd = asset::REGISTRY.parse_unit("test_usd");
    tracing::debug!(id = ?gm.id(), unit = %gm);
    tracing::debug!(id = ?gn.id(), unit = %gn);
    tracing::debug!(id = ?penumbra.id(), unit = %penumbra);
    tracing::debug!(id = ?pusd.id(), unit = %pusd);

    // `pusd` is treated as a numeraire, with gm:pusd, gn:pusd, and penumbra:pusd pairs with different prices.
    // some of the `gn:pusd` positions will be mispriced so we can exercise arbitrage and cycle resolution.
    // routing through `pusd` will let us test synthetic liquidity on the `gm:penumbra` and `gn:penumbra` pairs (which will have
    // no direct positions).
    // ┌──────┐
    // │      │         ┌─────┐
    // │  gm  │◀───┐    │ gn  │
    // │      │    └───▶│     │
    // └──────┘         └─────┘
    //     ▲               ▲
    //     │        ┌──────┘
    //     │        │
    //     │        ▼
    //     │   ┌────────┐      ┌──────────┐
    //     └──▶│  pusd  │◀────▶│ penumbra │
    //         └────────┘      └──────────┘

    // gm <-> gn
    let gm_gn_pair = DirectedTradingPair::new(gm.id(), gn.id());
    // gm <-> pusd
    let gm_pusd_pair = DirectedTradingPair::new(gm.id(), pusd.id());
    // gn <-> pusd
    let gn_pusd_pair = DirectedTradingPair::new(gn.id(), pusd.id());
    // penumbra <-> pusd
    let pen_pusd_pair = DirectedTradingPair::new(penumbra.id(), pusd.id());

    // Exchange rates:
    //
    // GM <-> GN: 1:1
    // GM <-> PUSD: 1:2
    // GN <-> PUSD: 1:2
    // PUSD <-> Penumbra: 10:1
    //
    // Some positions will be mispriced according to the above exchange rates.

    // Building positions:

    // 10bps fee from GM <-> GN at 1:1
    let position_1 = Position::new(
        OsRng,
        gm_gn_pair,
        10,
        // We want a 1:1 ratio of _display_ units, so cross-multiply with the unit<>base ratios:
        Amount::from(1u64) * gn.unit_amount(),
        Amount::from(1u64) * gm.unit_amount(),
        Reserves {
            r1: gm.parse_value("10").unwrap(),
            r2: gn.parse_value("10").unwrap(),
        },
    );
    // 20bps fee from GM <-> GN at 1:1
    let position_2 = Position::new(
        OsRng,
        gm_gn_pair,
        20,
        // We want a 1:1 ratio of _display_ units, so cross-multiply with the unit<>base ratios:
        Amount::from(1u64) * gn.unit_amount(),
        Amount::from(1u64) * gm.unit_amount(),
        Reserves {
            r1: gm.parse_value("20").unwrap(),
            r2: gn.parse_value("30").unwrap(),
        },
    );
    // 50bps fee from GM <-> GN at 1:1
    let position_3 = Position::new(
        OsRng,
        gm_gn_pair,
        50,
        // We want a 1:1 ratio of _display_ units, so cross-multiply with the unit<>base ratios:
        Amount::from(1u64) * gn.unit_amount(),
        Amount::from(1u64) * gm.unit_amount(),
        Reserves {
            r1: gm.parse_value("1000").unwrap(),
            r2: gn.parse_value("1000").unwrap(),
        },
    );
    // 10bps fee from GM <-> PUSD at 1:2
    let position_4 = Position::new(
        OsRng,
        gm_pusd_pair,
        10,
        // We want a 1:2 ratio of _display_ units, so cross-multiply with the unit<>base ratios:
        Amount::from(1u64) * pusd.unit_amount(),
        Amount::from(2u64) * gm.unit_amount(),
        Reserves {
            r1: gm.parse_value("1000").unwrap(),
            r2: pusd.parse_value("1000").unwrap(),
        },
    );
    // 10bps fee from GN <-> PUSD at 1:2
    let position_5 = Position::new(
        OsRng,
        gn_pusd_pair,
        10,
        // We want a 1:2 ratio of _display_ units, so cross-multiply with the unit<>base ratios:
        Amount::from(1u64) * pusd.unit_amount(),
        Amount::from(2u64) * gn.unit_amount(),
        Reserves {
            r1: gn.parse_value("1000").unwrap(),
            r2: pusd.parse_value("1000").unwrap(),
        },
    );
    // MISPRICED: this position has undervalued GN, so it will allow arbitrage.
    // 10bps fee from GN <-> PUSD at 1:1
    let position_6 = Position::new(
        OsRng,
        gn_pusd_pair,
        10,
        // We want a 1:1 ratio of _display_ units, so cross-multiply with the unit<>base ratios:
        Amount::from(1u64) * pusd.unit_amount(),
        Amount::from(1u64) * gn.unit_amount(),
        Reserves {
            r1: gn.parse_value("10").unwrap(),
            r2: pusd.parse_value("10").unwrap(),
        },
    );
    // 1bps fee from Penumbra <-> PUSD at 1:10
    let position_7 = Position::new(
        OsRng,
        pen_pusd_pair,
        1u32,
        // We want a 1:10 ratio of _display_ units, so cross-multiply with the unit<>base ratios:
        Amount::from(1u64) * pusd.unit_amount(),
        Amount::from(10u64) * penumbra.unit_amount(),
        Reserves {
            r1: gn.parse_value("2000").unwrap(),
            r2: pusd.parse_value("2000").unwrap(),
        },
    );
    // 1bps fee from Penumbra <-> PUSD at 1:10
    // We never touch the same position twice during pathfinding, so arbitrage
    // may require multiple positions on the same pair to find the route. In
    // practice this shouldn't be an issue since there will probably be more
    // than 1 person providing liquidity on penumbra.
    let position_8 = Position::new(
        OsRng,
        pen_pusd_pair,
        1u32,
        // We want a 1:10 ratio of _display_ units, so cross-multiply with the unit<>base ratios:
        Amount::from(1u64) * pusd.unit_amount(),
        Amount::from(10u64) * penumbra.unit_amount(),
        Reserves {
            r1: gn.parse_value("2000").unwrap(),
            r2: pusd.parse_value("2000").unwrap(),
        },
    );

    s.put_position(position_1);
    s.put_position(position_2);
    s.put_position(position_3);
    s.put_position(position_4);
    s.put_position(position_5);
    if misprice {
        s.put_position(position_6);
    }
    s.put_position(position_7);
    s.put_position(position_8);
}

/// Create a `Position` to buy `asset_1` using `asset_2` with explicit p/q.
/// e.g. "Buy `quantity` of `asset_1` for `price` units of `asset_2` each.
fn limit_buy_pq(
    market: DirectedUnitPair,
    quantity: Amount,
    p: Amount,
    q: Amount,
    fee: u32,
) -> Position {
    Position::new(
        OsRng,
        market.into_directed_trading_pair(),
        fee,
        p,
        q,
        Reserves {
            r1: Amount::zero(),
            r2: quantity * (q / p) * market.end.unit_amount(),
        },
    )
}

/// Create a `Position` to buy `asset_1` using `asset_2`.
/// e.g. "Buy `quantity` of `asset_1` for `price` units of `asset_2` each.
pub(crate) fn limit_buy(
    market: DirectedUnitPair,
    quantity: Amount,
    price_in_numeraire: Amount,
) -> Position {
    Position::new(
        OsRng,
        market.into_directed_trading_pair(),
        0u32,
        price_in_numeraire * market.end.unit_amount(),
        Amount::from(1u64) * market.start.unit_amount(),
        Reserves {
            r1: Amount::zero(),
            r2: quantity * price_in_numeraire * market.end.unit_amount(),
        },
    )
}

/// Create a `Position` to sell `asset_1` into `asset_2`.
pub(crate) fn limit_sell(
    market: DirectedUnitPair,
    quantity: Amount,
    price_in_numeraire: Amount,
) -> Position {
    Position::new(
        OsRng,
        market.into_directed_trading_pair(),
        0u32,
        price_in_numeraire * market.end.unit_amount(),
        Amount::from(1u64) * market.start.unit_amount(),
        Reserves {
            r1: quantity * market.start.unit_amount(),
            r2: Amount::zero(),
        },
    )
}

#[tokio::test]
/// Test that the best positions are surfaced first.
async fn position_get_best_price() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt::try_init();
    let storage = TempStorage::new().await?.apply_minimal_genesis().await?;
    let mut state = Arc::new(StateDelta::new(storage.latest_snapshot()));
    let mut state_tx = state.try_begin_transaction().unwrap();
    let gn = asset::REGISTRY.parse_unit("gn");
    let penumbra = asset::REGISTRY.parse_unit("penumbra");

    let pair = DirectedTradingPair::new(gn.id(), penumbra.id());
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
/// Test that positions are fetched in-order and that updating reserves
/// deindex them correctly.
async fn test_multiple_similar_position() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt::try_init();
    let storage = TempStorage::new().await?.apply_minimal_genesis().await?;
    let mut state = Arc::new(StateDelta::new(storage.latest_snapshot()));
    let mut state_tx = state.try_begin_transaction().unwrap();

    let gm = asset::REGISTRY.parse_unit("gm");
    let gn = asset::REGISTRY.parse_unit("gn");

    let pair_1 = DirectedUnitPair::new(gm.clone(), gn.clone());

    let one = 1u64.into();
    let price1 = one;
    let mut buy_1 = limit_buy(pair_1.clone(), 1u64.into(), price1);
    let mut buy_2 = limit_buy(pair_1.clone(), 1u64.into(), price1);
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

    assert!(state_tx
        .best_position(&pair_1.into_directed_trading_pair())
        .await
        .unwrap()
        .is_none());
    Ok(())
}

#[tokio::test]
async fn fill_route_constraint_stacked() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt::try_init();
    let storage = TempStorage::new().await?.apply_minimal_genesis().await?;
    let mut state = Arc::new(StateDelta::new(storage.latest_snapshot()));
    let mut state_tx = state.try_begin_transaction().unwrap();
    /*
            ------------------------------------------------------------------------------------------------------------
            |       Pair 1: gm <> gn       |       Pair 2: gn <> penumbra        |       Pair 3: penumbra <> pusd      |
            ------------------------------------------------------------------------------------------------------------
            | ^-bids---------asks-v        |    ^-bids---------asks-v             |       ^-bids---------asks-v        |
            |           3gm@2              |           1gn@2                      |           1penumbra@10000          |
            |           2gm@1              |          50gn@1                      |            1penumbra@3100          |
            |                              |          50gn@1                      |          198penumbra@3000          |
            |                              |          50gn@1                      |           1penumbra@2500           |
            |                              |                                      |           1penumbra@2000           |
            ------------------------------------------------------------------------------------------------------------
            * marginal price
            Delta_1 = 4gm
            Lambda_2 = 2000 + 2500
    */

    let gm = asset::REGISTRY.parse_unit("gm");
    let gn = asset::REGISTRY.parse_unit("gn");
    let penumbra = asset::REGISTRY.parse_unit("penumbra");
    let pusd = asset::REGISTRY.parse_unit("test_usd");

    let pair_1 = DirectedUnitPair::new(gm.clone(), gn.clone());
    let pair_2 = DirectedUnitPair::new(gn.clone(), penumbra.clone());
    let pair_3 = DirectedUnitPair::new(penumbra.clone(), pusd.clone());

    let one: Amount = 1u64.into();

    let price1 = one;
    let price2 = 2u64.into();

    let traces: im::Vector<Vec<Value>> = im::Vector::new();
    state_tx.object_put("trade_traces", traces);

    let buy_1 = limit_buy(pair_1.clone(), 3u64.into(), price2);
    let buy_2 = limit_buy(pair_1.clone(), 1u64.into(), price1);
    state_tx.put_position(buy_1);
    state_tx.put_position(buy_2);

    /* pair 2 */
    let price2 = Amount::from(2u64);

    let buy_1 = limit_buy(pair_2.clone(), 1u64.into(), price2);
    let buy_2 = limit_buy(pair_2.clone(), 50u64.into(), price1);
    let buy_3 = limit_buy(pair_2.clone(), 50u64.into(), price1);
    let buy_4 = limit_buy(pair_2.clone(), 50u64.into(), price1);

    state_tx.put_position(buy_1);
    state_tx.put_position(buy_2);
    state_tx.put_position(buy_3);
    state_tx.put_position(buy_4);

    /* pair 3 */
    let price2000 = 2000u64.into();
    let price2500 = 2500u64.into();
    let price3000 = 3000u64.into();
    let price3100 = 3100u64.into();
    let price10000 = 10_000u64.into();

    let buy_1 = limit_buy(pair_3.clone(), 1u64.into(), price2000);
    let buy_2 = limit_buy(pair_3.clone(), 1u64.into(), price2500);
    let buy_3 = limit_buy(pair_3.clone(), 198u64.into(), price3000);
    let buy_4 = limit_buy(pair_3.clone(), 1u64.into(), price3100);
    let buy_5 = limit_buy(pair_3.clone(), 1u64.into(), price10000);

    state_tx.put_position(buy_1);
    state_tx.put_position(buy_2);
    state_tx.put_position(buy_3);
    state_tx.put_position(buy_4);
    state_tx.put_position(buy_5);

    let delta_1 = Value {
        asset_id: gm.id(),
        amount: Amount::from(4u64) * gm.unit_amount(),
    };

    let route = vec![gn.id(), penumbra.id(), pusd.id()];

    let spill_price = U128x128::from(Amount::from(1_000_000_000u64) * pusd.unit_amount());

    let execution = FillRoute::fill_route(&mut state_tx, delta_1, &route, Some(spill_price))
        .await
        .unwrap();

    let unfilled = delta_1.amount.checked_sub(&execution.input.amount).unwrap();
    let output = execution.output;

    // let output_cal = U128x128::ratio(output.amount, pusd.unit_amount()).unwrap();
    let desired_output: Amount = (Amount::from(10_000u64)
        + Amount::from(3100u64)
        + Amount::from(6u64) * Amount::from(3000u64))
        * pusd.unit_amount();

    assert_eq!(execution.input.asset_id, gm.id());
    assert_eq!(unfilled, Amount::zero());

    assert_eq!(output.asset_id, pusd.id());
    assert_eq!(output.amount, desired_output);

    Ok(())
}

#[tokio::test]
async fn fill_route_constraint_1() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt::try_init();
    let storage = TempStorage::new().await?.apply_minimal_genesis().await?;
    let mut state = Arc::new(StateDelta::new(storage.latest_snapshot()));
    let mut state_tx = state.try_begin_transaction().unwrap();
    /*
            ------------------------------------------------------------------------------------------------------------
            |       Pair 1: gm <> gn       |       Pair 2: gn <> penumbra        |       Pair 3: penumbra <> pusd      |
            ------------------------------------------------------------------------------------------------------------
            | ^-bids---------asks-v        |    ^-bids---------asks-v             |       ^-bids---------asks-v        |
            |         200gm@1              |          50gn@2                      |           1penumbra@10000          |
            |                              |          50gn@2                      |            1penumbra@3100          |
            |                              |          50gn@2                      |          198penumbra@3000          |
            |                              |          50gn@2                      |           1penumbra@2500           |
            |                              |                                      |           1penumbra@2000           |
            ------------------------------------------------------------------------------------------------------------
            Delta_1 = 4gm
            Delta_2 = $0
            Lambda_1 = 0gm
            Lambda_2 = $10,000 + $3100 + 6 * $3000 = $29,100
    */

    let gm = asset::REGISTRY.parse_unit("gm");
    let gn = asset::REGISTRY.parse_unit("gn");
    let penumbra = asset::REGISTRY.parse_unit("penumbra");
    let pusd = asset::REGISTRY.parse_unit("test_usd");

    let pair_1 = DirectedUnitPair::new(gm.clone(), gn.clone());
    let pair_2 = DirectedUnitPair::new(gn.clone(), penumbra.clone());
    let pair_3 = DirectedUnitPair::new(penumbra.clone(), pusd.clone());

    let traces: im::Vector<Vec<Value>> = im::Vector::new();
    state_tx.object_put("trade_traces", traces);

    let one: Amount = 1u64.into();

    let price1 = one;

    let buy_1 = limit_buy(pair_1.clone(), 200u64.into(), price1);
    state_tx.put_position(buy_1);

    /* pair 2 */
    let price2 = Amount::from(2u64);

    let buy_1 = limit_buy(pair_2.clone(), 50u64.into(), price2);
    let buy_2 = limit_buy(pair_2.clone(), 50u64.into(), price2);
    let buy_3 = limit_buy(pair_2.clone(), 50u64.into(), price2);
    let buy_4 = limit_buy(pair_2.clone(), 50u64.into(), price2);

    state_tx.put_position(buy_1);
    state_tx.put_position(buy_2);
    state_tx.put_position(buy_3);
    state_tx.put_position(buy_4);

    /* pair 3 */
    let price2000 = 2000u64.into();
    let price2500 = 2500u64.into();
    let price3000 = 3000u64.into();
    let price3100 = 3100u64.into();
    let price10000 = 10_000u64.into();

    let buy_1 = limit_buy(pair_3.clone(), 1u64.into(), price2000);
    let buy_2 = limit_buy(pair_3.clone(), 1u64.into(), price2500);
    let buy_3 = limit_buy(pair_3.clone(), 198u64.into(), price3000);
    let buy_4 = limit_buy(pair_3.clone(), 1u64.into(), price3100);
    let buy_5 = limit_buy(pair_3.clone(), 1u64.into(), price10000);

    state_tx.put_position(buy_1);
    state_tx.put_position(buy_2);
    state_tx.put_position(buy_3);
    state_tx.put_position(buy_4);
    state_tx.put_position(buy_5);

    let delta_1 = Value {
        asset_id: gm.id(),
        amount: Amount::from(4u64) * gm.unit_amount(),
    };

    let route = vec![gn.id(), penumbra.id(), pusd.id()];

    let spill_price = U128x128::from(Amount::from(1_000_000_000u64) * pusd.unit_amount());

    let execution = FillRoute::fill_route(&mut state_tx, delta_1, &route, Some(spill_price))
        .await
        .unwrap();

    let unfilled = delta_1.amount.checked_sub(&execution.input.amount).unwrap();
    let output = execution.output;

    let desired_output: Amount = (Amount::from(10_000u64)
        + Amount::from(3100u64)
        + Amount::from(6u64) * Amount::from(3000u64))
        * pusd.unit_amount();

    assert_eq!(execution.input.asset_id, gm.id());
    assert_eq!(unfilled, Amount::zero());

    assert_eq!(output.asset_id, pusd.id());
    assert_eq!(output.amount, desired_output);

    Ok(())
}

#[tokio::test]
async fn fill_route_unconstrained() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt::try_init();
    let storage = TempStorage::new().await?.apply_minimal_genesis().await?;
    let mut state = Arc::new(StateDelta::new(storage.latest_snapshot()));
    let mut state_tx = state.try_begin_transaction().unwrap();
    /*
            ------------------------------------------------------------------------------------------------------------
            |       Pair 1: gm <> gn       |       Pair 2: gn <> penumbra        |       Pair 3: penumbra <> pusd      |
            ------------------------------------------------------------------------------------------------------------
            |                              |                                     |                                     |
            | ^-bids---------asks-v        |   ^-bids---------asks-v             |   ^-bids---------asks-v             |
            |        1gm@1                 |          1gn@1                      |         1penumbra@1500              |
            |        1gm@1                 |          1gn@1                      |         1penumbra@1500              |
            |                              |                                     |         1penumbra@1500              |
            |                              |                                     |         1penumbra@1500              |
            |                              |                                     |         1penumbra@1500              |
            ------------------------------------------------------------------------------------------------------------
    */

    let gm = asset::REGISTRY.parse_unit("gm");
    let gn = asset::REGISTRY.parse_unit("gn");
    let penumbra = asset::REGISTRY.parse_unit("penumbra");
    let pusd = asset::REGISTRY.parse_unit("test_usd");

    let pair_1 = DirectedUnitPair::new(gm.clone(), gn.clone());
    let pair_2 = DirectedUnitPair::new(gn.clone(), penumbra.clone());
    let pair_3 = DirectedUnitPair::new(penumbra.clone(), pusd.clone());

    let traces: im::Vector<Vec<Value>> = im::Vector::new();
    state_tx.object_put("trade_traces", traces);

    let one = 1u64.into();
    let price1 = one;
    let buy_1 = limit_buy(pair_1.clone(), 1u64.into(), price1);
    let buy_2 = limit_buy(pair_1.clone(), 1u64.into(), price1);
    state_tx.put_position(buy_1);
    state_tx.put_position(buy_2);

    let buy_1 = limit_buy(pair_2.clone(), 1u64.into(), price1);
    let buy_2 = limit_buy(pair_2.clone(), 1u64.into(), price1);
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

    let route = vec![gn.id(), penumbra.id(), pusd.id()];

    let spill_price =
        (U128x128::from(1_000_000_000_000u64) * U128x128::from(pusd.unit_amount())).unwrap();

    let execution = FillRoute::fill_route(&mut state_tx, delta_1, &route, Some(spill_price))
        .await
        .unwrap();

    let unfilled = delta_1.amount.checked_sub(&execution.input.amount).unwrap();
    let output = execution.output;

    let desired_output = Amount::from(1500u64) * pusd.unit_amount();

    assert_eq!(
        execution.input.asset_id,
        gm.id(),
        "the unfilled asset id is correct"
    );
    assert_eq!(output.asset_id, pusd.id(), "the output asset id is correct");
    assert_eq!(unfilled, Amount::zero(), "there is no unfilled amount");
    assert_eq!(
        output.amount, desired_output,
        "the output amount is correct"
    );

    Ok(())
}

#[tokio::test]
/// Test that we only fill up to the specified spill price.
/// TODO(erwan): stub, fleshing this out later.
async fn fill_route_hit_spill_price() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt::try_init();
    let storage = TempStorage::new().await?.apply_minimal_genesis().await?;
    let mut state = Arc::new(StateDelta::new(storage.latest_snapshot()));
    let mut state_tx = state.try_begin_transaction().unwrap();
    /*
            ------------------------------------------------------------------------------------------------------------
            |       Pair 1: gm <> gn       |       Pair 2: gn <> penumbra        |       Pair 3: penumbra <> pusd      |
            ------------------------------------------------------------------------------------------------------------
            |                              |                                     |                                     |
            | ^-bids---------asks-v        |   ^-bids---------asks-v             |   ^-bids---------asks-v             |
            |        1gm@1                 |          1gn@1                      |         1penumbra@1500              |
            |        2gm@1                 |          1gn@1                      |         1penumbra@1400              |
            |                              |          1gn@1                      |         1penumbra@1300              |
            ------------------------------------------------------------------------------------------------------------
    */

    let gm = asset::REGISTRY.parse_unit("gm");
    let gn = asset::REGISTRY.parse_unit("gn");
    let penumbra = asset::REGISTRY.parse_unit("penumbra");
    let pusd = asset::REGISTRY.parse_unit("test_usd");

    let pair_1 = DirectedUnitPair::new(gm.clone(), gn.clone());
    let pair_2 = DirectedUnitPair::new(gn.clone(), penumbra.clone());
    let pair_3 = DirectedUnitPair::new(penumbra.clone(), pusd.clone());

    let traces: im::Vector<Vec<Value>> = im::Vector::new();
    state_tx.object_put("trade_traces", traces);

    let one = 1u64.into();
    let price1 = one;
    let buy_1 = limit_buy(pair_1.clone(), 1u64.into(), price1);
    let buy_2 = limit_buy(pair_1.clone(), 2u64.into(), price1);
    state_tx.put_position(buy_1);
    state_tx.put_position(buy_2);

    let buy_1 = limit_buy(pair_2.clone(), one, price1);
    let buy_2 = limit_buy(pair_2.clone(), one, price1);
    let buy_3 = limit_buy(pair_2.clone(), one, price1);
    state_tx.put_position(buy_1);
    state_tx.put_position(buy_2);
    state_tx.put_position(buy_3);

    let price1500 = Amount::from(1500u64);
    let price1400 = Amount::from(1400u64);
    let price1300 = Amount::from(1300u64);

    let buy_1 = limit_buy(pair_3.clone(), one, price1500);
    let buy_2 = limit_buy(pair_3.clone(), one, price1400);
    let buy_3 = limit_buy(pair_3.clone(), one, price1300);
    state_tx.put_position(buy_1);
    state_tx.put_position(buy_2);
    state_tx.put_position(buy_3);

    let delta_1 = Value {
        asset_id: gm.id(),
        amount: Amount::from(3u64) * gm.unit_amount(),
    };

    let route = vec![gn.id(), penumbra.id(), pusd.id()];

    let valuation_penumbra =
        (U128x128::from(price1400) * U128x128::from(pusd.unit_amount())).unwrap();
    let valuation_gm = (U128x128::from(one) * U128x128::from(gm.unit_amount())).unwrap();
    let spill_price = U128x128::ratio(valuation_gm, valuation_penumbra).unwrap();

    let execution = FillRoute::fill_route(&mut state_tx, delta_1, &route, Some(spill_price))
        .await
        .unwrap();

    let unfilled = delta_1.amount.checked_sub(&execution.input.amount).unwrap();
    let output = execution.output;

    let desired_output = Amount::from(2900u64) * pusd.unit_amount();

    let one_gm = gm.unit_amount() * one;

    assert_eq!(unfilled, one_gm);
    assert_eq!(execution.input.asset_id, gm.id());
    assert_eq!(output.amount, desired_output);
    assert_eq!(output.asset_id, pusd.id());

    Ok(())
}

#[tokio::test]
/// Test that crafts the positions with the smallest effective price possible
/// and tries to cause an underflow during routing execution.
/// TODO(erwan): stub, fleshing this out later.
async fn fill_route_underflow_effective_price() -> anyhow::Result<()> {
    Ok(())
}

#[tokio::test]
async fn simple_route() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt::try_init();
    let storage = TempStorage::new().await?.apply_minimal_genesis().await?;
    let mut state = Arc::new(StateDelta::new(storage.latest_snapshot()));
    let mut state_tx = state.try_begin_transaction().unwrap();

    let gn = asset::REGISTRY.parse_unit("gn");
    let penumbra = asset::REGISTRY.parse_unit("penumbra");

    let pair_1 = DirectedUnitPair::new(gn.clone(), penumbra.clone());

    // Create a single 1:1 gn:penumbra position (i.e. buy 1 gn at 1 penumbra).
    let buy_1 = limit_buy(pair_1.clone(), 1u64.into(), 1u64.into());
    state_tx.put_position(buy_1);
    state_tx.apply();

    // We should be able to call path_search and route through that position.
    let (path, _spill) = state
        .path_search(
            gn.id(),
            penumbra.id(),
            RoutingParams {
                max_hops: 1,
                ..Default::default()
            },
        )
        .await
        .unwrap();

    assert!(path.is_some(), "path exists between gn<->penumbra");
    assert!(path.clone().unwrap().len() == 1, "path is of length 1");
    assert!(path.unwrap()[0] == penumbra.id(), "path[0] is penumbra");

    Ok(())
}

#[tokio::test]
async fn best_position_route_and_fill() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt::try_init();
    let storage = TempStorage::new().await?.apply_minimal_genesis().await?;
    let mut state = Arc::new(StateDelta::new(storage.latest_snapshot()));
    let mut state_tx = state.try_begin_transaction().unwrap();

    let gn = asset::REGISTRY.parse_unit("gn");
    let penumbra = asset::REGISTRY.parse_unit("penumbra");

    let pair_1 = DirectedUnitPair::new(gn.clone(), penumbra.clone());

    // Create a single 1:1 gn:penumbra position (i.e. buy 1 gn at 1 penumbra).
    let buy_1 = limit_buy(pair_1.clone(), 1u64.into(), 1u64.into());
    state_tx.put_position(buy_1);
    state_tx.apply();

    // We should be able to call path_search and route through that position.
    let (path, _spill) = state
        .path_search(gn.id(), penumbra.id(), RoutingParams::default())
        .await
        .unwrap();

    assert!(path.is_some(), "path exists between gn<->penumbra");
    assert!(path.clone().unwrap().len() == 1, "path is of length 1");
    assert!(path.unwrap()[0] == penumbra.id(), "path[0] is penumbra");

    // Now we should be able to fill a 1:1 gn:penumbra swap.
    let trading_pair = pair_1.into_directed_trading_pair().into();

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
        .handle_batch_swaps(
            trading_pair,
            swap_flow,
            0u32.into(),
            0,
            RoutingParams::default(),
        )
        .await
        .expect("unable to process batch swaps");

    // Output data should have 1 penumbra out and 1 gn in.
    let output_data = state.output_data(0, trading_pair).await?.unwrap();

    // 0 penumbra in
    assert_eq!(output_data.delta_1, 0u64.into());
    // 1 gn in
    assert_eq!(output_data.delta_2, 1u64.into());
    // 0 unfilled penumbra out
    assert_eq!(output_data.unfilled_1, 0u64.into());
    // 0 gn out for penumbra -> gn
    assert_eq!(output_data.lambda_2, 0u64.into());
    // 1 penumbra out for gn -> penumbra
    assert_eq!(output_data.lambda_1, 1u64.into());
    // 0 unfilled gn
    assert_eq!(output_data.unfilled_2, 0u64.into());

    Ok(())
}

#[tokio::test]
async fn multi_hop_route_and_fill() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt::try_init();
    let storage = TempStorage::new().await?.apply_minimal_genesis().await?;
    let mut state = Arc::new(StateDelta::new(storage.latest_snapshot()));
    let mut state_tx = state.try_begin_transaction().unwrap();

    let gm = asset::REGISTRY.parse_unit("gm");
    let gn = asset::REGISTRY.parse_unit("gn");
    let penumbra = asset::REGISTRY.parse_unit("penumbra");

    let pair_gn_penumbra = DirectedUnitPair::new(gn.clone(), penumbra.clone());
    let pair_gm_gn = DirectedUnitPair::new(gm.clone(), gn.clone());
    let pair_gn_gm = DirectedUnitPair::new(gn.clone(), gm.clone());
    let pair_gm_penumbra = DirectedUnitPair::new(gm.clone(), penumbra.clone());

    // Create a 2:1 penumbra:gm position (i.e. buy 20 gm at 2 penumbra each).
    let buy_1 = limit_buy_pq(
        pair_gm_penumbra.clone(),
        5u64.into(),
        1u64.into(),
        2u64.into(),
        0u32,
    );
    state_tx.put_position(buy_1);

    // Create a 2.1:1 penumbra:gm position (i.e. buy 40 gm at 2.1 penumbra each).
    let buy_2 = limit_buy_pq(
        pair_gm_penumbra.clone(),
        40u64.into(),
        1000000u64.into(),
        2100000u64.into(),
        0u32,
    );
    state_tx.put_position(buy_2);

    // Create a 2.2:1 penumbra:gm position (i.e. buy 160 gm at 2.2 penumbra each).
    let buy_3 = limit_buy_pq(
        pair_gm_penumbra.clone(),
        160u64.into(),
        1000000u64.into(),
        2200000u64.into(),
        0u32,
    );
    state_tx.put_position(buy_3);

    // Create a 1:1 gm:gn position (i.e. buy 100 gm at 1 gn each).
    let buy_4 = limit_buy_pq(
        pair_gm_gn.clone(),
        100u64.into(),
        1u64.into(),
        2u64.into(),
        // with 20bps fee
        20u32,
    );
    state_tx.put_position(buy_4);

    // Create a 1.9:1 penumbra:gn position (i.e. buy 160 gn at 1.9 penumbra each).
    let buy_5 = Position::new(
        OsRng,
        pair_gn_penumbra.into_directed_trading_pair(),
        0u32,
        1000000u64.into(),
        1900000u64.into(),
        Reserves {
            r1: Amount::zero(),
            r2: 80000000u32.into(),
        },
    );
    state_tx.put_position(buy_5);

    // Create a 1:1 gm:gn position (i.e. buy 100 gn at 1 gm each).
    let buy_6 = limit_buy_pq(
        pair_gn_gm.clone(),
        100u64.into(),
        1u64.into(),
        1u64.into(),
        // with 20bps fee
        20u32,
    );
    state_tx.put_position(buy_6);

    state_tx.apply();

    // Now if we swap 1000gm into penumbra, we should not get total execution, but we should
    // consume all penumbra liquidity on the direct gm:penumbra pairs, as well as route through the
    // gm:gn and gn:penumbra pairs to obtain penumbra.
    let (path, _spill) = state
        .path_search(gm.id(), penumbra.id(), RoutingParams::default())
        .await
        .unwrap();

    assert!(path.is_some(), "path exists between gm<->penumbra");
    assert!(path.unwrap()[0] == penumbra.id(), "path[0] is penumbra");

    let trading_pair = pair_gm_penumbra.into_directed_trading_pair().into();

    let mut swap_flow = state.swap_flow(&trading_pair);

    assert!(trading_pair.asset_1() == gm.id());

    // Add the amount of each asset being swapped to the batch swap flow.
    swap_flow.0 += MockFlowCiphertext::new(1_000_000_000_000u64.into());
    swap_flow.1 += MockFlowCiphertext::new(0u32.into());

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

    // Output data should have 1 penumbra out and 1000000 gm in.
    let output_data = state.output_data(0, trading_pair).await?.unwrap();

    // 1000000 gm in
    assert_eq!(output_data.delta_1, 1000000000000u64.into());
    // 0 penumbra in
    assert_eq!(output_data.delta_2, 0u64.into());
    // Some gm leftover
    assert!(output_data.unfilled_1 > 0u64.into());

    // Verify all positions that provided `penumbra` have had all their liquidity consumed.
    let mut s = state.all_positions();
    while let Some(position) = s.next().await.transpose()? {
        let trading_pair = position.phi.pair;

        if trading_pair.asset_1() == penumbra.id() {
            assert_eq!(position.reserves.r1, 0u64.into());
        }

        if trading_pair.asset_2() == penumbra.id() {
            assert_eq!(position.reserves.r2, 0u64.into());
        }
    }

    Ok(())
}

#[tokio::test]
/// Reproduce the dust constraint creating `current_input = 0`
async fn fill_dust_route() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt::try_init();
    let storage = TempStorage::new().await?.apply_minimal_genesis().await?;
    let mut state = Arc::new(StateDelta::new(storage.latest_snapshot()));
    let mut state_tx = state.try_begin_transaction().unwrap();

    let gm = asset::REGISTRY.parse_unit("gm");
    let gn = asset::REGISTRY.parse_unit("gn");
    let penumbra = asset::REGISTRY.parse_unit("penumbra");

    let pair_1 = DirectedUnitPair::new(gm.clone(), gn.clone());
    let pair_2 = DirectedUnitPair::new(gn.clone(), penumbra.clone());

    let traces: im::Vector<Vec<Value>> = im::Vector::new();
    state_tx.object_put("trade_traces", traces);

    let one = 1u64.into();
    let price1 = one;
    let buy_1 = limit_buy(pair_1.clone(), 1u64.into(), price1);
    let buy_2 = limit_buy(pair_1.clone(), 1u64.into(), price1);
    state_tx.put_position(buy_1);
    state_tx.put_position(buy_2);
    let dust_constraint = Position::new(
        OsRng,
        pair_2.into_directed_trading_pair(),
        100u32,
        1000000u64.into(),
        3000000u64.into(),
        Reserves {
            r1: 90909090u64.into(),
            r2: 1u64.into(),
        },
    );
    state_tx.put_position(dust_constraint);

    let delta_1 = Value {
        asset_id: gm.id(),
        amount: Amount::from(3u64),
    };

    let route = vec![gn.id(), penumbra.id()];

    let spill_price =
        (U128x128::from(1_000_000_000_000u64) * U128x128::from(penumbra.unit_amount())).unwrap();

    let execution = FillRoute::fill_route(&mut state_tx, delta_1, &route, Some(spill_price))
        .await
        .unwrap();

    let unfilled = delta_1.amount.checked_sub(&execution.input.amount).unwrap();
    let output = execution.output;

    println!("unfilled: {unfilled:?}");
    println!("output: {output:?}");
    Ok(())
}

#[tokio::test]
/// Try filling a route with a dust position.
async fn fill_route_dust() -> () {
    let _ = tracing_subscriber::fmt::try_init();
    let storage = TempStorage::new()
        .await
        .unwrap()
        .apply_minimal_genesis()
        .await
        .unwrap();
    let mut state = Arc::new(StateDelta::new(storage.latest_snapshot()));
    let mut state_tx = state.try_begin_transaction().unwrap();

    let gm = asset::REGISTRY.parse_unit("gm");
    let gn = asset::REGISTRY.parse_unit("gn");
    let penumbra = asset::REGISTRY.parse_unit("penumbra");

    let pair_1 = DirectedUnitPair::new(gm.clone(), gn.clone());
    let pair_2 = DirectedUnitPair::new(gn.clone(), penumbra.clone());

    let traces: im::Vector<Vec<Value>> = im::Vector::new();
    state_tx.object_put("trade_traces", traces);

    let one = 1u64.into();
    let price1 = one;
    let buy_1 = limit_buy(pair_1.clone(), 1u64.into(), price1);
    let buy_2 = limit_buy(pair_1.clone(), 1u64.into(), price1);
    state_tx.put_position(buy_1);
    state_tx.put_position(buy_2);
    let dust_constraint = Position::new(
        OsRng,
        pair_2.into_directed_trading_pair(),
        100u32,
        1000000u64.into(),
        3000000u64.into(),
        Reserves {
            r1: 90909090u64.into(),
            r2: 1u64.into(),
        },
    );
    state_tx.put_position(dust_constraint);

    let delta_1 = Value {
        asset_id: gm.id(),
        amount: Amount::from(3u64),
    };

    let route = vec![gn.id(), penumbra.id()];

    let spill_price =
        (U128x128::from(1_000_000_000_000u64) * U128x128::from(penumbra.unit_amount())).unwrap();

    let execution = FillRoute::fill_route(&mut state_tx, delta_1, &route, Some(spill_price))
        .await
        .unwrap();

    let unfilled = delta_1.amount.checked_sub(&execution.input.amount).unwrap();
    let output = execution.output;

    println!("unfilled: {unfilled:?}");
    println!("output: {output:?}");
}

#[tokio::test]
/// Reproduce dust fill constraint that occurs when a constraint is
/// also a dust position.
async fn fill_route_with_dust_constraint() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt::try_init();
    let storage = TempStorage::new().await?.apply_minimal_genesis().await?;
    let mut state = Arc::new(StateDelta::new(storage.latest_snapshot()));
    let mut state_tx = state.try_begin_transaction().unwrap();

    let gm = asset::REGISTRY.parse_unit("gm");
    let gn = asset::REGISTRY.parse_unit("gn");
    let penumbra = asset::REGISTRY.parse_unit("penumbra");
    let pusd = asset::REGISTRY.parse_unit("test_usd");

    let pair_1 = DirectedUnitPair::new(gm.clone(), gn.clone());
    let pair_2 = DirectedUnitPair::new(gn.clone(), penumbra.clone());
    let pair_3 = DirectedUnitPair::new(penumbra.clone(), pusd.clone());

    let traces: im::Vector<Vec<Value>> = im::Vector::new();
    state_tx.object_put("trade_traces", traces);

    let one = 1u64.into();
    let price1 = one;
    let buy_1 = limit_buy(pair_1.clone(), 1u64.into(), price1);
    let buy_2 = limit_buy(pair_1.clone(), 1u64.into(), price1);
    state_tx.put_position(buy_1);
    state_tx.put_position(buy_2);

    let dust_constraint = Position::new(
        OsRng,
        pair_2.into_directed_trading_pair(),
        100u32,
        1000000u64.into(),
        3000000u64.into(),
        Reserves {
            r1: 90909090u64.into(),
            r2: 1u64.into(),
        },
    );

    let normal_order = Position::new(
        OsRng,
        pair_2.into_directed_trading_pair(),
        150u32,
        Amount::from(1u64) * pair_1.start.unit_amount(),
        Amount::from(3u64) * pair_2.end.unit_amount(),
        Reserves {
            r1: 0u64.into(),
            r2: Amount::from(100u64) * pair_2.end.unit_amount(),
        },
    );

    state_tx.put_position(dust_constraint);
    state_tx.put_position(normal_order);
    let buy_1 = limit_buy(pair_3, 100u64.into(), 1400u64.into());
    state_tx.put_position(buy_1);

    let delta_1 = Value {
        asset_id: gm.id(),
        amount: Amount::from(3u64) * gm.unit_amount(),
    };

    let route = vec![gn.id(), penumbra.id(), pusd.id()];

    let spill_price =
        (U128x128::from(1_000_000_000_000u64) * U128x128::from(penumbra.unit_amount())).unwrap();

    let execution = FillRoute::fill_route(&mut state_tx, delta_1, &route, Some(spill_price))
        .await
        .unwrap();

    let unfilled = delta_1.amount.checked_sub(&execution.input.amount).unwrap();
    let output = execution.output;

    println!("unfilled: {unfilled:?}");
    println!("output: {output:?}");
    Ok(())
}

#[tokio::test]
/// Reproduce dust fill constraint that occurs when a constraint is
/// also a dust position.
async fn fill_route_with_stacked_dust_constraint() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt::try_init();
    let storage = TempStorage::new().await?.apply_minimal_genesis().await?;
    let mut state = Arc::new(StateDelta::new(storage.latest_snapshot()));
    let mut state_tx = state.try_begin_transaction().unwrap();

    let gm = asset::REGISTRY.parse_unit("gm");
    let gn = asset::REGISTRY.parse_unit("gn");
    let penumbra = asset::REGISTRY.parse_unit("penumbra");
    let pusd = asset::REGISTRY.parse_unit("test_usd");
    let btc = asset::REGISTRY.parse_unit("test_btc");

    let pair_1 = DirectedUnitPair::new(gm.clone(), gn.clone());
    let pair_2 = DirectedUnitPair::new(gn.clone(), penumbra.clone());
    let pair_3 = DirectedUnitPair::new(penumbra.clone(), btc.clone());
    let pair_4 = DirectedUnitPair::new(btc.clone(), pusd.clone());

    let traces: im::Vector<Vec<Value>> = im::Vector::new();
    state_tx.object_put("trade_traces", traces);

    let one = 1u64.into();
    let price1 = one;
    let buy_1 = limit_buy(pair_1.clone(), 1u64.into(), price1);
    let buy_2 = limit_buy(pair_1.clone(), 1u64.into(), price1);
    state_tx.put_position(buy_1);
    state_tx.put_position(buy_2);

    let dust_constraint_p2 = Position::new(
        OsRng,
        pair_2.into_directed_trading_pair(),
        100u32,
        1000000u64.into(),
        3000000u64.into(),
        Reserves {
            r1: 90909090u64.into(),
            r2: 1u64.into(),
        },
    );

    let normal_order_p2 = Position::new(
        OsRng,
        pair_2.into_directed_trading_pair(),
        150u32,
        1000000u64.into(),
        3000000u64.into(),
        Reserves {
            r1: 0u64.into(),
            r2: Amount::from(100u64) * pair_2.end.unit_amount(),
        },
    );

    state_tx.put_position(dust_constraint_p2);
    state_tx.put_position(normal_order_p2);

    let dust_constraint_p3 = Position::new(
        OsRng,
        pair_3.into_directed_trading_pair(),
        100u32,
        1000000u64.into(),
        3000000u64.into(),
        Reserves {
            r1: 9090u64.into(),
            r2: 1u64.into(),
        },
    );

    let normal_order_p3 = Position::new(
        OsRng,
        pair_3.into_directed_trading_pair(),
        150u32,
        1000000u64.into(),
        3000000u64.into(),
        Reserves {
            r1: 0u64.into(),
            r2: Amount::from(100u64) * pair_3.end.unit_amount(),
        },
    );

    state_tx.put_position(dust_constraint_p3);
    state_tx.put_position(normal_order_p3);

    let buy_1 = limit_buy(pair_4, 100u64.into(), 1400u64.into());
    state_tx.put_position(buy_1);

    let delta_1 = Value {
        asset_id: gm.id(),
        amount: Amount::from(3u64) * gm.unit_amount(),
    };

    let route = vec![gn.id(), penumbra.id(), btc.id(), pusd.id()];

    let spill_price =
        (U128x128::from(1_000_000_000_000u64) * U128x128::from(penumbra.unit_amount())).unwrap();

    let _execution = FillRoute::fill_route(&mut state_tx, delta_1, &route, Some(spill_price))
        .await
        .unwrap();
    Ok(())
}
