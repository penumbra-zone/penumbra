use penumbra_keys::Address;
use penumbra_num::Amount;
use penumbra_proto::{penumbra::core::component::stake::v1alpha1 as pb, DomainType};
use serde::{Deserialize, Serialize};

use crate::rate::BaseRateData;

/// A destination for a portion of a validator's commission of staking rewards.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone, Copy)]
#[serde(try_from = "pb::FundingStream", into = "pb::FundingStream")]
pub enum FundingStream {
    ToAddress {
        /// The destinatination address for the funding stream..
        address: Address,

        /// The portion (in terms of [basis points](https://en.wikipedia.org/wiki/Basis_point)) of the
        /// validator's total staking reward that goes to this funding stream.
        rate_bps: u16,
    },
    ToCommunityPool {
        /// The portion (in terms of [basis points](https://en.wikipedia.org/wiki/Basis_point)) of the
        /// validator's total staking reward that goes to this funding stream.
        rate_bps: u16,
    },
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Recipient {
    Address(Address),
    CommunityPool,
}

impl FundingStream {
    pub fn rate_bps(&self) -> u16 {
        match self {
            FundingStream::ToAddress { rate_bps, .. } => *rate_bps,
            FundingStream::ToCommunityPool { rate_bps } => *rate_bps,
        }
    }

    pub fn recipient(&self) -> Recipient {
        match self {
            FundingStream::ToAddress { address, .. } => Recipient::Address(*address),
            FundingStream::ToCommunityPool { .. } => Recipient::CommunityPool,
        }
    }
}

impl FundingStream {
    /// Computes the amount of reward at the epoch specified by base_rate_data
    pub fn reward_amount(
        &self,
        prev_base_rate: &BaseRateData,
        next_base_rate: &BaseRateData,
        total_delegation_tokens: Amount,
    ) -> u64 {
        if prev_base_rate.epoch_index != next_base_rate.epoch_index - 1 {
            panic!("wrong base rate data for previous epoch")
        }
        // take yv*cve*re*psi(e-1)
        let mut r =
            (total_delegation_tokens.value() * (self.rate_bps() as u128 * 1_0000)) / 1_0000_0000;
        r = (r * next_base_rate.base_reward_rate as u128) / 1_0000_0000;
        r = (r * prev_base_rate.base_exchange_rate as u128) / 1_0000_0000;

        r as u64
    }
}

impl DomainType for FundingStream {
    type Proto = pb::FundingStream;
}

impl From<FundingStream> for pb::FundingStream {
    fn from(fs: FundingStream) -> Self {
        pb::FundingStream {
            recipient: match fs {
                FundingStream::ToAddress { address, rate_bps } => Some(
                    pb::funding_stream::Recipient::ToAddress(pb::funding_stream::ToAddress {
                        address: address.to_string(),
                        rate_bps: rate_bps.into(),
                    }),
                ),
                FundingStream::ToCommunityPool { rate_bps } => {
                    Some(pb::funding_stream::Recipient::ToCommunityPool(
                        pb::funding_stream::ToCommunityPool {
                            rate_bps: rate_bps.into(),
                        },
                    ))
                }
            },
        }
    }
}

impl TryFrom<pb::FundingStream> for FundingStream {
    type Error = anyhow::Error;

    fn try_from(fs: pb::FundingStream) -> Result<Self, Self::Error> {
        match fs
            .recipient
            .ok_or_else(|| anyhow::anyhow!("missing funding stream recipient"))?
        {
            pb::funding_stream::Recipient::ToAddress(to_address) => {
                let address = to_address
                    .address
                    .parse()
                    .map_err(|e| anyhow::anyhow!("invalid funding stream address: {}", e))?;
                let rate_bps = to_address
                    .rate_bps
                    .try_into()
                    .map_err(|e| anyhow::anyhow!("invalid funding stream rate: {}", e))?;
                if rate_bps > 10_000 {
                    anyhow::bail!("funding stream rate exceeds 100% (10,000bps)");
                }
                Ok(FundingStream::ToAddress { address, rate_bps })
            }
            pb::funding_stream::Recipient::ToCommunityPool(to_community_pool) => {
                let rate_bps = to_community_pool
                    .rate_bps
                    .try_into()
                    .map_err(|e| anyhow::anyhow!("invalid funding stream rate: {}", e))?;
                if rate_bps > 10_000 {
                    anyhow::bail!("funding stream rate exceeds 100% (10,000bps)");
                }
                Ok(FundingStream::ToCommunityPool { rate_bps })
            }
        }
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
        if funding_streams.iter().map(|fs| fs.rate_bps()).sum::<u16>() > 10_000 {
            anyhow::bail!("sum of funding rates exceeds 100% (10,000bps)");
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

impl<'a> IntoIterator for &'a FundingStreams {
    type Item = &'a FundingStream;
    type IntoIter = std::slice::Iter<'a, FundingStream>;

    fn into_iter(self) -> Self::IntoIter {
        (&self.funding_streams).into_iter()
    }
}
