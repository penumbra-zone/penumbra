use penumbra_proto::{dex as pb, Protobuf};

/// The reserves of a position.
///
/// Like a position, this implicitly treats the trading function as being
/// between assets 1 and 2, without specifying what those assets are, to avoid
/// duplicating data (each asset ID alone is four times the size of the
/// reserves).
#[derive(Debug, Clone)]
pub struct Reserves {
    pub r1: u64,
    pub r2: u64,
}

impl Protobuf<pb::Reserves> for Reserves {}

impl TryFrom<pb::Reserves> for Reserves {
    type Error = anyhow::Error;

    fn try_from(value: pb::Reserves) -> Result<Self, Self::Error> {
        Ok(Self {
            r1: value.r1,
            r2: value.r2,
        })
    }
}

impl From<Reserves> for pb::Reserves {
    fn from(value: Reserves) -> Self {
        Self {
            r1: value.r1,
            r2: value.r2,
        }
    }
}
