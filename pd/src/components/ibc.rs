mod client;

use crate::components::Component;
use crate::{genesis, Overlay};
use anyhow::Result;
use async_trait::async_trait;
use client::ClientComponent;
use penumbra_transaction::Transaction;
use tendermint::abci;
use tracing::instrument;

pub struct IBCComponent {
    client: client::ClientComponent,
}

#[async_trait]
impl Component for IBCComponent {
    #[instrument(name = "ibc", skip(overlay))]
    async fn new(overlay: Overlay) -> Result<Self> {
        let client = ClientComponent::new(overlay.clone()).await?;

        Ok(Self { client })
    }

    #[instrument(name = "ibc", skip(self, app_state))]
    async fn init_chain(&mut self, app_state: &genesis::AppState) -> Result<()> {
        self.client.init_chain(app_state).await?;

        Ok(())
    }

    #[instrument(name = "ibc", skip(self, begin_block))]
    async fn begin_block(&mut self, begin_block: &abci::request::BeginBlock) -> Result<()> {
        self.client.begin_block(begin_block).await?;

        Ok(())
    }

    #[instrument(name = "ibc", skip(tx))]
    fn check_tx_stateless(tx: &Transaction) -> Result<()> {
        client::ClientComponent::check_tx_stateless(tx)?;

        Ok(())
    }

    #[instrument(name = "ibc", skip(self, tx))]
    async fn check_tx_stateful(&self, tx: &Transaction) -> Result<()> {
        self.client.check_tx_stateful(tx).await?;

        Ok(())
    }

    #[instrument(name = "ibc", skip(self, tx))]
    async fn execute_tx(&mut self, tx: &Transaction) -> Result<()> {
        self.client.execute_tx(tx).await?;

        Ok(())
    }

    #[instrument(name = "ibc", skip(self, end_block))]
    async fn end_block(&mut self, end_block: &abci::request::EndBlock) -> Result<()> {
        self.client.end_block(end_block).await?;

        Ok(())
    }
}
