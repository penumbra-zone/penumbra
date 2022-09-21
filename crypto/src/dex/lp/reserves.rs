use crate::asset::Amount;
use penumbra_proto::{dex as pb, Protobuf};

/// The reserves of a position.
///
/// Like a position, this implicitly treats the trading function as being
/// between assets 1 and 2, without specifying what those assets are, to avoid
/// duplicating data (each asset ID alone is four times the size of the
/// reserves).
#[derive(Debug, Clone)]
pub struct Reserves {
    pub r1: Amount,
    pub r2: Amount,
}

impl Protobuf<pb::Reserves> for Reserves {}

impl TryFrom<pb::Reserves> for Reserves {
    type Error = anyhow::Error;

    fn try_from(value: pb::Reserves) -> Result<Self, Self::Error> {
        Ok(Self {
            r1: value
                .r1
                .ok_or_else(|| anyhow::anyhow!("missing r1"))?
                .try_into()?,
            r2: value
                .r2
                .ok_or_else(|| anyhow::anyhow!("missing r2"))?
                .try_into()?,
        })
    }
}

impl From<Reserves> for pb::Reserves {
    fn from(value: Reserves) -> Self {
        Self {
            r1: Some(value.r1.into()),
            r2: Some(value.r2.into()),
        }
    }
}
