use crate::BPS_SQUARED_SCALING_FACTOR;
use penumbra_sdk_keys::Address;
use penumbra_sdk_num::{fixpoint::U128x128, Amount};
use penumbra_sdk_proto::{penumbra::core::component::stake::v1 as pb, DomainType};
use serde::{Deserialize, Serialize};

/// A destination for a portion of a validator's commission of staking rewards.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone)]
#[serde(try_from = "pb::FundingStream", into = "pb::FundingStream")]
pub enum FundingStream {
    ToAddress {
        /// The destination address for the funding stream..
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
#[derive(Debug, PartialEq, Eq, Clone)]
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
            FundingStream::ToAddress { address, .. } => Recipient::Address(address.clone()),
            FundingStream::ToCommunityPool { .. } => Recipient::CommunityPool,
        }
    }
}

impl FundingStream {
    /// Computes the amount of reward at the epoch boundary.
    /// The input rates are assumed to be in basis points squared, this means that
    /// to get the actual rate, you need to rescale by [`BPS_SQUARED_SCALING_FACTOR`].
    pub fn reward_amount(
        &self,
        base_reward_rate: Amount,
        validator_exchange_rate: Amount,
        total_delegation_tokens: Amount,
    ) -> Amount {
        // Setup:
        let total_delegation_tokens = U128x128::from(total_delegation_tokens);
        let prev_validator_exchange_rate_bps_sq = U128x128::from(validator_exchange_rate);
        let prev_base_reward_rate_bps_sq = U128x128::from(base_reward_rate);
        let commission_rate_bps = U128x128::from(self.rate_bps());
        let max_bps = U128x128::from(10_000u128);

        // First, we remove the scaling factors:
        let commission_rate = (commission_rate_bps / max_bps).expect("nonzero divisor");
        let prev_validator_exchange_rate = (prev_validator_exchange_rate_bps_sq
            / *BPS_SQUARED_SCALING_FACTOR)
            .expect("nonzero divisor");
        let prev_base_reward_rate =
            (prev_base_reward_rate_bps_sq / *BPS_SQUARED_SCALING_FACTOR).expect("nonzero divisor");

        // The reward amount at epoch e, for validator v, is R_{v,e}.
        // It is computed as:
        //   R_{v,e} = y_v * c_{v,e} * r_e * psi_v(e)
        //   where:
        //          y_v = total delegation tokens for validator v
        //          c_{v,e} = commission rate for validator v, at epoch e
        //          r_e = base reward rate for epoch e
        //          psi_v(e) = the validator exchange rate for epoch e
        //
        // The commission rate is the sum of all the funding streams rate, and is capped at 100%.
        // In this method, we use a partial commission rate specific to `this` funding stream.

        // Then, we compute the cumulative depreciation for this pool:
        let staking_tokens = (total_delegation_tokens * prev_validator_exchange_rate)
            .expect("exchange rate is close to 1");

        // Now, we can compute the total reward amount for this pool:
        let total_reward_amount =
            (staking_tokens * prev_base_reward_rate).expect("does not overflow");

        /* ********** Compute the reward amount for this funding stream ************* */
        let stream_reward_amount =
            (total_reward_amount * commission_rate).expect("commission rate is between 0 and 1");
        /* ************************************************************************** */

        stream_reward_amount
            .round_down()
            .try_into()
            .expect("does not overflow")
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
///
/// Similarly, it's not possible to build a [`FundingStreams`] with more than 8 funding streams.
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

    pub fn len(&self) -> usize {
        self.funding_streams.len()
    }
}

impl TryFrom<Vec<FundingStream>> for FundingStreams {
    type Error = anyhow::Error;

    fn try_from(funding_streams: Vec<FundingStream>) -> Result<Self, Self::Error> {
        if funding_streams.len() > 8 {
            anyhow::bail!("validators can declare at most 8 funding streams");
        }

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
        (self.funding_streams).iter()
    }
}
