//! [`TestNode`] interfaces for sending consensus requests to an ABCI application.

use {
    super::TestNode,
    anyhow::{anyhow, Context},
    bytes::Bytes,
    tap::{Tap, TapFallible},
    tendermint::{
        abci::types::CommitInfo,
        block::Header,
        v0_37::abci::{request, response, ConsensusRequest, ConsensusResponse},
    },
    tower::{BoxError, Service},
    tracing::{error, instrument, trace},
};

/// ABCI-related interfaces.
impl<C> TestNode<C>
where
    C: Service<ConsensusRequest, Response = ConsensusResponse, Error = BoxError>
        + Send
        + Clone
        + 'static,
    C::Future: Send + 'static,
    C::Error: Sized,
{
    /// Yields a mutable reference to the consensus service when it is ready to accept a request.
    async fn service(&mut self) -> Result<&mut C, anyhow::Error> {
        use tower::ServiceExt;
        self.consensus
            .ready()
            .tap(|_| trace!("waiting for consensus service"))
            .await
            .tap_err(|error| error!(?error, "failed waiting for consensus service"))
            .map_err(|_| anyhow!("failed waiting for consensus service"))
            .tap_ok(|_| trace!("consensus service is now ready"))
    }

    /// Sends a [`ConsensusRequest::BeginBlock`] request to the ABCI application.
    #[instrument(level = "debug", skip_all)]
    pub async fn begin_block(
        &mut self,
        header: Header,
        last_commit_info: CommitInfo,
    ) -> Result<response::BeginBlock, anyhow::Error> {
        let request = ConsensusRequest::BeginBlock(request::BeginBlock {
            hash: tendermint::Hash::None,
            header,
            last_commit_info,
            byzantine_validators: Default::default(),
        });
        let service = self.service().await?;
        match service
            .tap(|_| trace!("sending BeginBlock request"))
            .call(request)
            .await
            .tap_err(|error| error!(?error, "consensus service returned error"))
            .map_err(|_| anyhow!("consensus service returned error"))?
        {
            ConsensusResponse::BeginBlock(response) => {
                let response::BeginBlock { events } = &response;
                trace!(?events, "received BeginBlock events");
                Ok(response)
            }
            response => {
                error!(?response, "unexpected InitChain response");
                Err(anyhow!("unexpected InitChain response"))
            }
        }
    }

    /// Sends a [`ConsensusRequest::DeliverTx`] request to the ABCI application.
    #[instrument(level = "debug", skip_all)]
    pub async fn deliver_tx(&mut self, tx: Bytes) -> Result<response::DeliverTx, anyhow::Error> {
        let request = ConsensusRequest::DeliverTx(request::DeliverTx { tx });
        let service = self.service().await?;
        match service
            .tap(|_| trace!("sending DeliverTx request"))
            .call(request)
            .await
            .tap_err(|error| error!(?error, "consensus service returned error"))
            .map_err(|_| anyhow!("consensus service returned error"))?
        {
            ConsensusResponse::DeliverTx(response) => {
                let response::DeliverTx {
                    code,
                    gas_used,
                    gas_wanted,
                    events,
                    ..
                } = &response;
                trace!(
                    ?code,
                    ?gas_used,
                    ?gas_wanted,
                    ?events,
                    "received DeliverTx response"
                );
                Ok(response)
            }
            response => {
                error!(?response, "unexpected DeliverTx response");
                Err(anyhow!("unexpected DeliverTx response"))
            }
        }
    }

    /// Sends a [`ConsensusRequest::EndBlock`] request to the ABCI application.
    #[instrument(level = "debug", skip_all)]
    pub async fn end_block(&mut self) -> Result<response::EndBlock, anyhow::Error> {
        let height = self
            .height
            .value()
            .try_into()
            .context("converting height into `i64`")?;
        let request = ConsensusRequest::EndBlock(request::EndBlock { height });

        let service = self.service().await?;
        match service
            .call(request)
            .await
            .tap_err(|error| error!(?error, "consensus service returned error"))
            .map_err(|_| anyhow!("consensus service returned error"))?
        {
            ConsensusResponse::EndBlock(response) => {
                let response::EndBlock {
                    validator_updates,
                    consensus_param_updates,
                    events,
                } = &response;
                trace!(
                    ?validator_updates,
                    ?consensus_param_updates,
                    ?events,
                    "received EndBlock response"
                );
                Ok(response)
            }
            response => {
                error!(?response, "unexpected EndBlock response");
                Err(anyhow!("unexpected EndBlock response"))
            }
        }
    }

    /// Sends a [`ConsensusRequest::Commit`] request to the ABCI application.
    #[instrument(level = "debug", skip_all)]
    pub async fn commit(&mut self) -> Result<response::Commit, anyhow::Error> {
        let request = ConsensusRequest::Commit;
        let service = self.service().await?;
        match service
            .call(request)
            .await
            .tap_err(|error| error!(?error, "consensus service returned error"))
            .map_err(|_| anyhow!("consensus service returned error"))?
        {
            ConsensusResponse::Commit(response) => {
                let response::Commit {
                    data,
                    retain_height,
                } = &response;
                trace!(?data, ?retain_height, "received Commit response");

                Ok(response)
            }
            response => {
                error!(?response, "unexpected Commit response");
                Err(anyhow!("unexpected Commit response"))
            }
        }
    }
}
