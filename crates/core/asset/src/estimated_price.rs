use crate::asset;
use penumbra_sdk_proto::{penumbra::core::asset::v1 as pb, DomainType};
use serde::{Deserialize, Serialize};

/// The estimated price of one asset in terms of another.
///
/// This is used to generate an [`EquivalentValue`](crate::EquivalentValue)
/// that may be helpful in interpreting a [`Value`](crate::Value).
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(try_from = "pb::EstimatedPrice", into = "pb::EstimatedPrice")]

pub struct EstimatedPrice {
    /// The asset that is being priced.
    pub priced_asset: asset::Id,
    /// The numeraire that the price is being expressed in.
    pub numeraire: asset::Id,
    /// Multiply units of the priced asset by this number to get the value in the numeraire.
    ///
    /// This is a floating-point number since the price is approximate.
    pub numeraire_per_unit: f64,
    /// If nonzero, gives some idea of when the price was estimated (in terms of block height).
    pub as_of_height: u64,
}

impl DomainType for EstimatedPrice {
    type Proto = pb::EstimatedPrice;
}

impl From<EstimatedPrice> for pb::EstimatedPrice {
    fn from(msg: EstimatedPrice) -> Self {
        Self {
            priced_asset: Some(msg.priced_asset.into()),
            numeraire: Some(msg.numeraire.into()),
            numeraire_per_unit: msg.numeraire_per_unit,
            as_of_height: msg.as_of_height,
        }
    }
}

impl TryFrom<pb::EstimatedPrice> for EstimatedPrice {
    type Error = anyhow::Error;

    fn try_from(msg: pb::EstimatedPrice) -> Result<Self, Self::Error> {
        Ok(Self {
            priced_asset: msg
                .priced_asset
                .ok_or_else(|| anyhow::anyhow!("missing priced asset"))?
                .try_into()?,
            numeraire: msg
                .numeraire
                .ok_or_else(|| anyhow::anyhow!("missing numeraire"))?
                .try_into()?,
            numeraire_per_unit: msg.numeraire_per_unit,
            as_of_height: msg.as_of_height,
        })
    }
}
