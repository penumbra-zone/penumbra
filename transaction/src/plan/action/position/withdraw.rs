use ark_ff::Zero;
use decaf377::Fr;
use penumbra_crypto::{
    dex::{
        lp::{position, LpNft, Reserves},
        TradingPair,
    },
    Value,
};
use penumbra_proto::{core::dex::v1alpha1 as pb, DomainType, TypeUrl};
use serde::{Deserialize, Serialize};

use crate::action::PositionWithdraw;

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
}

impl PositionWithdrawPlan {
    /// Create a new [`PositionWithdrawPlan`]
    pub fn new(
        reserves: Reserves,
        position_id: position::Id,
        pair: TradingPair,
    ) -> PositionWithdrawPlan {
        PositionWithdrawPlan {
            reserves,
            position_id,
            pair,
        }
    }

    /// Convenience method to construct the [`PositionWithdraw`] described by this [`PositionWithdrawPlan`].
    pub fn position_withdraw(&self) -> PositionWithdraw {
        PositionWithdraw {
            position_id: self.position_id,
            reserves_commitment: self.reserves_commitment(),
        }
    }

    pub fn reserves_commitment(&self) -> penumbra_crypto::balance::Commitment {
        self.reserves.balance(&self.pair).commit(Fr::zero())
    }

    pub fn balance(&self) -> penumbra_crypto::Balance {
        // PositionWithdraw outputs will correspond to the final reserves
        // and a PositionWithdraw token.
        // Spends will be the PositionClose token.
        let mut balance = self.reserves.balance(&self.pair);
        balance -= Value {
            amount: 1u64.into(),
            asset_id: LpNft::new(self.position_id, position::State::Closed).asset_id(),
        };
        balance += Value {
            amount: 1u64.into(),
            asset_id: LpNft::new(self.position_id, position::State::Withdrawn).asset_id(),
        };

        balance
    }
}

impl TypeUrl for PositionWithdrawPlan {
    const TYPE_URL: &'static str = "/penumbra.core.dex.v1alpha1.PositionWithdrawPlan";
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
        })
    }
}
