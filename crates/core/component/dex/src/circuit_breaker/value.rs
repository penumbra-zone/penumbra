use std::collections::BTreeMap;

use penumbra_asset::{asset, Value};
use penumbra_num::Amount;
use penumbra_proto::{core::component::dex::v1alpha1 as pb, DomainType, TypeUrl};

#[derive(Debug, Clone, Default)]
pub struct AssetTallies {
    tallies: BTreeMap<asset::Id, Amount>,
}

impl AssetTallies {
    pub fn tally(&mut self, value: Value) {
        *self.tallies.entry(value.asset_id).or_default() += value.amount;
    }
}

impl From<AssetTallies> for pb::AssetTallies {
    fn from(inner: AssetTallies) -> Self {
        pb::AssetTallies {
            tallies: inner
                .tallies
                .iter()
                .map(|(k, v)| pb::AssetTally {
                    asset_id: Some(
                        Into::<penumbra_proto::core::asset::v1alpha1::AssetId>::into(*k),
                    ),
                    amount: Some(Into::<penumbra_proto::core::num::v1alpha1::Amount>::into(
                        *v,
                    )),
                })
                .collect(),
        }
    }
}

impl TryFrom<pb::AssetTallies> for AssetTallies {
    type Error = anyhow::Error;

    fn try_from(value: pb::AssetTallies) -> Result<Self, Self::Error> {
        Ok(Self {
            tallies: value
                .tallies
                .iter()
                .map(|pb_tally| {
                    (
                        // TODO: remove expects, return results
                        pb_tally
                            .asset_id
                            .clone()
                            .expect("asset ID should be present in tally")
                            .try_into()
                            .expect("invalid protobuf"),
                        pb_tally
                            .amount
                            .clone()
                            .expect("amount should be present in tally")
                            .try_into()
                            .expect("invalid protobuf"),
                    )
                })
                .collect(),
        })
    }
}

impl DomainType for AssetTallies {
    type Proto = pb::AssetTallies;
}
