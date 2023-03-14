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

    use penumbra_crypto::Value;

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
            1_000_000u64.into(),
            1_200_000u64.into(),
        );

        let position_1 = position::Metadata {
            reserves,
            position: position::Position::new(OsRng, phi),
            state: position::State::Opened,
        };

        let position_1_id = position_1.position.id();
        // replace this with transactions?
        state_tx.put_position(position_1.clone());

        let mut state_test_1 = state_tx.fork();

        // Scenario 1: A single limit order quotes asset 2
        // We execute four swaps against this order and verify that:
        //      - reserves are updated accurately immediately after execution
        //
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
        //
        //

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
        // a single limit order for 100gm@1.2gn.

        let position = state_tx
            .position_by_id(&position_1_id)
            .await
            .unwrap()
            .unwrap();

        // We are splitting a single large fill for a `100gm` into, 100 fills for `1gm`.
        for _i in 1..=100 {
            let delta_1 = Value {
                amount: 1000u64.into(),
                asset_id: gm.id(),
            };
            let (unfilled, output) = state_tx.fill_against(delta_1, &position_1_id).await?;
            assert_eq!(
                unfilled,
                Value {
                    amount: Amount::zero(),
                    asset_id: gm.id(),
                }
            );
            assert_eq!(
                output,
                Value {
                    amount: 1200u64.into(),
                    asset_id: gn.id(),
                }
            );
        }

        // Finally, check that the position reserves were updated correctly and the entire order was filled.
        let position = state_tx
            .position_by_id(&position_1_id)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(position.reserves.r1, 100_000u64.into());
        assert_eq!(position.reserves.r2, Amount::zero());

        Ok(())
    }
}
