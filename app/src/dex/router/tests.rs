use std::sync::Arc;

use crate::dex::{router::path::Path, PositionManager};

use penumbra_crypto::{
    asset,
    dex::{
        lp::{position::Position, Reserves},
        DirectedTradingPair,
    },
    Amount,
};
use penumbra_storage::{StateDelta, StateWrite};
use rand_core::OsRng;

use super::PathSearch;

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
    let (path, spill) = state.path_search(gm.id(), penumbra.id(), 4).await.unwrap();

    // Now try routing from "penumbra" to "penumbra".
    tracing::info!(src = %penumbra, dst = %penumbra, "searching for path");
    let (path, spill) = state
        .path_search(penumbra.id(), penumbra.id(), 8)
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
    let mut path = Path::begin(gm.id(), state);

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
    let pusd = asset::REGISTRY.parse_unit("pusd");
    let path = path
        .extend_to(pusd.id())
        .await
        .expect("extend_to failed")
        .expect("path to pusd not found");

    assert_eq!(path.end(), &pusd.id(), "path ends on pusd");
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
        .expect("path to pusd not found")
        .extend_to(penumbra.id())
        .await
        .expect("extend_to failed")
        .expect("path to penumbra not found");

    // This price should be more expensive since the the cheaper path along the mispriced gn:pusd position no longer exists.
    let expensive_price = path.price;

    println!("cheap: {}", cheap_price);
    println!("expensive: {}", expensive_price);
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
    let pusd = asset::REGISTRY.parse_unit("pusd");
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
