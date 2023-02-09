use serde::{Deserialize, Serialize};

use penumbra_crypto::{
    balance,
    dex::lp::{
        position::{self, Position, MAX_RESERVE_AMOUNT},
        LpNft, Reserves,
    },
    Balance, Fr, Value, Zero,
};
use penumbra_proto::{core::dex::v1alpha1 as pb, DomainType};

use crate::{ActionView, TransactionPerspective};

use super::IsAction;

/// A transaction action that opens a new position.
///
/// This action's contribution to the transaction's value balance is to consume
/// the initial reserves and contribute an opened position NFT.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::PositionOpen", into = "pb::PositionOpen")]
pub struct PositionOpen {
    /// Contains the data defining the position, sufficient to compute its `PositionId`.
    ///
    /// Positions are immutable, so the `PositionData` (and hence the `PositionId`)
    /// are unchanged over the entire lifetime of the position.
    pub position: Position,
    /// The initial reserves of the position.  Unlike the `PositionData`, the
    /// reserves evolve over time as trades are executed against the position.
    pub initial_reserves: Reserves,
}

impl IsAction for PositionOpen {
    fn balance_commitment(&self) -> balance::Commitment {
        self.balance().commit(Fr::zero())
    }

    fn view_from_perspective(&self, _txp: &TransactionPerspective) -> ActionView {
        ActionView::PositionOpen(self.to_owned())
    }
}

impl PositionOpen {
    /// Create a new valid `PositionOpen`.
    pub fn new(position: Position, initial_reserves: Reserves) -> anyhow::Result<Self> {
        if initial_reserves.r1.value() as u128 > MAX_RESERVE_AMOUNT
            || initial_reserves.r2.value() as u128 > MAX_RESERVE_AMOUNT
        {
            Err(anyhow::anyhow!(format!(
                "position reserves too big (limit: {MAX_RESERVE_AMOUNT})"
            )))
        } else {
            Ok(Self {
                position,
                initial_reserves,
            })
        }
    }

    /// Compute a commitment to the value this action contributes to its transaction.
    pub fn balance(&self) -> Balance {
        let opened_position_nft = Value {
            amount: 1u64.into(),
            asset_id: LpNft::new(self.position.id(), position::State::Opened).asset_id(),
        };

        let r1 = Value {
            amount: self.initial_reserves.r1,
            asset_id: self.position.phi.pair.asset_1(),
        };

        let r2 = Value {
            amount: self.initial_reserves.r2,
            asset_id: self.position.phi.pair.asset_2(),
        };

        let reserves = Balance::from(r1) + r2;

        // The action consumes the reserves and produces an LP NFT
        Balance::from(opened_position_nft) - reserves
    }
}

/// A transaction action that closes a position.
///
/// This action's contribution to the transaction's value balance is to consume
/// an opened position NFT and contribute a closed position NFT.
///
/// Closing a position does not immediately withdraw funds, because Penumbra
/// transactions (like any ZK transaction model) are early-binding: the prover
/// must know the state transition they prove knowledge of, and they cannot know
/// the final reserves with certainty until after the position has been deactivated.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::PositionClose", into = "pb::PositionClose")]
pub struct PositionClose {
    pub position_id: position::Id,
}

impl IsAction for PositionClose {
    fn balance_commitment(&self) -> balance::Commitment {
        self.balance().commit(Fr::zero())
    }

    fn view_from_perspective(&self, _txp: &TransactionPerspective) -> ActionView {
        ActionView::PositionClose(self.to_owned())
    }
}

impl PositionClose {
    /// Compute a commitment to the value this action contributes to its transaction.
    pub fn balance(&self) -> Balance {
        let opened_position_nft = Value {
            amount: 1u64.into(),
            asset_id: LpNft::new(self.position_id, position::State::Opened).asset_id(),
        };

        let closed_position_nft = Value {
            amount: 1u64.into(),
            asset_id: LpNft::new(self.position_id, position::State::Closed).asset_id(),
        };

        // The action consumes an opened position and produces a closed position.
        Balance::from(closed_position_nft) - opened_position_nft
    }
}

/// A transaction action that withdraws funds from a closed position.
///
/// This action's contribution to the transaction's value balance is to consume a
/// closed position NFT and contribute a withdrawn position NFT, as well as all
/// of the funds that were in the position at the time of closing.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::PositionWithdraw", into = "pb::PositionWithdraw")]
pub struct PositionWithdraw {
    pub position_id: position::Id,
    /// A transparent (zero blinding factor) commitment to the position's final reserves and fees.
    ///
    /// The chain will check this commitment by recomputing it with the on-chain state.
    pub reserves_commitment: balance::Commitment,
}

impl IsAction for PositionWithdraw {
    fn balance_commitment(&self) -> balance::Commitment {
        let closed_position_nft = Value {
            amount: 1u64.into(),
            asset_id: LpNft::new(self.position_id, position::State::Closed).asset_id(),
        }
        .commit(Fr::zero());

        // The action consumes a closed position and produces the position's reserves.
        self.reserves_commitment - closed_position_nft
    }

    fn view_from_perspective(&self, _txp: &TransactionPerspective) -> ActionView {
        ActionView::PositionWithdraw(self.to_owned())
    }
}

/// A transaction action that claims retroactive rewards for a historical
/// position.
///
/// This action's contribution to the transaction's value balance is to consume a
/// withdrawn position NFT and contribute its reward balance.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::PositionRewardClaim", into = "pb::PositionRewardClaim")]
pub struct PositionRewardClaim {
    pub position_id: position::Id,
    /// A transparent (zero blinding factor) commitment to the position's accumulated rewards.
    ///
    /// The chain will check this commitment by recomputing it with the on-chain state.
    pub rewards_commitment: balance::Commitment,
}

impl IsAction for PositionRewardClaim {
    fn balance_commitment(&self) -> balance::Commitment {
        let withdrawn_position_nft = Value {
            amount: 1u64.into(),
            asset_id: LpNft::new(self.position_id, position::State::Withdrawn).asset_id(),
        }
        .commit(Fr::zero());

        // The action consumes a closed position and produces the position's reserves.
        self.rewards_commitment - withdrawn_position_nft
    }

    fn view_from_perspective(&self, _txp: &TransactionPerspective) -> ActionView {
        ActionView::PositionRewardClaim(self.to_owned())
    }
}

impl DomainType for PositionOpen {
    type Proto = pb::PositionOpen;
}

impl From<PositionOpen> for pb::PositionOpen {
    fn from(value: PositionOpen) -> Self {
        Self {
            position: Some(value.position.into()),
            initial_reserves: Some(value.initial_reserves.into()),
        }
    }
}

impl TryFrom<pb::PositionOpen> for PositionOpen {
    type Error = anyhow::Error;

    fn try_from(value: pb::PositionOpen) -> Result<Self, Self::Error> {
        Ok(Self {
            position: value
                .position
                .ok_or_else(|| anyhow::anyhow!("missing position"))?
                .try_into()?,
            initial_reserves: value
                .initial_reserves
                .ok_or_else(|| anyhow::anyhow!("missing initial reserves"))?
                .try_into()?,
        })
    }
}

impl DomainType for PositionClose {
    type Proto = pb::PositionClose;
}

impl From<PositionClose> for pb::PositionClose {
    fn from(value: PositionClose) -> Self {
        Self {
            position_id: Some(value.position_id.into()),
        }
    }
}

impl TryFrom<pb::PositionClose> for PositionClose {
    type Error = anyhow::Error;

    fn try_from(value: pb::PositionClose) -> Result<Self, Self::Error> {
        Ok(Self {
            position_id: value
                .position_id
                .ok_or_else(|| anyhow::anyhow!("missing position_id"))?
                .try_into()?,
        })
    }
}

impl DomainType for PositionWithdraw {
    type Proto = pb::PositionWithdraw;
}

impl From<PositionWithdraw> for pb::PositionWithdraw {
    fn from(value: PositionWithdraw) -> Self {
        Self {
            position_id: Some(value.position_id.into()),
            reserves_commitment: Some(value.reserves_commitment.into()),
        }
    }
}

impl TryFrom<pb::PositionWithdraw> for PositionWithdraw {
    type Error = anyhow::Error;

    fn try_from(value: pb::PositionWithdraw) -> Result<Self, Self::Error> {
        Ok(Self {
            position_id: value
                .position_id
                .ok_or_else(|| anyhow::anyhow!("missing position_id"))?
                .try_into()?,
            reserves_commitment: value
                .reserves_commitment
                .ok_or_else(|| anyhow::anyhow!("missing balance_commitment"))?
                .try_into()?,
        })
    }
}

impl DomainType for PositionRewardClaim {
    type Proto = pb::PositionRewardClaim;
}

impl From<PositionRewardClaim> for pb::PositionRewardClaim {
    fn from(value: PositionRewardClaim) -> Self {
        Self {
            position_id: Some(value.position_id.into()),
            rewards_commitment: Some(value.rewards_commitment.into()),
        }
    }
}

impl TryFrom<pb::PositionRewardClaim> for PositionRewardClaim {
    type Error = anyhow::Error;

    fn try_from(value: pb::PositionRewardClaim) -> Result<Self, Self::Error> {
        Ok(Self {
            position_id: value
                .position_id
                .ok_or_else(|| anyhow::anyhow!("missing position_id"))?
                .try_into()?,
            rewards_commitment: value
                .rewards_commitment
                .ok_or_else(|| anyhow::anyhow!("missing balance_commitment"))?
                .try_into()?,
        })
    }
}
