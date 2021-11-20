use anyhow::Result;
use penumbra_proto::wallet::{wallet_client::WalletClient, CompactBlockRangeRequest};
use tracing::instrument;

use crate::ClientStateFile;

#[instrument(skip(state), fields(start_height = state.last_block_height()))]
pub async fn sync(state: &mut ClientStateFile, wallet_uri: String) -> Result<()> {
    let mut client = WalletClient::connect(wallet_uri).await?;

    let start_height = state.last_block_height().map(|h| h + 1).unwrap_or(0);
    let mut stream = client
        .compact_block_range(tonic::Request::new(CompactBlockRangeRequest {
            start_height,
            end_height: 0,
        }))
        .await?
        .into_inner();

    let mut count = 0;
    while let Some(block) = stream.message().await? {
        state.scan_block(block)?;
        // very basic form of intermediate checkpointing
        count += 1;
        if count % 1000 == 0 {
            state.commit()?;
        }
    }

    tracing::info!("finished sync");
    state.commit()?;
    Ok(())
}
