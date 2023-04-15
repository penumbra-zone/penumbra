mod test {
    use anyhow::Ok;
    use penumbra_crypto::{
        asset,
        dex::{
            lp::{position::Position, Reserves},
            DirectedTradingPair,
        },
        fixpoint::U128x128,
        Amount,
    };
    use penumbra_storage::{ArcStateDeltaExt, StateDelta, TempStorage};

    use rand_core::OsRng;

    use penumbra_crypto::Value;

    use crate::dex::position_manager::PositionManager;
    use crate::dex::position_manager::PositionRead;
    use crate::TempStorageExt;
    use futures::StreamExt;
    use std::sync::Arc;

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

    fn limit_buy(pair: DirectedTradingPair, amount: Amount, price: (Amount, Amount)) -> Position {
        Position::new(
            OsRng,
            pair,
            0,
            price.0,
            price.1,
            Reserves {
                r1: amount,
                r2: 0u64.into(),
            },
        )
    }

    fn limit_sell(pair: DirectedTradingPair, amount: Amount, price: (Amount, Amount)) -> Position {
        Position::new(
            OsRng,
            pair,
            0,
            price.1,
            price.0,
            Reserves {
                r1: 0u64.into(),
                r2: amount,
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

        tracing::subscriber::set_global_default(subscriber)
            .expect("setting default subscriber failed");

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

        let mut positions = state_tx
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

        let mut positions = state_tx
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
    async fn test_find_constraint() -> anyhow::Result<()> {
        // tracing_subscriber::fmt().try_init().unwrap();

        use tracing::{info, Level};
        use tracing_subscriber::FmtSubscriber;
        let subscriber = FmtSubscriber::builder()
            // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
            // will be written to stdout.
            .with_max_level(Level::TRACE)
            // completes the builder.
            .finish();

        tracing::subscriber::set_global_default(subscriber)
            .expect("setting default subscriber failed");

        let storage = TempStorage::new().await?.apply_default_genesis().await?;
        let mut state = Arc::new(StateDelta::new(storage.latest_snapshot()));
        let mut state_tx = state.try_begin_transaction().unwrap();

        let gm = asset::REGISTRY.parse_unit("gm");
        let gn = asset::REGISTRY.parse_unit("gn");
        let penumbra = asset::REGISTRY.parse_unit("penumbra");
        let pusd = asset::REGISTRY.parse_unit("pusd");

        println!("gm......: {}", gm.id());
        println!("gn......: {}", gn.id());
        println!("penumbra: {}", penumbra.id());
        println!("pusd....: {}", pusd.id());

        let pair_1 = DirectedTradingPair::new(gm.id(), gn.id());
        let pair_2 = DirectedTradingPair::new(gn.id(), penumbra.id());
        let pair_3 = DirectedTradingPair::new(penumbra.id(), pusd.id());

        /*

                 Pair 1                 Pair 2                Pair 3
                --------------------   -------------------   -------------------
                Bids       Asks         Bids         Asks       Bids                Asks
                --------------------   -------------------   --------------------------
                50gm@1     10gn@1     55gn@100    54gn@101   1penumbra@1450      5000penumbra@1500
                120gm@1    100gn@1    54gn@100    1000gn@102 1500penumbra@1445      1penumbra@1550
                100gm@1    50gn@1     53gn@100

        */

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

        let one = (1u64.into(), 1u64.into());

        let buy_1 = limit_buy(pair_1, 50u64.into(), one);
        let buy_2 = limit_buy(pair_1, 120u64.into(), one);
        let buy_3 = limit_buy(pair_1, 100u64.into(), one);

        let sell_1 = limit_sell(pair_1, 10u64.into(), one);
        let sell_2 = limit_sell(pair_1, 100u64.into(), one);
        let sell_3 = limit_sell(pair_1, 50u64.into(), one);

        state_tx.put_position(buy_1);
        state_tx.put_position(buy_2);
        state_tx.put_position(buy_3);

        state_tx.put_position(sell_1);
        state_tx.put_position(sell_2);
        state_tx.put_position(sell_3);

        /*

        * pair 2: gn <> penumbra
                53gn@100
                54gn@100
                55gn@100
            ^-bids---------asks-v
                 54gn@101
                 1000gn@102


                 */

        let price100 = (100u64.into(), 1u64.into());
        let price101i = (1u64.into(), 101u64.into());
        let price102i = (1u64.into(), 102u64.into());

        let buy_1 = limit_buy(pair_2, 55u64.into(), price100);
        let buy_2 = limit_buy(pair_2, 54u64.into(), price100);
        let buy_3 = limit_buy(pair_2, 53u64.into(), price100);

        let sell_1 = limit_sell(pair_2, 54u64.into(), price101i);
        let sell_2 = limit_sell(pair_2, 1000u64.into(), price102i);

        state_tx.put_position(buy_1);
        state_tx.put_position(buy_2);
        state_tx.put_position(buy_3);

        state_tx.put_position(sell_1);
        state_tx.put_position(sell_2);

        /*

            * pair 3: pusd <> penumbra
                1500penumbra@1445
                10penumbra@1450
            ^-bids---------asks-v
                5penumbra@1500
                1000penumbra@1550

                2023-04-15T02:36:41.342718Z DEBUG put_position{id=plpid1zfuh5n5tqvqevxppw3vcwvd7z248zn5065cz3wg5uxjpa2thm3esyv0f3n}: penumbra_app::dex::position_manager: position=Position { state: Opened, reserves: Reserves { r1: 1000, r2: 0 }, phi: TradingFunction { component: BareTradingFunction { fee: 0, p: 1, q: 102 }, pair: TradingPair { asset_1: passet1984fctenw8m2fpl8a9wzguzp7j34d7vravryuhft808nyt9fdggqxmanqm, asset_2: passet1nupu8yg2kua09ec8qxfsl60xhafp7mmpsjv9pgp50t20hm6pkygscjcqn2 } }, nonce: "a5ef7097a09c31f0e0057058ead299369b7f1845846e2c89a04637f67c067db2" }
        */

        let price1450 = (1450u64.into(), 1u64.into());
        let price1445 = (1445u64.into(), 1u64.into());
        let price1500i = (1u64.into(), 1500u64.into());
        let price1550i = (1u64.into(), 1550u64.into());

        let buy_1 = limit_buy(pair_3, 10u64.into(), price1450);
        let buy_2 = limit_buy(pair_3, 1500u64.into(), price1445);

        let sell_1 = limit_sell(pair_3, 5000u64.into(), price1500i);
        let sell_2 = limit_sell(pair_3, 1u64.into(), price1550i);

        state_tx.put_position(buy_1);
        state_tx.put_position(buy_2);

        state_tx.put_position(sell_1);
        state_tx.put_position(sell_2);

        /*

           Fill route scratchpad:

        */

        let delta_1 = Value {
            asset_id: gm.id(),
            amount: 100_000u64.into(),
        };

        let route = vec![gm.id(), gn.id(), penumbra.id(), pusd.id()];

        let spill_price = U128x128::from(1_000_000u64);

        //state_tx.fill_route(delta_1, route, spill_price).await;
        state_tx.fill_route(input, route, spill_price)
        Ok(())
    }
}
