use anyhow::Result;
use async_trait::async_trait;
use penumbra_chain::params::ChainParams;
use penumbra_stake::Epoch;
use penumbra_transaction::Transaction;
use tendermint::abci;

use super::{Component, Overlay};
use crate::{components::validator_set::BlockChanges, genesis, WriteOverlayExt};

// Stub component
pub struct Staking {
    overlay: Overlay,

    // TODO: this shouldn't be here -- we should grab this from a common location
    // in the JMT, i.e. genesis/AppState but that's not available until #511 lands.
    // since consensus can't yet change chain params on the fly, it's not an issue,
    // but it definitely will be in the future!
    chain_params: Option<ChainParams>,
}

#[async_trait]
impl Component for Staking {
    async fn new(overlay: Overlay) -> Result<Self> {
        Ok(Self {
            overlay,
            chain_params: None,
        })
    }

    async fn init_chain(&mut self, app_state: &genesis::AppState) -> Result<()> {
        self.chain_params = Some(app_state.chain_params.clone());

        Ok(())
    }

    async fn begin_block(&mut self, begin_block: &abci::request::BeginBlock) -> Result<()> {
        tracing::debug!("Staking: begin_block");
        let block_height = begin_block.header.height;
        let epoch = Epoch::from_height(
            block_height.into(),
            self.chain_params.unwrap().epoch_duration,
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
        todo!()
    }

    async fn check_tx_stateful(&self, _tx: &Transaction) -> Result<()> {
        todo!()
    }

    async fn execute_tx(&mut self, _tx: &Transaction) -> Result<()> {
        todo!()
    }

    async fn end_block(&mut self, _end_block: &abci::request::EndBlock) -> Result<()> {
        todo!()
    }
}
