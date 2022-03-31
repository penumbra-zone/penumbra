use anyhow::Result;
use async_trait::async_trait;
use penumbra_chain::params::ChainParams;
use penumbra_stake::Epoch;
use penumbra_transaction::Transaction;
use tendermint::abci;

use super::{Component, Overlay};
use crate::{components::validator_set::BlockChanges, genesis, PenumbraStore, WriteOverlayExt};

// Stub component
pub struct Staking {
    overlay: Overlay,
}

#[async_trait]
impl Component for Staking {
    async fn new(overlay: Overlay) -> Result<Self> {
        Ok(Self { overlay })
    }

    async fn init_chain(&mut self, app_state: &genesis::AppState) -> Result<()> {
        Ok(())
    }

    async fn begin_block(&mut self, begin_block: &abci::request::BeginBlock) -> Result<()> {
        tracing::debug!("Staking: begin_block");
        let block_height = begin_block.header.height;
        let epoch = Epoch::from_height(
            block_height.into(),
            self.overlay.get_epoch_duration().await?,
        );
        // Reset all staking state in the JMT overlay
        let block_changes = BlockChanges {
            starting_epoch: epoch,
            new_validators: Default::default(),
            updated_validators: Default::default(),
            slashed_validators: Default::default(),
            delegation_changes: Default::default(),
            tm_validator_updates: Default::default(),
            epoch_changes: Default::default(),
        };
        // TODO: need to write proto impl for BlockChanges
        // self.overlay.put_domain(
        //     format!("staking/block_changes/{}", block_height).into(),
        //     block_changes,
        // );

        Ok(())
    }

    fn check_tx_stateless(_tx: &Transaction) -> Result<()> {
        Ok(())
    }

    async fn check_tx_stateful(&self, _tx: &Transaction) -> Result<()> {
        Ok(())
    }

    async fn execute_tx(&mut self, _tx: &Transaction) -> Result<()> {
        Ok(())
    }

    async fn end_block(&mut self, _end_block: &abci::request::EndBlock) -> Result<()> {
        Ok(())
    }
}
