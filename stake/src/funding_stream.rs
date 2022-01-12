use penumbra_crypto::Address;
use penumbra_proto::{stake as pb, Protobuf};
use serde::{Deserialize, Serialize};

/// A destination for a portion of a validator's commission of staking rewards.
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone, Copy)]
#[serde(try_from = "pb::FundingStream", into = "pb::FundingStream")]
pub struct FundingStream {
    /// The destinatination address for the funding stream..
    pub address: Address,

    /// The portion (in terms of [basis points](https://en.wikipedia.org/wiki/Basis_point)) of the
    /// validator's total staking reward that goes to this funding stream.
    pub rate_bps: u16,
}

impl Protobuf for FundingStream {
    type Protobuf = pb::FundingStream;
}

impl From<FundingStream> for pb::FundingStream {
    fn from(fs: FundingStream) -> Self {
        pb::FundingStream {
            address: fs.address.to_string(),
            rate_bps: fs.rate_bps as u32,
        }
    }
}

impl TryFrom<pb::FundingStream> for FundingStream {
    type Error = anyhow::Error;

    fn try_from(fs: pb::FundingStream) -> Result<Self, Self::Error> {
        let rate_bps = if fs.rate_bps <= 10_000 {
            fs.rate_bps as u16
        } else {
            return Err(anyhow::anyhow!(
                "rate_bps {} is more than 100%",
                fs.rate_bps
            ));
        };

        Ok(FundingStream {
            address: fs.address.parse()?,
            rate_bps,
        })
    }
}
