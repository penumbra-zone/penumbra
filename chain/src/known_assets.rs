use penumbra_crypto::asset::Asset;
use penumbra_proto::{core::chain::v1alpha1 as pb, Protobuf};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(try_from = "pb::KnownAssets", into = "pb::KnownAssets")]
pub struct KnownAssets(pub Vec<Asset>);

impl Protobuf<pb::KnownAssets> for KnownAssets {}

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
