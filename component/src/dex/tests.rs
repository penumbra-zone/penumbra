mod test {
    use penumbra_chain::StateWriteExt;
    use penumbra_crypto::{
        asset,
        dex::{
            lp::{position, Reserves},
            DirectedTradingPair,
        },
    };
    use penumbra_storage::{ArcStateDeltaExt, StateDelta, TempStorage};
    use penumbra_transaction::action::PositionOpen;

    use crate::dex::Dex;
    use tendermint::abci;

    use super::PositionManager;
    use crate::dex::position_manager::PositionRead;
    use crate::Component;
    use std::sync::Arc;

    #[tokio::test]
    /// Builds a simple order book and check that positions only get executed once.
    async fn simple_order_book() {
        let storage = TempStorage::new().await.unwrap();
        let mut state = Arc::new(StateDelta::new(storage.latest_snapshot()));
        let height = 1;

        // `BeginBlock`.
        let mut state_tx = state.try_begin_transaction().unwrap();
        state_tx.put_block_height(height);

        let gm = asset::REGISTRY.parse_unit("gm");
        let gn = asset::REGISTRY.parse_unit("gn");

        let pair = DirectedTradingPair::new(gm.id(), gn.id());
        let reserves = Reserves {
            r1: 100_000u64.into(),
            r2: 5_000u64.into(),
        };

        let position = position::Position::new(pair.into(), 1u64.into(), reserves.clone());

        // Positions
        let md_position_1 = position::Metadata {
            reserves,
            position,
            state: position::State::Opened,
        };

        // Create a "simple" orderbook
        let md_position_1_id = md_position_1.position.id();
        state_tx.put_position(md_position_1);
        let end_block = abci::request::EndBlock {
            height: height.try_into().unwrap(),
        };

        Dex::end_block(&mut state_tx, &end_block).await;
        ShieldedPool::end_block(&mut state_tx, &end_block).await;
        let (s, events) = state_tx.apply();
        let position = s.position_by_id(&md_position_1_id).await.unwrap().unwrap();
        println!("fetched position: {position:?}");
        println!("ABCI events recorded:");
        for event in events.iter() {
            println!("event: {event:?}");
        }
    }
}
