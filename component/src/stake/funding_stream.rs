use penumbra_crypto::Address;
use penumbra_proto::{core::stake::v1alpha1 as pb, Protobuf};
use serde::{Deserialize, Serialize};

use crate::stake::rate::BaseRateData;

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

impl FundingStream {
    /// Computes the amount of reward at the epoch specified by base_rate_data
    pub fn reward_amount(
        &self,
        total_delegation_tokens: u64,
        base_rate_data: &BaseRateData,
        prev_epoch_rate_data: &BaseRateData,
    ) -> u64 {
        if prev_epoch_rate_data.epoch_index != base_rate_data.epoch_index - 1 {
            panic!("wrong base rate data for previous epoch")
        }
        // take yv*cve*re*psi(e-1)
        let mut r =
            (total_delegation_tokens as u128 * (self.rate_bps as u128 * 1_0000)) / 1_0000_0000;
        r = (r * base_rate_data.base_reward_rate as u128) / 1_0000_0000;
        r = (r * prev_epoch_rate_data.base_exchange_rate as u128) / 1_0000_0000;

        r as u64
    }
}

impl Protobuf<pb::FundingStream> for FundingStream {}

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

/// A list of funding streams whose total commission is less than 100%.
///
/// The total commission of a validator is the sum of the individual reward rate of the
/// [`FundingStream`]s, and cannot exceed 10000bps (100%). This property is guaranteed by the
/// `TryFrom<Vec<FundingStream>` implementation for [`FundingStreams`], which checks the sum, and is
/// the only way to build a non-empty [`FundingStreams`].
#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct FundingStreams {
    funding_streams: Vec<FundingStream>,
}

impl FundingStreams {
    pub fn new() -> Self {
        Self {
            funding_streams: Vec::new(),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &FundingStream> {
        self.funding_streams.iter()
    }
}

impl TryFrom<Vec<FundingStream>> for FundingStreams {
    type Error = anyhow::Error;

    fn try_from(funding_streams: Vec<FundingStream>) -> Result<Self, Self::Error> {
        if funding_streams.iter().map(|fs| fs.rate_bps).sum::<u16>() > 10_000 {
            return Err(anyhow::anyhow!(
                "sum of funding rates exceeds 100% (10000bps)"
            ));
        }

        Ok(Self { funding_streams })
    }
}

impl From<FundingStreams> for Vec<FundingStream> {
    fn from(funding_streams: FundingStreams) -> Self {
        funding_streams.funding_streams
    }
}

impl AsRef<[FundingStream]> for FundingStreams {
    fn as_ref(&self) -> &[FundingStream] {
        &self.funding_streams
    }
}

impl IntoIterator for FundingStreams {
    type Item = FundingStream;
    type IntoIter = std::vec::IntoIter<FundingStream>;

    fn into_iter(self) -> Self::IntoIter {
        self.funding_streams.into_iter()
    }
}
