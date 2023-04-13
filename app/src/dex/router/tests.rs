use crate::dex::{router::path::Path, PositionManager};

use penumbra_crypto::{
    asset,
    dex::{
        lp::{position::Position, Reserves},
        DirectedTradingPair,
    },
};
use penumbra_storage::{StateDelta, StateWrite};
use rand_core::OsRng;

#[tokio::test]
async fn path_extension_basic() {
    let _ = tracing_subscriber::fmt::try_init();
    let mut state = StateDelta::new(());

    // Write some test positions.
    create_test_positions_basic(&mut state);

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
    let pusd = asset::REGISTRY.parse_unit("penumbra");
    let path = path
        .extend_to(pusd.id())
        .await
        .expect("extend_to failed")
        .expect("path to penumbra not found");

    assert_eq!(path.end(), &penumbra.id(), "path ends on penumbra");
    assert_eq!(path.start, gm.id(), "path starts on gm");

    // TODO: ensure best-valued path is taken
    // TODO: test synthetic liquidity
}

fn create_test_positions_basic<S: StateWrite>(s: &mut S) {
    let gm = asset::REGISTRY.parse_unit("gm");
    let gn = asset::REGISTRY.parse_unit("gn");
    let penumbra = asset::REGISTRY.parse_unit("penumbra");
    let pusd = asset::REGISTRY.parse_unit("pusd");

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

    // For basic testing, both sides of the position will have liquidity available.
    // TODO: test positions with only one side of liquidity available (after fill phase is implemented)
    let reserves_1 = Reserves {
        r1: 120_000u64.into(),
        r2: 120_000u64.into(),
    };

    // Exchange rates:
    //
    // GM <-> GN: 1:2
    // GM <-> PUSD: 1:1
    // GN <-> PUSD: 2:1
    // PUSD <-> Penumbra: 1:1
    //
    // Some positions will be mispriced according to the above exchange rates.

    // Building positions:

    // 1bps fee from GM <-> GN at 1:2
    let position_1 = Position::new(
        OsRng,
        gm_gn_pair,
        1u32,
        1_000_000u64.into(),
        2_000_000u64.into(),
        reserves_1.clone(),
    );
    // 2bps fee from GM <-> GN at 1:2
    let position_2 = Position::new(
        OsRng,
        gm_gn_pair,
        2u32,
        1_000_000u64.into(),
        2_000_000u64.into(),
        reserves_1.clone(),
    );
    // 1bps fee from GM <-> GN at 1:2
    let position_3 = Position::new(
        OsRng,
        gm_gn_pair,
        1u32,
        1_000_000u64.into(),
        2_000_000u64.into(),
        reserves_1.clone(),
    );
    // 1bps fee from GM <-> PUSD at 1:1
    let position_4 = Position::new(
        OsRng,
        gm_pusd_pair,
        1u32,
        1_000_000u64.into(),
        1_000_000u64.into(),
        reserves_1.clone(),
    );
    // 1bps fee from GN <-> PUSD at 2:1
    let position_5 = Position::new(
        OsRng,
        gn_pusd_pair,
        1u32,
        2_000_000u64.into(),
        1_000_000u64.into(),
        reserves_1.clone(),
    );
    // MISPRICED: this position has overvalued PUSD, so it will allow arbitrage.
    // 1bps fee from GN <-> PUSD at 3:1
    let position_6 = Position::new(
        OsRng,
        gn_pusd_pair,
        1u32,
        3_000_000u64.into(),
        1_000_000u64.into(),
        reserves_1.clone(),
    );
    // 1bps fee from Penumbra <-> PUSD at 1:1
    let position_7 = Position::new(
        OsRng,
        pen_pusd_pair,
        1u32,
        1_000_000u64.into(),
        1_000_000u64.into(),
        reserves_1.clone(),
    );

    s.put_position(position_1);
    s.put_position(position_2);
    s.put_position(position_3);
    s.put_position(position_4);
    s.put_position(position_5);
    s.put_position(position_6);
    s.put_position(position_7);
}
