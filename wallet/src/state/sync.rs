use std::pin::Pin;

use futures::{Future, StreamExt};
use penumbra_proto::light_wallet::{
    light_wallet_client::LightWalletClient, CompactBlockRangeRequest,
};
use tracing::instrument;

use super::ClientState;

impl ClientState {
    /// Synchronize the [`ClientState`] with the latest state on the network via the light wallet
    /// protocol.
    ///
    /// This function takes a wallet URI to connect to for receiving the latest state, and a
    /// checkpointing function which can be used to persist intermediate states during the course of
    /// synchronization in case the network connection is interrupted or another error occurs.
    ///
    /// The checkpointing function is called with the current state of the [`ClientState`] after
    /// each block is received, as well as the height of that block and a boolean indicating whether
    /// that block was the final block to be received (i.e. synchronization is complete). It can
    /// determine whether or not to persist the state, and do so in whatever manner is appropriate
    /// to the application.
    ///
    /// You do not have to manually implement [`SyncCheckpoint`] to use this function: it is already
    /// implemented for [`FnMut`] closures taking `(&'a ClientState, u64, bool)` and returning
    /// `Future<Output = anyhow::Result<()>> + Send + 'a`.
    #[instrument(skip(self, checkpoint), fields(start_height = self.last_block_height()))]
    pub async fn sync<F>(&mut self, wallet_uri: String, mut checkpoint: F) -> anyhow::Result<()>
    where
        for<'a> F: SyncCheckpoint<'a>,
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
            let block_height = block.height;

            // Update internal state based on the block received
            self.scan_block(block)?;

            // On the final block, prune the timeouts before invoking the callback
            let is_final_block = Pin::new(&mut stream).peek().await.is_none();
            if is_final_block {
                self.prune_timeouts();
            }

            // Invoke the callback, passing it a bool indicating whether or not this is the final
            // block, the height of the block, and `&self`, which enables checkpointing the wallet
            // during synchronization in a client-determined way
            checkpoint
                .per_block(self, block_height, is_final_block)
                .await?;
        }

        tracing::info!(end_height = ?self.last_block_height().unwrap(), "finished wallet sync");
        Ok(())
    }
}

/// An async checkpointing function to be used as the parameter to [`ClientState::sync`].
///
/// You do not typically need to implement this trait yourself, since it is implemented already for
/// `FnMut` closures of the appropriate type.
pub trait SyncCheckpoint<'a> {
    /// The future returned by the [`SyncCheckpoint::per_block`] function.
    type Future: Future<Output = anyhow::Result<()>> + Send + 'a;

    /// Potentially checkpoint the current client state, in case synchronization is interrupted.
    ///
    /// This function is called after every block is processed with a reference to the client state,
    /// the height of the just-processed block, and a boolean indicating whether the just-processed
    /// block was the final block,.
    ///
    /// Note that an implementation *may* refer to the client state in the returned future, avoiding
    /// the need to clone it in many situations.
    fn per_block(
        &mut self,
        state: &'a ClientState,
        block_height: u32,
        is_final_block: bool,
    ) -> Self::Future;
}

/// Implemennt `SyncCheckpoint` for `FnMut` closures of the appropriate type.
impl<'a, F, U> SyncCheckpoint<'a> for F
where
    F: FnMut(&'a ClientState, u32, bool) -> U,
    U: Future<Output = anyhow::Result<()>> + Send + 'a,
{
    type Future = U;

    fn per_block(
        &mut self,
        state: &'a ClientState,
        block_height: u32,
        is_final_block: bool,
    ) -> Self::Future {
        (self)(state, block_height, is_final_block)
    }
}
