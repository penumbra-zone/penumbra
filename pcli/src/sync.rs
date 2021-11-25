use anyhow::Result;

use penumbra_proto::wallet::{
    wallet_client::WalletClient, AssetListRequest, CompactBlockRangeRequest,
};
use tracing::instrument;

use crate::ClientStateFile;

#[instrument(skip(state), fields(start_height = state.last_block_height()))]
pub async fn sync(state: &mut ClientStateFile, wallet_uri: String) -> Result<()> {
    tracing::info!("starting client sync");
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
            tracing::info!(height = ?state.last_block_height().unwrap(), "syncing...");
        }
    }

    // Update asset registry.
    let request = tonic::Request::new(AssetListRequest {});
    let mut stream = client.asset_list(request).await?.into_inner();
    while let Some(asset) = stream.message().await? {
        state.add_asset_to_registry(
            asset.asset_id.try_into().map_err(|_| {
                anyhow::anyhow!("could not parse asset ID for denom {}", asset.asset_denom)
            })?,
            asset.asset_denom.clone(),
        );
    }

    state.commit()?;
    tracing::info!(end_height = ?state.last_block_height().unwrap(), "finished sync");
    Ok(())
}
