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
    let mut state = StateDelta::new(());

    // Write some test positions.
    create_test_positions_basic(&mut state);

    // Create a new path starting at "gm".
    let gm = asset::REGISTRY.parse_unit("gm");
    let mut path = Path::begin(gm.id(), state);

    // Extend the path to "gn".
    let gn = asset::REGISTRY.parse_unit("gn");
    let path = path
        .extend_to(gn.id())
        .await
        .expect("extend_to failed")
        .expect("path to gn not found");

    assert_eq!(path.end(), &gn.id(), "path ends on gn");
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
}

fn create_test_positions_basic<S: StateWrite>(s: &mut S) {
    let gm = asset::REGISTRY.parse_unit("gm");
    let gn = asset::REGISTRY.parse_unit("gn");
    let penumbra = asset::REGISTRY.parse_unit("penumbra");

    let pair = DirectedTradingPair::new(gm.id(), gn.id());
    let pair2 = DirectedTradingPair::new(gn.id(), penumbra.id());

    let reserves_1 = Reserves {
        r1: 0u64.into(),
        r2: 120_000u64.into(),
    };

    // Building positions:

    // 1bps fee from GM <-> GN at 1:1.2
    let position_1 = Position::new(
        OsRng,
        pair,
        1u32,
        1_000_000u64.into(),
        1_200_000u64.into(),
        reserves_1.clone(),
    );
    // 2bps fee from GM <-> GN at 1:1.2
    let position_2 = Position::new(
        OsRng,
        pair,
        2u32,
        1_000_000u64.into(),
        1_200_000u64.into(),
        reserves_1.clone(),
    );
    // 1bps fee from GM <-> GN at 1:2
    let position_3 = Position::new(
        OsRng,
        pair,
        1u32,
        1_000_000u64.into(),
        2_000_000u64.into(),
        reserves_1.clone(),
    );
    // 1bps fee from GN <-> Penumbra at 1:2
    let position_4 = Position::new(
        OsRng,
        pair2,
        1u32,
        1_000_000u64.into(),
        2_000_000u64.into(),
        reserves_1.clone(),
    );

    s.put_position(position_1);
    s.put_position(position_2);
    s.put_position(position_3);
    s.put_position(position_4);
}
