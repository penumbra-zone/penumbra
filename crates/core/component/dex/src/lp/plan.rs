use ark_ff::Zero;
use decaf377::Fr;
use penumbra_asset::{balance, Balance, Value};
use penumbra_proto::{penumbra::core::component::dex::v1 as pb, DomainType};
use serde::{Deserialize, Serialize};

use crate::{
    lp::{position, LpNft, Reserves},
    TradingPair,
};

use super::action::PositionWithdraw;

/// A planned [`PositionWithdraw`](PositionWithdraw).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(
    try_from = "pb::PositionWithdrawPlan",
    into = "pb::PositionWithdrawPlan"
)]
pub struct PositionWithdrawPlan {
    pub reserves: Reserves,
    pub position_id: position::Id,
    pub pair: TradingPair,
    pub sequence: u64,
    pub rewards: Vec<Value>,
}

impl PositionWithdrawPlan {
    /// Convenience method to construct the [`PositionWithdraw`] described by this [`PositionWithdrawPlan`].
    pub fn position_withdraw(&self) -> PositionWithdraw {
        PositionWithdraw {
            position_id: self.position_id,
            reserves_commitment: self.reserves_commitment(),
            sequence: self.sequence,
        }
    }

    pub fn reserves_commitment(&self) -> balance::Commitment {
        let mut reserves_balance = self.reserves.balance(&self.pair);
        for reward in &self.rewards {
            reserves_balance += *reward;
        }
        reserves_balance.commit(Fr::zero())
    }

    pub fn balance(&self) -> Balance {
        // PositionWithdraw outputs will correspond to the final reserves
        // and a PositionWithdraw token.
        // Spends will be the PositionClose token.
        let mut balance = self.reserves.balance(&self.pair);

        // We consume a token of self.sequence-1 and produce one of self.sequence.
        // We treat -1 as "closed", the previous state.
        balance -= if self.sequence == 0 {
            Value {
                amount: 1u64.into(),
                asset_id: LpNft::new(self.position_id, position::State::Closed).asset_id(),
            }
        } else {
            Value {
                amount: 1u64.into(),
                asset_id: LpNft::new(
                    self.position_id,
                    position::State::Withdrawn {
                        sequence: self.sequence - 1,
                    },
                )
                .asset_id(),
            }
        };
        balance += Value {
            amount: 1u64.into(),
            asset_id: LpNft::new(
                self.position_id,
                position::State::Withdrawn {
                    sequence: self.sequence,
                },
            )
            .asset_id(),
        };

        balance
    }
}

impl DomainType for PositionWithdrawPlan {
    type Proto = pb::PositionWithdrawPlan;
}

impl From<PositionWithdrawPlan> for pb::PositionWithdrawPlan {
    fn from(msg: PositionWithdrawPlan) -> Self {
        Self {
            reserves: Some(msg.reserves.into()),
            position_id: Some(msg.position_id.into()),
            pair: Some(msg.pair.into()),
            sequence: msg.sequence,
            rewards: msg.rewards.into_iter().map(Into::into).collect(),
        }
    }
}

impl TryFrom<pb::PositionWithdrawPlan> for PositionWithdrawPlan {
    type Error = anyhow::Error;
    fn try_from(msg: pb::PositionWithdrawPlan) -> Result<Self, Self::Error> {
        Ok(Self {
            reserves: msg
                .reserves
                .ok_or_else(|| anyhow::anyhow!("missing reserves"))?
                .try_into()?,
            position_id: msg
                .position_id
                .ok_or_else(|| anyhow::anyhow!("missing position_id"))?
                .try_into()?,
            pair: msg
                .pair
                .ok_or_else(|| anyhow::anyhow!("missing pair"))?
                .try_into()?,
            sequence: msg.sequence,
            rewards: msg
                .rewards
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
        })
    }
}
