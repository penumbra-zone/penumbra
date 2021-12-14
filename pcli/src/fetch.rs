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
        state.add_asset_to_registry(
            asset.asset_id.try_into().map_err(|_| {
                anyhow::anyhow!("could not parse asset ID for denom {}", asset.asset_denom)
            })?,
            asset::REGISTRY
                .parse_base(&asset.asset_denom)
                .ok_or_else(|| anyhow::anyhow!("invalid asset denomination"))?,
        );
    }

    state.commit()?;
    tracing::info!("updated asset registry");
    Ok(())
}
