use penumbra_crypto::asset::{self, Asset};
use penumbra_proto::{
    client::v1alpha1::AssetListResponse, core::chain::v1alpha1 as pb, DomainType,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(try_from = "pb::KnownAssets", into = "pb::KnownAssets")]
pub struct KnownAssets(pub Vec<Asset>);

impl DomainType for KnownAssets {
    type Proto = pb::KnownAssets;
}

impl TryFrom<pb::KnownAssets> for KnownAssets {
    type Error = anyhow::Error;
    fn try_from(known_assets: pb::KnownAssets) -> anyhow::Result<Self> {
        Ok(KnownAssets(
            known_assets
                .assets
                .into_iter()
                .map(|asset| asset.try_into())
                .collect::<anyhow::Result<Vec<Asset>>>()?,
        ))
    }
}

impl From<KnownAssets> for pb::KnownAssets {
    fn from(known_assets: KnownAssets) -> Self {
        Self {
            assets: known_assets
                .0
                .into_iter()
                .map(|asset| asset.into())
                .collect(),
        }
    }
}

impl From<KnownAssets> for AssetListResponse {
    fn from(assets: KnownAssets) -> Self {
        Self {
            asset_list: Some(assets.into()),
        }
    }
}

impl From<KnownAssets> for asset::Cache {
    fn from(assets: KnownAssets) -> Self {
        Self::from_iter(assets.0.into_iter().map(|asset| asset.denom))
    }
}
