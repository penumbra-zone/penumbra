use penumbra_sdk_asset::{asset, STAKING_TOKEN_ASSET_ID};
use penumbra_sdk_proto::penumbra::core::component::dex::v1 as pb;
use penumbra_sdk_proto::DomainType;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(try_from = "pb::DexParameters", into = "pb::DexParameters")]
/// The configuration parameters for the DEX component.
pub struct DexParameters {
    pub is_enabled: bool,
    pub fixed_candidates: Vec<asset::Id>,
    pub max_hops: u32,
    pub max_positions_per_pair: u32,
    pub max_execution_budget: u32,
}

impl DomainType for DexParameters {
    type Proto = pb::DexParameters;
}

impl TryFrom<pb::DexParameters> for DexParameters {
    type Error = anyhow::Error;

    fn try_from(msg: pb::DexParameters) -> anyhow::Result<Self> {
        Ok(DexParameters {
            is_enabled: msg.is_enabled,
            fixed_candidates: msg
                .fixed_candidates
                .into_iter()
                .map(|id| id.try_into())
                .collect::<Result<_, _>>()?,
            max_hops: msg.max_hops,
            max_positions_per_pair: msg.max_positions_per_pair,
            max_execution_budget: msg.max_execution_budget,
        })
    }
}

impl From<DexParameters> for pb::DexParameters {
    fn from(params: DexParameters) -> Self {
        pb::DexParameters {
            is_enabled: params.is_enabled,
            fixed_candidates: params
                .fixed_candidates
                .into_iter()
                .map(Into::into)
                .collect(),
            max_hops: params.max_hops,
            max_positions_per_pair: params.max_positions_per_pair,
            max_execution_budget: params.max_execution_budget,
        }
    }
}

#[allow(clippy::unwrap_used)]
impl Default for DexParameters {
    fn default() -> Self {
        // This will get used for generating default chain parameters; put some
        // test assets in there.
        let cache = asset::Cache::with_known_assets();
        Self {
            is_enabled: true,
            fixed_candidates: vec![
                *STAKING_TOKEN_ASSET_ID,
                cache.get_unit("test_usd").unwrap().id(),
                cache.get_unit("gm").unwrap().id(),
                cache.get_unit("gn").unwrap().id(),
                cache.get_unit("test_atom").unwrap().id(),
                cache.get_unit("test_osmo").unwrap().id(),
                cache.get_unit("test_btc").unwrap().id(),
            ],
            max_hops: 4,
            max_positions_per_pair: 1_000,
            max_execution_budget: 64,
        }
    }
}
