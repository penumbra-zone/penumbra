use anyhow::{anyhow, Context};
use penumbra_sdk_asset::{
    asset::{self, Denom},
    Value,
};
use penumbra_sdk_dex::lp::position;
use penumbra_sdk_keys::Address;
use penumbra_sdk_num::Amount;
use penumbra_sdk_proto::{penumbra::core::component::funding::v1 as pb, DomainType, Name as _};
use penumbra_sdk_txhash::TransactionId;

#[derive(Clone, Debug)]
pub struct EventFundingStreamReward {
    pub recipient: String,
    pub epoch_index: u64,
    pub reward_amount: Amount,
}

impl TryFrom<pb::EventFundingStreamReward> for EventFundingStreamReward {
    type Error = anyhow::Error;

    fn try_from(value: pb::EventFundingStreamReward) -> Result<Self, Self::Error> {
        fn inner(value: pb::EventFundingStreamReward) -> anyhow::Result<EventFundingStreamReward> {
            Ok(EventFundingStreamReward {
                recipient: value.recipient,
                epoch_index: value.epoch_index,
                reward_amount: value
                    .reward_amount
                    .ok_or(anyhow!("missing `reward_amount`"))?
                    .try_into()?,
            })
        }
        inner(value).context(format!("parsing {}", pb::EventFundingStreamReward::NAME))
    }
}

impl From<EventFundingStreamReward> for pb::EventFundingStreamReward {
    fn from(value: EventFundingStreamReward) -> Self {
        Self {
            recipient: value.recipient,
            epoch_index: value.epoch_index,
            reward_amount: Some(value.reward_amount.into()),
        }
    }
}

impl DomainType for EventFundingStreamReward {
    type Proto = pb::EventFundingStreamReward;
}

/// Event emitted when a delegator receives a reward in the liquidity tournament.
#[derive(Clone, Debug)]
pub struct EventLqtDelegatorReward {
    /// The epoch for which the reward was paid.
    pub epoch_index: u64,
    /// The reward amount in staking tokens.
    pub reward_amount: Amount,
    /// The reward amount in delegation tokens.
    pub delegation_tokens: Value,
    /// The recipient address.
    pub address: Address,
    /// The incentivized asset.
    pub incentivized_asset_id: asset::Id,
}

impl TryFrom<pb::EventLqtDelegatorReward> for EventLqtDelegatorReward {
    type Error = anyhow::Error;
    fn try_from(value: pb::EventLqtDelegatorReward) -> Result<Self, Self::Error> {
        fn inner(value: pb::EventLqtDelegatorReward) -> anyhow::Result<EventLqtDelegatorReward> {
            Ok(EventLqtDelegatorReward {
                epoch_index: value.epoch_index,
                reward_amount: value
                    .reward_amount
                    .ok_or_else(|| anyhow!("missing `reward_amount`"))?
                    .try_into()?,
                delegation_tokens: value
                    .delegation_tokens
                    .ok_or_else(|| anyhow!("missing `delegation_tokens`"))?
                    .try_into()?,
                address: value
                    .address
                    .ok_or_else(|| anyhow!("missing `address`"))?
                    .try_into()?,
                incentivized_asset_id: value
                    .incentivized_asset_id
                    .ok_or_else(|| anyhow!("missing `asset_id`"))?
                    .try_into()?,
            })
        }
        inner(value).context(format!("parsing {}", pb::EventLqtDelegatorReward::NAME))
    }
}

impl From<EventLqtDelegatorReward> for pb::EventLqtDelegatorReward {
    fn from(value: EventLqtDelegatorReward) -> Self {
        Self {
            epoch_index: value.epoch_index,
            reward_amount: Some(value.reward_amount.into()),
            delegation_tokens: Some(value.delegation_tokens.into()),
            address: Some(value.address.into()),
            incentivized_asset_id: Some(value.incentivized_asset_id.into()),
        }
    }
}

impl DomainType for EventLqtDelegatorReward {
    type Proto = pb::EventLqtDelegatorReward;
}

/// Event emitted when a liquidity position receives a reward.
#[derive(Clone, Debug)]
pub struct EventLqtPositionReward {
    /// The epoch for which the reward was paid.
    pub epoch_index: u64,
    /// The reward amount in staking tokens.
    pub reward_amount: Amount,
    /// The liquidity position that receives the reward.
    pub position_id: position::Id,
    /// The incentivized asset.
    pub incentivized_asset_id: asset::Id,
    /// The total volume for the pair during the tournament (in staking tokens).
    pub tournament_volume: Amount,
    /// The cumulative volume for the LP (in staking tokens).
    pub position_volume: Amount,
}

impl TryFrom<pb::EventLqtPositionReward> for EventLqtPositionReward {
    type Error = anyhow::Error;
    fn try_from(value: pb::EventLqtPositionReward) -> Result<Self, Self::Error> {
        fn inner(value: pb::EventLqtPositionReward) -> anyhow::Result<EventLqtPositionReward> {
            Ok(EventLqtPositionReward {
                epoch_index: value.epoch_index,
                reward_amount: value
                    .reward_amount
                    .ok_or_else(|| anyhow!("missing `reward_amount`"))?
                    .try_into()?,
                position_id: value
                    .position_id
                    .ok_or_else(|| anyhow!("missing `position_id`"))?
                    .try_into()?,
                incentivized_asset_id: value
                    .incentivized_asset_id
                    .ok_or_else(|| anyhow!("missing `asset_id`"))?
                    .try_into()?,
                tournament_volume: value
                    .tournament_volume
                    .ok_or_else(|| anyhow!("missing `tournament_volume`"))?
                    .try_into()?,
                position_volume: value
                    .position_volume
                    .ok_or_else(|| anyhow!("missing `position_volume`"))?
                    .try_into()?,
            })
        }
        inner(value).context(format!("parsing {}", pb::EventLqtPositionReward::NAME))
    }
}

impl From<EventLqtPositionReward> for pb::EventLqtPositionReward {
    fn from(value: EventLqtPositionReward) -> Self {
        Self {
            epoch_index: value.epoch_index,
            reward_amount: Some(value.reward_amount.into()),
            position_id: Some(value.position_id.into()),
            incentivized_asset_id: Some(value.incentivized_asset_id.into()),
            tournament_volume: Some(value.tournament_volume.into()),
            position_volume: Some(value.position_volume.into()),
        }
    }
}

impl DomainType for EventLqtPositionReward {
    type Proto = pb::EventLqtPositionReward;
}

/// Event emitted when a vote is cast for a liquidity tournament.
#[derive(Clone, Debug)]
pub struct EventLqtVote {
    /// The tournament epoch for which the vote was cast.
    pub epoch_index: u64,
    /// The voting power associated with the vote.
    pub voting_power: Amount,
    /// The asset being voted on.
    pub incentivized_asset_id: asset::Id,
    /// The denom string for the incentivized asset.
    pub incentivized: Denom,
    /// The address of the beneficiary.
    pub rewards_recipient: Address,
    /// The transaction id that the vote is associated with.
    pub tx_id: TransactionId,
}

impl TryFrom<pb::EventLqtVote> for EventLqtVote {
    type Error = anyhow::Error;
    fn try_from(value: pb::EventLqtVote) -> Result<Self, Self::Error> {
        fn inner(value: pb::EventLqtVote) -> anyhow::Result<EventLqtVote> {
            Ok(EventLqtVote {
                epoch_index: value.epoch_index,
                voting_power: value
                    .voting_power
                    .ok_or_else(|| anyhow!("missing `voting_power`"))?
                    .try_into()?,
                incentivized_asset_id: value
                    .incentivized_asset_id
                    .ok_or_else(|| anyhow!("missing `asset_id`"))?
                    .try_into()?,
                incentivized: value
                    .incentivized
                    .ok_or_else(|| anyhow!("missing `incentivized`"))?
                    .try_into()?,
                rewards_recipient: value
                    .rewards_recipient
                    .ok_or_else(|| anyhow!("missing `voter_address`"))?
                    .try_into()?,
                tx_id: value
                    .tx_id
                    .ok_or_else(|| anyhow!("missing `tx`"))?
                    .try_into()?,
            })
        }
        inner(value).context(format!("parsing {}", pb::EventLqtVote::NAME))
    }
}

impl From<EventLqtVote> for pb::EventLqtVote {
    fn from(value: EventLqtVote) -> Self {
        Self {
            epoch_index: value.epoch_index,
            voting_power: Some(value.voting_power.into()),
            incentivized_asset_id: Some(value.incentivized_asset_id.into()),
            incentivized: Some(value.incentivized.into()),
            rewards_recipient: Some(value.rewards_recipient.into()),
            tx_id: Some(value.tx_id.into()),
        }
    }
}

impl DomainType for EventLqtVote {
    type Proto = pb::EventLqtVote;
}
