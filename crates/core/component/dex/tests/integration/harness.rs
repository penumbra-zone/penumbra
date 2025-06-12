//! This module contains a testing harness which allows us to exercise common dex functionality.
//!
//! The basic flow is that you build up a `TestingStrategy` given a `TestingStrategyBuilder`,
//! and then run this strategy on an empty dex, in order to get an outcome.
//! The strategy involves setting up certain positions, and then performing a batch swap
//! with particular swap flows combined together.
//!
//! The outcome consists of the remaining reserves for each position, and the output for each swapper.
use cnidarium::{Snapshot, StateDelta, TempStorage};
use cnidarium_component::{ActionHandler, Component};
use penumbra_sdk_asset::asset;
use penumbra_sdk_dex::{
    component::{Dex, PositionRead, StateReadExt},
    lp::{
        position::{self, Position},
        Reserves,
    },
    swap::{self, proof::SwapProof, SwapPlaintext},
    BatchSwapOutputData, DirectedTradingPair, PositionClose, PositionOpen, PositionWithdraw, Swap,
    TradingPair,
};
use penumbra_sdk_keys::test_keys;
use penumbra_sdk_num::Amount;
use penumbra_sdk_proto::core::component::dex::v1::ZkSwapProof;
use penumbra_sdk_sct::{
    component::{clock::EpochManager, source::SourceContext},
    epoch::Epoch,
};
use rand_core::{CryptoRngCore, OsRng};
use std::{mem, sync::Arc};
use tendermint::abci::request::EndBlock;

struct DexTester {
    storage: TempStorage,
    height: u8,
    handle: Option<StateDelta<Snapshot>>,
}

impl DexTester {
    async fn init() -> anyhow::Result<Self> {
        let mut out = Self {
            storage: TempStorage::new().await?,
            height: 0,
            handle: None,
        };
        Dex::init_chain(out.handle(), Some(&Default::default())).await;
        Ok(out)
    }

    fn handle(&mut self) -> &mut StateDelta<Snapshot> {
        if let None = self.handle {
            self.handle = Some(self.consume_handle());
        }
        // NO-PANIC: we defined this above.
        self.handle.as_mut().unwrap()
    }

    fn consume_handle(&mut self) -> StateDelta<Snapshot> {
        match mem::replace(&mut self.handle, None) {
            Some(x) => x,
            None => {
                let mut out = StateDelta::new(self.storage.latest_snapshot());
                out.put_mock_source(self.height);
                out.put_block_height(self.height.into());
                out.put_epoch_by_height(
                    self.height.into(),
                    Epoch {
                        index: 0,
                        start_height: 0,
                    },
                );
                out
            }
        }
    }

    async fn position_open(&mut self, tx: PositionOpen) -> anyhow::Result<()> {
        let handle = self.handle();
        tx.check_and_execute(handle).await?;
        Ok(())
    }

    async fn position_close(&mut self, tx: PositionClose) -> anyhow::Result<()> {
        let handle = self.handle();
        tx.check_and_execute(handle).await?;
        Ok(())
    }

    async fn position_withdraw(&mut self, tx: PositionWithdraw) -> anyhow::Result<()> {
        let handle = self.handle();
        tx.check_and_execute(handle).await?;
        Ok(())
    }

    async fn position_by_id(&mut self, id: &position::Id) -> anyhow::Result<Option<Position>> {
        let handle = self.handle();
        handle.position_by_id(id).await
    }

    async fn swap(&mut self, tx: Swap) -> anyhow::Result<()> {
        let handle = self.handle();
        tx.check_and_execute(handle).await?;
        Ok(())
    }

    async fn end_block(&mut self) -> anyhow::Result<()> {
        let handle = self.consume_handle();
        let mut temp_handle = Arc::new(handle);
        Dex::end_block(
            &mut temp_handle,
            &EndBlock {
                height: self.height.into(),
            },
        )
        .await;
        self.storage
            .commit(Arc::into_inner(temp_handle).unwrap())
            .await?;
        self.height += 1;
        Ok(())
    }

    async fn previous_bsod(
        &mut self,
        trading_pair: TradingPair,
    ) -> anyhow::Result<Option<BatchSwapOutputData>> {
        assert!(
            self.height >= 1,
            "did not call `end_block` before calling `previous_bsod`"
        );
        let height = u64::from(self.height - 1);
        let handle = self.handle();
        let bsod = handle.output_data(height, trading_pair).await?;
        Ok(bsod)
    }
}

/// This allows incrementally building up a test strategy.
pub struct TestingStrategyBuilder<'rng> {
    rng: &'rng mut dyn CryptoRngCore,
    strategy: TestingStrategy,
    asset1: asset::Id,
    asset2: asset::Id,
}

impl<'rng> TestingStrategyBuilder<'rng> {
    pub fn new(rng: &'rng mut dyn CryptoRngCore, asset1: asset::Id, asset2: asset::Id) -> Self {
        Self {
            rng,
            strategy: TestingStrategy::new(asset1, asset2),
            asset1,
            asset2,
        }
    }

    pub fn with_position(mut self, reserves1: Amount, reserves2: Amount) -> Self {
        let position = Position::new(
            &mut self.rng,
            DirectedTradingPair::new(self.asset1, self.asset2),
            0,
            1u64.into(),
            1u64.into(),
            Reserves {
                r1: reserves1,
                r2: reserves2,
            },
        );
        self.strategy.positions.push(position);
        self
    }

    pub fn with_swap(mut self, amount: Amount) -> Self {
        let pair = TradingPair::new(self.asset1, self.asset2);
        let plaintext = SwapPlaintext::new(
            &mut OsRng,
            pair,
            if pair.asset_1() == self.asset1 {
                amount
            } else {
                0u64.into()
            },
            if pair.asset_2() == self.asset1 {
                amount
            } else {
                0u64.into()
            },
            Default::default(),
            test_keys::ADDRESS_0.clone(),
        );
        let swap = swap::Body {
            trading_pair: plaintext.trading_pair,
            delta_1_i: plaintext.delta_1_i,
            delta_2_i: plaintext.delta_2_i,
            fee_commitment: plaintext.claim_fee.commit(Default::default()),
            payload: plaintext.encrypt(test_keys::FULL_VIEWING_KEY.outgoing()),
        };
        self.strategy.swaps.push(swap);
        self
    }

    pub fn build(self) -> TestingStrategy {
        self.strategy
    }
}

pub struct TestingStrategy {
    asset1: asset::Id,
    asset2: asset::Id,
    positions: Vec<Position>,
    swaps: Vec<swap::Body>,
}

impl TestingStrategy {
    fn new(asset1: asset::Id, asset2: asset::Id) -> Self {
        Self {
            asset1,
            asset2,
            positions: Vec::new(),
            swaps: Vec::new(),
        }
    }

    /// Run a given strategy, producing a result.
    ///
    /// With the current strategy implementation, this function will:
    /// - create all the positions built up here,
    /// - perform a batch swap with all the flows built up here,
    /// - withdraw and close all positions.
    pub async fn run(self) -> anyhow::Result<TestingOutcome> {
        let mut dex = DexTester::init().await?;

        let mut position_ids = Vec::new();
        for position in self.positions {
            position_ids.push(position.id());
            let tx = PositionOpen {
                position,
                encrypted_metadata: None,
            };
            dex.position_open(tx).await?;
        }
        dex.end_block().await?;

        for body in &self.swaps {
            let swap = Swap {
                proof: SwapProof::try_from(ZkSwapProof {
                    inner: vec![0u8; 192],
                })
                .expect("should be able to create dummy proof"),
                body: body.clone(),
            };
            dex.swap(swap).await?;
        }
        dex.end_block().await?;

        let bsod = dex
            .previous_bsod(TradingPair::new(self.asset1, self.asset2))
            .await?;
        for position_id in position_ids.iter().copied() {
            dex.position_close(PositionClose { position_id }).await?;
        }
        dex.end_block().await?;

        let mut position_reserves = Vec::new();
        for position_id in position_ids {
            let position = dex
                .position_by_id(&position_id)
                .await?
                .expect("position should have been created");

            dex.position_withdraw(PositionWithdraw {
                position_id,
                reserves_commitment: position
                    .reserves
                    .balance(&position.phi.pair)
                    .commit(Default::default()),
                sequence: 0,
            })
            .await?;

            position_reserves.push((
                position
                    .reserves_for(self.asset1)
                    .expect("position should have reserves"),
                position
                    .reserves_for(self.asset2)
                    .expect("position should have reserves"),
            ));
        }
        dex.end_block().await?;

        let mut outputs = Vec::new();
        for swap in self.swaps {
            let bsod = bsod.unwrap();
            let (a, b) = bsod.pro_rata_outputs((swap.delta_1_i, swap.delta_2_i));
            let (a, b) = if bsod.trading_pair.asset_1() == self.asset1 {
                (a, b)
            } else {
                (b, a)
            };
            outputs.push((a, b));
        }

        Ok(TestingOutcome {
            position_reserves,
            outputs,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestingOutcome {
    /// The reserves of each position, in the order of the strategy.
    pub position_reserves: Vec<(Amount, Amount)>,
    /// The outcome of the swap.
    pub outputs: Vec<(Amount, Amount)>,
}
