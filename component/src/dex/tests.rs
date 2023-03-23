mod test {
    use penumbra_chain::StateWriteExt;
    use penumbra_crypto::{
        asset,
        dex::{
            lp::{position, Reserves, TradingFunction},
            DirectedTradingPair,
        },
        Amount,
    };
    use penumbra_storage::{ArcStateDeltaExt, StateDelta, TempStorage};

    use rand_core::OsRng;

    use penumbra_crypto::Value;

    use crate::dex::position_manager::PositionManager;
    use crate::dex::position_manager::PositionRead;
    use crate::TempStorageExt;
    use std::sync::Arc;

    #[tokio::test]
    /// Builds a simple order book with a single limit order, and tests different
    /// market order execution against it.
    async fn single_limit_order() -> anyhow::Result<()> {
        let storage = TempStorage::new().await?.apply_default_genesis().await?;
        let mut state = Arc::new(StateDelta::new(storage.latest_snapshot()));
        let mut state_tx = state.try_begin_transaction().unwrap();
        let height = 1;

        state_tx.put_block_height(height);
        state_tx.put_epoch_by_height(
            height,
            penumbra_chain::Epoch {
                index: 0,
                start_height: 0,
            },
        );

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
}
