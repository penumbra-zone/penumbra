mod test {
    use penumbra_chain::StateWriteExt;
    use penumbra_crypto::{
        asset,
        dex::{
            lp::{position, Reserves, TradingFunction},
            swap::SwapPlaintext,
            DirectedTradingPair,
        },
        transaction::Fee,
        Address, Amount,
    };
    use penumbra_storage::{ArcStateDeltaExt, StateDelta, TempStorage};
    use penumbra_transaction::{action::PositionOpen, plan::SwapPlan, Transaction};

    use penumbra_chain::test_keys;

    use crate::{dex::Dex, TempStorageExt};
    use tendermint::abci;

    use crate::action_handler::ActionHandler;
    use crate::dex::position_manager::PositionManager;
    use crate::dex::position_manager::PositionRead;
    use crate::shielded_pool::ShieldedPool;
    use crate::Component;
    use rand_core::OsRng;
    use std::sync::Arc;

    /*
     *
     * We want to test:
     *
     *
     * fee at 100% -> outputs are zero
     * fee at 0% -> outputs are untouched
     * layered: AB_AB
     * stacked: AB_BA
     * regular: A_A executed once
     * */

    #[tokio::test]
    /// Builds a simple order book with a single limit order, and tests different
    /// market executions against it.
    async fn single_limit_order() -> anyhow::Result<()> {
        let storage = TempStorage::new().await?.apply_default_genesis().await?;
        let mut state = Arc::new(StateDelta::new(storage.latest_snapshot()));
        let mut state_tx = state.try_begin_transaction().unwrap();
        let height = 1;

        // `BeginBlock`.
        state_tx.put_block_height(height);

        let gm = asset::REGISTRY.parse_unit("gm");
        let gn = asset::REGISTRY.parse_unit("gn");

        let pair = DirectedTradingPair::new(gm.id(), gn.id());

        /* position_1: Limit Buy 100gm@1.2gn */
        let reserves = Reserves {
            r1: 0u64.into(),
            r2: 120_000u64.into(),
        };

        let phi = TradingFunction::new(
            pair.into(),
            0u32.into(),
            1_200_000u64.into(),
            1_000_000u64.into(),
        );

        let position_1 = position::Metadata {
            reserves,
            position: position::Position::new(OsRng, phi),
            state: position::State::Opened,
        };

        let position_1_id = position_1.position.id();
        // replace this with transactions?
        state_tx.put_position(position_1.clone());

        // Scenario 1: A single limit order quotes asset 2
        // We execute four swaps against this order and verify that:
        //      - reserves are updated accurately immediately after execution
        //
        //      Test 1:
        //          * swap_1: fills the entire order
        //          -> reserves are updated correctly
        //      Test 2:
        //          * swap_1: try to fill the entire order
        //          -> reserves are updated correctly
        //          * swap_2: clone of swap 1
        //          -> fails (no liquidity)
        //      Test 3:
        //          * swap_1: fills entire order
        //          * swap_2: fills entire order (other direction)
        //          -> reserves are updated correctly
        //      Test 4:
        //          * swap_1: partial fill
        //          -> reserves are updated correctly
        //          * swap_2: clone of swap_1
        //          -> reserves are updated correctly
        //          * swap_3: fills what is left
        //          -> reserves are updated correctly
        //
        //

        // Test 1: We're trying to fill the entire order.
        let delta_1 = penumbra_crypto::Value {
            amount: 100_000u64.into(),
            asset_id: gm.id(),
        };
        let (unfilled, output) = state_tx.fill_against(delta_1, &position_1_id).await?;
        println!("unfilled: {unfilled:?}");
        println!("output: {output:?}");

        let position = state_tx
            .position_by_id(&position_1_id)
            .await
            .unwrap()
            .unwrap();

        println!("fetched position: {position:?}");

        /*
        let (scenario_1, events) = current_state.apply();
        //
        println!("fetched position: {position:?}");
        println!("ABCI events recorded:");
        for event in events.iter() {
            println!("event: {event:?}");
        }
        */

        // Scenario 2: layered
        Ok(())
    }
}
