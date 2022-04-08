use anyhow::Result;
use penumbra_chain::KnownAssets;
use penumbra_proto::light_wallet::{AssetListRequest, ChainParamsRequest};
use tracing::instrument;

use crate::{ClientStateFile, Opt};

#[instrument(skip(opt, state))]
pub async fn assets(opt: &Opt, state: &mut ClientStateFile) -> Result<()> {
    let mut client = opt.light_wallet_client().await?;

    // Update asset registry.
    let request = tonic::Request::new(AssetListRequest {
        chain_id: state.chain_id().unwrap_or_default(),
    });
    let assets: KnownAssets = client.asset_list(request).await?.into_inner().try_into()?;
    for asset in assets.0 {
        state.asset_cache_mut().extend(std::iter::once(asset.denom));
    }

    state.commit()?;
    tracing::info!("updated asset registry");
    Ok(())
}

/// Fetches the global chain parameters and stores them on `ClientState`.
#[instrument(skip(opt, state))]
pub async fn chain_params(opt: &Opt, state: &mut ClientStateFile) -> Result<()> {
    let mut client = opt.light_wallet_client().await?;

    let params = client
        .chain_params(tonic::Request::new(ChainParamsRequest {
            chain_id: state.chain_id().unwrap_or_default(),
        }))
        .await?
        .into_inner()
        .into();

    tracing::info!(?params, "saving chain params");

    *state.chain_params_mut() = Some(params);
    state.commit()?;
    Ok(())
}
