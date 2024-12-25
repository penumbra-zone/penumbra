use anyhow::Context;
use penumbra_sdk_proto::{penumbra::core::component::fee::v1 as pb, DomainType};
use std::fmt;
use std::str::FromStr;

use decaf377::Fr;
use penumbra_sdk_asset::{asset, balance, Balance, Value, STAKING_TOKEN_ASSET_ID};
use penumbra_sdk_num::Amount;

// Each fee tier multiplier has an implicit 100 denominator.
pub static FEE_TIER_LOW_MULTIPLIER: u32 = 105;
pub static FEE_TIER_MEDIUM_MULTIPLIER: u32 = 130;
pub static FEE_TIER_HIGH_MULTIPLIER: u32 = 200;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Fee(pub Value);

impl Default for Fee {
    fn default() -> Self {
        Fee::from_staking_token_amount(Amount::zero())
    }
}

impl Fee {
    pub fn from_staking_token_amount(amount: Amount) -> Self {
        Self(Value {
            amount,
            asset_id: *STAKING_TOKEN_ASSET_ID,
        })
    }

    pub fn amount(&self) -> Amount {
        self.0.amount
    }

    pub fn asset_id(&self) -> asset::Id {
        self.0.asset_id
    }

    pub fn asset_matches(&self, other: &Fee) -> bool {
        self.asset_id() == other.asset_id()
    }

    pub fn balance(&self) -> balance::Balance {
        -Balance::from(self.0)
    }

    pub fn commit(&self, blinding: Fr) -> balance::Commitment {
        self.balance().commit(blinding)
    }

    pub fn format(&self, cache: &asset::Cache) -> String {
        self.0.format(cache)
    }

    pub fn apply_tier(self, fee_tier: FeeTier) -> Self {
        // TODO: this could be fingerprinted since fees are public; it would be ideal to apply
        // some sampling distribution, see https://github.com/penumbra-zone/penumbra/issues/3153
        match fee_tier {
            FeeTier::Low => {
                let amount = (self.amount() * FEE_TIER_LOW_MULTIPLIER.into()) / 100u32.into();
                Self(Value {
                    amount,
                    asset_id: self.0.asset_id,
                })
            }
            FeeTier::Medium => {
                let amount = (self.amount() * FEE_TIER_MEDIUM_MULTIPLIER.into()) / 100u32.into();
                Self(Value {
                    amount,
                    asset_id: self.0.asset_id,
                })
            }
            FeeTier::High => {
                let amount = (self.amount() * FEE_TIER_HIGH_MULTIPLIER.into()) / 100u32.into();
                Self(Value {
                    amount,
                    asset_id: self.0.asset_id,
                })
            }
        }
    }
}

impl DomainType for Fee {
    type Proto = pb::Fee;
}

impl From<Fee> for pb::Fee {
    fn from(fee: Fee) -> Self {
        if fee.0.asset_id == *STAKING_TOKEN_ASSET_ID {
            pb::Fee {
                amount: Some(fee.0.amount.into()),
                asset_id: None,
            }
        } else {
            pb::Fee {
                amount: Some(fee.0.amount.into()),
                asset_id: Some(fee.0.asset_id.into()),
            }
        }
    }
}

impl TryFrom<pb::Fee> for Fee {
    type Error = anyhow::Error;

    fn try_from(proto: pb::Fee) -> anyhow::Result<Self> {
        if proto.asset_id.is_some() {
            Ok(Fee(Value {
                amount: proto
                    .amount
                    .context("missing protobuf contents for Fee Amount")?
                    .try_into()?,
                asset_id: proto
                    .asset_id
                    .context("missing protobuf contents for Fee Asset ID")?
                    .try_into()?,
            }))
        } else {
            Ok(Fee(Value {
                amount: proto
                    .amount
                    .context("missing protobuf contents for Fee Amount")?
                    .try_into()?,
                asset_id: *STAKING_TOKEN_ASSET_ID,
            }))
        }
    }
}

impl Fee {
    pub fn value(&self) -> Value {
        self.0
    }
}

#[derive(Copy, Clone, Debug)]
pub enum FeeTier {
    Low,
    Medium,
    High,
}

impl Default for FeeTier {
    fn default() -> Self {
        Self::Low
    }
}

impl fmt::Display for FeeTier {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            FeeTier::Low => "low".to_owned(),
            FeeTier::Medium => "medium".to_owned(),
            FeeTier::High => "high".to_owned(),
        };
        write!(f, "{}", s)
    }
}

impl FromStr for FeeTier {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "low" => Ok(FeeTier::Low),
            "medium" => Ok(FeeTier::Medium),
            "high" => Ok(FeeTier::High),
            _ => anyhow::bail!(format!("cannot parse '{}' as FeeTier", s)),
        }
    }
}

impl DomainType for FeeTier {
    type Proto = pb::FeeTier;
}

impl From<FeeTier> for pb::FeeTier {
    fn from(prices: FeeTier) -> Self {
        match prices {
            FeeTier::Low => pb::FeeTier {
                fee_tier: pb::fee_tier::Tier::Low.into(),
            },
            FeeTier::Medium => pb::FeeTier {
                fee_tier: pb::fee_tier::Tier::Medium.into(),
            },
            FeeTier::High => pb::FeeTier {
                fee_tier: pb::fee_tier::Tier::High.into(),
            },
        }
    }
}

impl TryFrom<pb::FeeTier> for FeeTier {
    type Error = anyhow::Error;

    fn try_from(proto: pb::FeeTier) -> Result<Self, Self::Error> {
        match pb::fee_tier::Tier::try_from(proto.fee_tier)? {
            pb::fee_tier::Tier::Low => Ok(FeeTier::Low),
            pb::fee_tier::Tier::Medium => Ok(FeeTier::Medium),
            pb::fee_tier::Tier::High => Ok(FeeTier::High),
            _ => Err(anyhow::anyhow!("invalid fee tier")),
        }
    }
}
