// These mock-consensus helper traits aren't consumed just yet
#![allow(dead_code)]

use {
    async_trait::async_trait, cnidarium::TempStorage, penumbra_sdk_mock_consensus::TestNode,
    penumbra_sdk_sct::component::clock::EpochRead as _, tap::Tap,
};

#[async_trait]
pub trait TestNodeExt: Sized {
    async fn fast_forward_to_next_epoch(
        &mut self,
        storage: &TempStorage,
    ) -> anyhow::Result<penumbra_sdk_sct::epoch::Epoch>;
}

#[async_trait]
impl<C> TestNodeExt for TestNode<C>
where
    C: tower::Service<
            tendermint::v0_37::abci::ConsensusRequest,
            Response = tendermint::v0_37::abci::ConsensusResponse,
            Error = tower::BoxError,
        > + Send
        + Clone
        + 'static,
    C::Future: Send + 'static,
    C::Error: Sized,
{
    async fn fast_forward_to_next_epoch(
        &mut self,
        storage: &TempStorage,
    ) -> Result<penumbra_sdk_sct::epoch::Epoch, anyhow::Error> {
        let get_epoch = || async { storage.latest_snapshot().get_current_epoch().await };
        let start = get_epoch()
            .await?
            .tap(|start| tracing::info!(?start, "fast forwarding to next epoch"));

        loop {
            self.block().execute().await?;
            let current = get_epoch().await?;
            if current != start {
                tracing::debug!(end = ?current, ?start, "reached next epoch");
                return Ok(current);
            }
        }
    }
}
