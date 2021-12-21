use anyhow::Result;
use penumbra_crypto::asset;
use penumbra_proto::thin_wallet::{thin_wallet_client::ThinWalletClient, AssetListRequest};
use tracing::instrument;

use crate::ClientStateFile;

#[instrument(skip(state))]
pub async fn assets(state: &mut ClientStateFile, wallet_uri: String) -> Result<()> {
    let mut client = ThinWalletClient::connect(wallet_uri).await?;

    // Update asset registry.
    let request = tonic::Request::new(AssetListRequest {});
    let mut stream = client.asset_list(request).await?.into_inner();
    while let Some(asset) = stream.message().await? {
        state.asset_cache_mut().extend(std::iter::once(
            asset::REGISTRY
                .parse_denom(&asset.asset_denom)
                .ok_or_else(|| {
                    anyhow::anyhow!("invalid asset denomination: {}", asset.asset_denom)
                })?,
        ));
    }

    state.commit()?;
    tracing::info!("updated asset registry");
    Ok(())
}
