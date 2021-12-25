use std::pin::Pin;

use futures::{Future, StreamExt};
use penumbra_proto::light_wallet::{
    light_wallet_client::LightWalletClient, CompactBlockRangeRequest,
};
use tracing::instrument;

impl super::ClientState {
    #[instrument(skip(self, for_each_block), fields(start_height = self.last_block_height()))]
    pub async fn sync<F, Fut>(
        &mut self,
        wallet_uri: String,
        mut for_each_block: F,
    ) -> anyhow::Result<()>
    where
        F: FnMut(bool, u32, &Self) -> Fut,
        Fut: Future<Output = anyhow::Result<()>>,
    {
        tracing::info!("starting wallet sync");
        let mut client = LightWalletClient::connect(wallet_uri).await?;

        let start_height = self.last_block_height().map(|h| h + 1).unwrap_or(0);
        let mut stream = client
            .compact_block_range(tonic::Request::new(CompactBlockRangeRequest {
                start_height,
                end_height: 0,
            }))
            .await?
            .into_inner()
            .peekable();

        while let Some(result) = stream.next().await {
            let block = result?;
            let height = block.height;

            // Update internal state based on the block received
            self.scan_block(block)?;

            // On the final block, prune the timeouts before invoking the callback
            let final_block = Pin::new(&mut stream).peek().await.is_none();
            if final_block {
                self.prune_timeouts();
            }

            // Invoke the callback, passing it a bool indicating whether or not this is the final
            // block, the height of the block, and `&self`, which enables checkpointing the wallet
            // during synchronization in a client-determined way
            for_each_block(final_block, height, self).await?;
        }

        tracing::info!(end_height = ?self.last_block_height().unwrap(), "finished wallet sync");
        Ok(())
    }
}
