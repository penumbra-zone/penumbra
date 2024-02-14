//! [`TestNode`] interfaces for sending [`Block`]s.

use {
    crate::TestNode,
    tendermint::{
        v0_37::abci::{ConsensusRequest, ConsensusResponse},
        Block,
    },
    tower::{BoxError, Service},
    tracing::{info, instrument},
};

impl<C> TestNode<C>
where
    C: Service<ConsensusRequest, Response = ConsensusResponse, Error = BoxError>
        + Send
        + Clone
        + 'static,
    C::Future: Send + 'static,
    C::Error: Sized,
{
    /// Sends the provided [`Block`] to the consensus service.
    ///
    /// Use [`TestNode::block()`] to build a new block.
    #[instrument(
        level = "info",
        skip_all,
        fields(
            height = %block.header.height,
            time = %block.header.time,
            app_hash = %block.header.app_hash,
        )
    )]
    pub async fn send_block(&mut self, block: Block) -> Result<(), anyhow::Error> {
        let Block {
            header,
            data,
            evidence: _,
            last_commit: _,
            ..
        } = block;

        info!("sending block");
        self.begin_block(header).await?;
        for tx in data {
            let tx = tx.into();
            self.deliver_tx(tx).await?;
        }
        self.end_block().await?;
        self.commit().await?;
        info!("finished sending block");

        Ok(())
    }
}
