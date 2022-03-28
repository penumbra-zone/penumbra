use anyhow::Result;
use async_trait::async_trait;
use penumbra_transaction::Transaction;
use tendermint::abci;

use super::{Component, Overlay};
use crate::genesis;

// Stub component
pub struct ShieldedPool {
    overlay: Overlay,
}

#[async_trait]
impl Component for ShieldedPool {
    async fn new(overlay: Overlay) -> Result<Self> {
        Ok(Self { overlay })
    }

    async fn init_chain(&mut self, _app_state: &genesis::AppState) -> Result<()> {
        todo!()
    }

    async fn begin_block(&mut self, _begin_block: &abci::request::BeginBlock) -> Result<()> {
        todo!()
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
