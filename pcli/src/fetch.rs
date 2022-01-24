use anyhow::Result;
use penumbra_crypto::asset;
use penumbra_proto::{light_wallet::ChainParamsRequest, thin_wallet::AssetListRequest};
use tracing::instrument;

use crate::{ClientStateFile, Opt};

#[instrument(skip(opt, state))]
pub async fn assets(opt: &Opt, state: &mut ClientStateFile) -> Result<()> {
    let mut client = opt.thin_wallet_client().await?;

    // Update asset registry.
    let request = tonic::Request::new(AssetListRequest {
        chain_id: state.chain_id().unwrap_or_default(),
    });
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
