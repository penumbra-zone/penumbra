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
    fn new(overlay: Overlay) -> Self {
        Self { overlay }
    }

    fn init_chain(&self, _app_state: &genesis::AppState) {
        todo!()
    }

    async fn begin_block(&self, _begin_block: &abci::request::BeginBlock) {
        todo!()
    }

    fn check_tx_stateless(_tx: &Transaction) -> Result<()> {
        todo!()
    }

    async fn check_tx_stateful(&self, _tx: &Transaction) -> Result<()> {
        todo!()
    }

    async fn execute_tx(&self, _tx: &Transaction) {
        todo!()
    }

    async fn end_block(&self, _end_block: &abci::request::EndBlock) {
        todo!()
    }
}
