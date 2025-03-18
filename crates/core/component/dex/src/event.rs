use crate::{
    lp::position::{self, Position},
    swap::Swap,
    swap_claim::SwapClaim,
    BatchSwapOutputData, CandlestickData, DirectedTradingPair, SwapExecution, TradingPair,
};
use anyhow::{anyhow, Context};
use penumbra_sdk_asset::asset;
use penumbra_sdk_num::Amount;
use penumbra_sdk_proto::{penumbra::core::component::dex::v1 as pb, DomainType};
use penumbra_sdk_sct::Nullifier;
use penumbra_sdk_tct::StateCommitment;
use prost::Name as _;

#[derive(Clone, Debug)]
pub struct EventSwap {
    pub trading_pair: TradingPair,
    pub delta_1_i: Amount,
    pub delta_2_i: Amount,
    pub swap_commitment: StateCommitment,
}

impl From<Swap> for EventSwap {
    fn from(value: Swap) -> Self {
        Self::from(&value)
    }
}

impl From<&Swap> for EventSwap {
    fn from(value: &Swap) -> Self {
        Self {
            trading_pair: value.body.trading_pair,
            delta_1_i: value.body.delta_1_i,
            delta_2_i: value.body.delta_2_i,
            swap_commitment: value.body.payload.commitment,
        }
    }
}

impl TryFrom<pb::EventSwap> for EventSwap {
    type Error = anyhow::Error;

    fn try_from(value: pb::EventSwap) -> Result<Self, Self::Error> {
        fn inner(value: pb::EventSwap) -> anyhow::Result<EventSwap> {
            Ok(EventSwap {
                trading_pair: value
                    .trading_pair
                    .ok_or(anyhow!("missing `trading_pair`"))?
                    .try_into()?,
                delta_1_i: value
                    .delta_1_i
                    .ok_or(anyhow!("missing `delta_1_i`"))?
                    .try_into()?,
                delta_2_i: value
                    .delta_2_i
                    .ok_or(anyhow!("missing `delta_2_i`"))?
                    .try_into()?,
                swap_commitment: value
                    .swap_commitment
                    .ok_or(anyhow!("missing `swap_commitment`"))?
                    .try_into()?,
            })
        }
        inner(value).context(format!("parsing {}", pb::EventSwap::NAME))
    }
}

impl From<EventSwap> for pb::EventSwap {
    fn from(value: EventSwap) -> Self {
        Self {
            trading_pair: Some(value.trading_pair.into()),
            delta_1_i: Some(value.delta_1_i.into()),
            delta_2_i: Some(value.delta_2_i.into()),
            swap_commitment: Some(value.swap_commitment.into()),
        }
    }
}

impl DomainType for EventSwap {
    type Proto = pb::EventSwap;
}

#[derive(Clone, Debug)]
pub struct EventSwapClaim {
    pub trading_pair: TradingPair,
    pub output_1_commitment: StateCommitment,
    pub output_2_commitment: StateCommitment,
    pub nullifier: Nullifier,
}

impl From<SwapClaim> for EventSwapClaim {
    fn from(value: SwapClaim) -> Self {
        Self::from(&value)
    }
}

impl From<&SwapClaim> for EventSwapClaim {
    fn from(value: &SwapClaim) -> Self {
        Self {
            trading_pair: value.body.output_data.trading_pair,
            output_1_commitment: value.body.output_1_commitment,
            output_2_commitment: value.body.output_2_commitment,
            nullifier: value.body.nullifier,
        }
    }
}

impl TryFrom<pb::EventSwapClaim> for EventSwapClaim {
    type Error = anyhow::Error;

    fn try_from(value: pb::EventSwapClaim) -> Result<Self, Self::Error> {
        fn inner(value: pb::EventSwapClaim) -> anyhow::Result<EventSwapClaim> {
            Ok(EventSwapClaim {
                trading_pair: value
                    .trading_pair
                    .ok_or(anyhow!("missing `trading_pair`"))?
                    .try_into()?,
                output_1_commitment: value
                    .output_1_commitment
                    .ok_or(anyhow!("missing `output_1_commitment`"))?
                    .try_into()?,
                output_2_commitment: value
                    .output_2_commitment
                    .ok_or(anyhow!("missing `output_2_commitment`"))?
                    .try_into()?,
                nullifier: value
                    .nullifier
                    .ok_or(anyhow!("missing `nullifier`"))?
                    .try_into()?,
            })
        }
        inner(value).context(format!("parsing {}", pb::EventSwapClaim::NAME))
    }
}

impl From<EventSwapClaim> for pb::EventSwapClaim {
    fn from(value: EventSwapClaim) -> Self {
        Self {
            trading_pair: Some(value.trading_pair.into()),
            output_1_commitment: Some(value.output_1_commitment.into()),
            output_2_commitment: Some(value.output_2_commitment.into()),
            nullifier: Some(value.nullifier.into()),
        }
    }
}

impl DomainType for EventSwapClaim {
    type Proto = pb::EventSwapClaim;
}

#[derive(Clone, Debug)]
pub struct EventPositionOpen {
    pub position_id: position::Id,
    pub trading_pair: TradingPair,
    pub reserves_1: Amount,
    pub reserves_2: Amount,
    pub trading_fee: u32,
    pub position: Position,
}

impl From<Position> for EventPositionOpen {
    fn from(value: Position) -> Self {
        Self {
            position_id: value.id(),
            trading_pair: value.phi.pair,
            reserves_1: value.reserves_1().amount,
            reserves_2: value.reserves_2().amount,
            trading_fee: value.phi.component.fee,
            position: value,
        }
    }
}

impl TryFrom<pb::EventPositionOpen> for EventPositionOpen {
    type Error = anyhow::Error;

    fn try_from(value: pb::EventPositionOpen) -> Result<Self, Self::Error> {
        fn inner(value: pb::EventPositionOpen) -> anyhow::Result<EventPositionOpen> {
            Ok(EventPositionOpen {
                position_id: value
                    .position_id
                    .ok_or(anyhow!("missing `position_id`"))?
                    .try_into()?,
                trading_pair: value
                    .trading_pair
                    .ok_or(anyhow!("missing `trading_pair`"))?
                    .try_into()?,
                reserves_1: value
                    .reserves_1
                    .ok_or(anyhow!("missing `reserves_1`"))?
                    .try_into()?,
                reserves_2: value
                    .reserves_2
                    .ok_or(anyhow!("missing `reserves_2`"))?
                    .try_into()?,
                trading_fee: value.trading_fee,
                position: value
                    .position
                    .ok_or(anyhow!("missing `position`"))?
                    .try_into()?,
            })
        }
        inner(value).context(format!("parsing {}", pb::EventPositionOpen::NAME))
    }
}

impl From<EventPositionOpen> for pb::EventPositionOpen {
    fn from(value: EventPositionOpen) -> Self {
        Self {
            position_id: Some(value.position_id.into()),
            trading_pair: Some(value.trading_pair.into()),
            reserves_1: Some(value.reserves_1.into()),
            reserves_2: Some(value.reserves_2.into()),
            trading_fee: value.trading_fee,
            position: Some(value.position.into()),
        }
    }
}

impl DomainType for EventPositionOpen {
    type Proto = pb::EventPositionOpen;
}

#[derive(Clone, Debug)]
pub struct EventPositionClose {
    pub position_id: position::Id,
}

impl TryFrom<pb::EventPositionClose> for EventPositionClose {
    type Error = anyhow::Error;

    fn try_from(value: pb::EventPositionClose) -> Result<Self, Self::Error> {
        fn inner(value: pb::EventPositionClose) -> anyhow::Result<EventPositionClose> {
            Ok(EventPositionClose {
                position_id: value
                    .position_id
                    .ok_or(anyhow!("missing `position_id`"))?
                    .try_into()?,
            })
        }
        inner(value).context(format!("parsing {}", pb::EventPositionClose::NAME))
    }
}

impl From<EventPositionClose> for pb::EventPositionClose {
    fn from(value: EventPositionClose) -> Self {
        Self {
            position_id: Some(value.position_id.into()),
        }
    }
}

impl DomainType for EventPositionClose {
    type Proto = pb::EventPositionClose;
}

#[derive(Clone, Debug)]
pub struct EventQueuePositionClose {
    pub position_id: position::Id,
}

impl TryFrom<pb::EventQueuePositionClose> for EventQueuePositionClose {
    type Error = anyhow::Error;

    fn try_from(value: pb::EventQueuePositionClose) -> Result<Self, Self::Error> {
        fn inner(value: pb::EventQueuePositionClose) -> anyhow::Result<EventQueuePositionClose> {
            Ok(EventQueuePositionClose {
                position_id: value
                    .position_id
                    .ok_or(anyhow!("missing `position_id`"))?
                    .try_into()?,
            })
        }
        inner(value).context(format!("parsing {}", pb::EventQueuePositionClose::NAME))
    }
}

impl From<EventQueuePositionClose> for pb::EventQueuePositionClose {
    fn from(value: EventQueuePositionClose) -> Self {
        Self {
            position_id: Some(value.position_id.into()),
        }
    }
}

impl DomainType for EventQueuePositionClose {
    type Proto = pb::EventQueuePositionClose;
}

#[derive(Clone, Debug)]
pub struct EventPositionWithdraw {
    pub position_id: position::Id,
    pub trading_pair: TradingPair,
    pub reserves_1: Amount,
    pub reserves_2: Amount,
    pub sequence: u64,
}

impl EventPositionWithdraw {
    /// Create this event using the usual context available to us.
    pub fn in_context(position_id: position::Id, final_position_state: &Position) -> Self {
        let sequence =
            if let position::State::Withdrawn { sequence, .. } = final_position_state.state {
                sequence + 1
            } else {
                0
            };
        Self {
            position_id,
            trading_pair: final_position_state.phi.pair,
            reserves_1: final_position_state.reserves.r1,
            reserves_2: final_position_state.reserves.r2,
            sequence,
        }
    }
}

impl TryFrom<pb::EventPositionWithdraw> for EventPositionWithdraw {
    type Error = anyhow::Error;

    fn try_from(value: pb::EventPositionWithdraw) -> Result<Self, Self::Error> {
        fn inner(value: pb::EventPositionWithdraw) -> anyhow::Result<EventPositionWithdraw> {
            Ok(EventPositionWithdraw {
                position_id: value
                    .position_id
                    .ok_or(anyhow!("missing `position_id`"))?
                    .try_into()?,
                trading_pair: value
                    .trading_pair
                    .ok_or(anyhow!("missing `trading_pair`"))?
                    .try_into()?,
                reserves_1: value
                    .reserves_1
                    .ok_or(anyhow!("missing `reserves_1`"))?
                    .try_into()?,
                reserves_2: value
                    .reserves_2
                    .ok_or(anyhow!("missing `reserves_2`"))?
                    .try_into()?,
                sequence: value.sequence,
            })
        }
        inner(value).context(format!("parsing {}", pb::EventPositionWithdraw::NAME))
    }
}

impl From<EventPositionWithdraw> for pb::EventPositionWithdraw {
    fn from(value: EventPositionWithdraw) -> Self {
        Self {
            position_id: Some(value.position_id.into()),
            trading_pair: Some(value.trading_pair.into()),
            reserves_1: Some(value.reserves_1.into()),
            reserves_2: Some(value.reserves_2.into()),
            sequence: value.sequence,
        }
    }
}

impl DomainType for EventPositionWithdraw {
    type Proto = pb::EventPositionWithdraw;
}

#[derive(Clone, Debug)]
pub struct EventPositionExecution {
    pub position_id: position::Id,
    pub trading_pair: TradingPair,
    pub reserves_1: Amount,
    pub reserves_2: Amount,
    pub prev_reserves_1: Amount,
    pub prev_reserves_2: Amount,
    pub context: DirectedTradingPair,
}

impl EventPositionExecution {
    /// Create this event using the usual context available to us.
    pub fn in_context(
        prev_state: &Position,
        new_state: &Position,
        context: DirectedTradingPair,
    ) -> Self {
        Self {
            position_id: new_state.id(),
            trading_pair: new_state.phi.pair,
            reserves_1: new_state.reserves_1().amount,
            reserves_2: new_state.reserves_2().amount,
            prev_reserves_1: prev_state.reserves_1().amount,
            prev_reserves_2: prev_state.reserves_2().amount,
            context,
        }
    }
}

impl TryFrom<pb::EventPositionExecution> for EventPositionExecution {
    type Error = anyhow::Error;

    fn try_from(value: pb::EventPositionExecution) -> Result<Self, Self::Error> {
        fn inner(value: pb::EventPositionExecution) -> anyhow::Result<EventPositionExecution> {
            Ok(EventPositionExecution {
                position_id: value
                    .position_id
                    .ok_or(anyhow!("missing `position_id`"))?
                    .try_into()?,
                trading_pair: value
                    .trading_pair
                    .ok_or(anyhow!("missing `trading_pair`"))?
                    .try_into()?,
                reserves_1: value
                    .reserves_1
                    .ok_or(anyhow!("missing `reserves_1`"))?
                    .try_into()?,
                reserves_2: value
                    .reserves_2
                    .ok_or(anyhow!("missing `reserves_2`"))?
                    .try_into()?,
                prev_reserves_1: value
                    .prev_reserves_1
                    .ok_or(anyhow!("missing `prev_reserves_1`"))?
                    .try_into()?,
                prev_reserves_2: value
                    .prev_reserves_2
                    .ok_or(anyhow!("missing `prev_reserves_2`"))?
                    .try_into()?,
                context: value
                    .context
                    .ok_or(anyhow!("missing `context`"))?
                    .try_into()?,
            })
        }
        inner(value).context(format!("parsing {}", pb::EventPositionExecution::NAME))
    }
}

impl From<EventPositionExecution> for pb::EventPositionExecution {
    fn from(value: EventPositionExecution) -> Self {
        Self {
            position_id: Some(value.position_id.into()),
            trading_pair: Some(value.trading_pair.into()),
            reserves_1: Some(value.reserves_1.into()),
            reserves_2: Some(value.reserves_2.into()),
            prev_reserves_1: Some(value.prev_reserves_1.into()),
            prev_reserves_2: Some(value.prev_reserves_2.into()),
            context: Some(value.context.into()),
        }
    }
}

impl DomainType for EventPositionExecution {
    type Proto = pb::EventPositionExecution;
}

#[derive(Clone, Debug)]
pub struct EventBatchSwap {
    pub batch_swap_output_data: BatchSwapOutputData,
    pub swap_execution_1_for_2: Option<SwapExecution>,
    pub swap_execution_2_for_1: Option<SwapExecution>,
}

impl TryFrom<pb::EventBatchSwap> for EventBatchSwap {
    type Error = anyhow::Error;

    fn try_from(value: pb::EventBatchSwap) -> Result<Self, Self::Error> {
        fn inner(value: pb::EventBatchSwap) -> anyhow::Result<EventBatchSwap> {
            Ok(EventBatchSwap {
                batch_swap_output_data: value
                    .batch_swap_output_data
                    .ok_or(anyhow!("missing `batch_swap_output_data`"))?
                    .try_into()?,
                swap_execution_1_for_2: value
                    .swap_execution_1_for_2
                    .map(|x| x.try_into())
                    .transpose()?,
                swap_execution_2_for_1: value
                    .swap_execution_2_for_1
                    .map(|x| x.try_into())
                    .transpose()?,
            })
        }
        inner(value).context(format!("parsing {}", pb::EventBatchSwap::NAME))
    }
}

impl From<EventBatchSwap> for pb::EventBatchSwap {
    fn from(value: EventBatchSwap) -> Self {
        Self {
            batch_swap_output_data: Some(value.batch_swap_output_data.into()),
            swap_execution_1_for_2: value.swap_execution_1_for_2.map(|x| x.into()),
            swap_execution_2_for_1: value.swap_execution_2_for_1.map(|x| x.into()),
        }
    }
}

impl DomainType for EventBatchSwap {
    type Proto = pb::EventBatchSwap;
}

#[derive(Clone, Debug)]
pub struct EventArbExecution {
    pub height: u64,
    pub swap_execution: SwapExecution,
}

impl TryFrom<pb::EventArbExecution> for EventArbExecution {
    type Error = anyhow::Error;

    fn try_from(value: pb::EventArbExecution) -> Result<Self, Self::Error> {
        fn inner(value: pb::EventArbExecution) -> anyhow::Result<EventArbExecution> {
            Ok(EventArbExecution {
                height: value.height,
                swap_execution: value
                    .swap_execution
                    .ok_or(anyhow!("missing `swap_execution`"))?
                    .try_into()?,
            })
        }
        inner(value).context(format!("parsing {}", pb::EventArbExecution::NAME))
    }
}

impl From<EventArbExecution> for pb::EventArbExecution {
    fn from(value: EventArbExecution) -> Self {
        Self {
            height: value.height,
            swap_execution: Some(value.swap_execution.into()),
        }
    }
}

impl DomainType for EventArbExecution {
    type Proto = pb::EventArbExecution;
}

#[derive(Clone, Debug)]
pub struct EventValueCircuitBreakerCredit {
    pub asset_id: asset::Id,
    pub previous_balance: Amount,
    pub new_balance: Amount,
}

impl TryFrom<pb::EventValueCircuitBreakerCredit> for EventValueCircuitBreakerCredit {
    type Error = anyhow::Error;

    fn try_from(value: pb::EventValueCircuitBreakerCredit) -> Result<Self, Self::Error> {
        fn inner(
            value: pb::EventValueCircuitBreakerCredit,
        ) -> anyhow::Result<EventValueCircuitBreakerCredit> {
            Ok(EventValueCircuitBreakerCredit {
                asset_id: value
                    .asset_id
                    .ok_or(anyhow!("missing `asset_id`"))?
                    .try_into()?,
                previous_balance: value
                    .previous_balance
                    .ok_or(anyhow!("missing `previous_balance`"))?
                    .try_into()?,
                new_balance: value
                    .new_balance
                    .ok_or(anyhow!("missing `new_balance`"))?
                    .try_into()?,
            })
        }
        inner(value).context(format!(
            "parsing {}",
            pb::EventValueCircuitBreakerCredit::NAME
        ))
    }
}

impl From<EventValueCircuitBreakerCredit> for pb::EventValueCircuitBreakerCredit {
    fn from(value: EventValueCircuitBreakerCredit) -> Self {
        Self {
            asset_id: Some(value.asset_id.into()),
            previous_balance: Some(value.previous_balance.into()),
            new_balance: Some(value.new_balance.into()),
        }
    }
}

impl DomainType for EventValueCircuitBreakerCredit {
    type Proto = pb::EventValueCircuitBreakerCredit;
}

#[derive(Clone, Debug)]
pub struct EventValueCircuitBreakerDebit {
    pub asset_id: asset::Id,
    pub previous_balance: Amount,
    pub new_balance: Amount,
}

impl TryFrom<pb::EventValueCircuitBreakerDebit> for EventValueCircuitBreakerDebit {
    type Error = anyhow::Error;

    fn try_from(value: pb::EventValueCircuitBreakerDebit) -> Result<Self, Self::Error> {
        fn inner(
            value: pb::EventValueCircuitBreakerDebit,
        ) -> anyhow::Result<EventValueCircuitBreakerDebit> {
            Ok(EventValueCircuitBreakerDebit {
                asset_id: value
                    .asset_id
                    .ok_or(anyhow!("missing `asset_id`"))?
                    .try_into()?,
                previous_balance: value
                    .previous_balance
                    .ok_or(anyhow!("missing `previous_balance`"))?
                    .try_into()?,
                new_balance: value
                    .new_balance
                    .ok_or(anyhow!("missing `new_balance`"))?
                    .try_into()?,
            })
        }
        inner(value).context(format!(
            "parsing {}",
            pb::EventValueCircuitBreakerDebit::NAME
        ))
    }
}

impl From<EventValueCircuitBreakerDebit> for pb::EventValueCircuitBreakerDebit {
    fn from(value: EventValueCircuitBreakerDebit) -> Self {
        Self {
            asset_id: Some(value.asset_id.into()),
            previous_balance: Some(value.previous_balance.into()),
            new_balance: Some(value.new_balance.into()),
        }
    }
}

impl DomainType for EventValueCircuitBreakerDebit {
    type Proto = pb::EventValueCircuitBreakerDebit;
}

#[derive(Clone, Debug)]
pub struct EventCandlestickData {
    pub pair: DirectedTradingPair,
    pub stick: CandlestickData,
}

impl TryFrom<pb::EventCandlestickData> for EventCandlestickData {
    type Error = anyhow::Error;

    fn try_from(value: pb::EventCandlestickData) -> Result<Self, Self::Error> {
        fn inner(value: pb::EventCandlestickData) -> anyhow::Result<EventCandlestickData> {
            Ok(EventCandlestickData {
                pair: value.pair.ok_or(anyhow!("missing `pair`"))?.try_into()?,
                stick: value.stick.ok_or(anyhow!("missing `stick`"))?.try_into()?,
            })
        }
        inner(value).context(format!("parsing {}", pb::EventCandlestickData::NAME))
    }
}

impl From<EventCandlestickData> for pb::EventCandlestickData {
    fn from(value: EventCandlestickData) -> Self {
        Self {
            pair: Some(value.pair.into()),
            stick: Some(value.stick.into()),
        }
    }
}

impl DomainType for EventCandlestickData {
    type Proto = pb::EventCandlestickData;
}

#[derive(Clone, Debug)]
pub struct EventLqtPositionVolume {
    pub epoch_index: u64,
    pub asset_id: asset::Id,
    pub position_id: position::Id,
    pub volume: Amount,
    pub total_volume: Amount,
    pub staking_token_in: Amount,
    pub asset_in: Amount,
    pub staking_fees: Amount,
    pub asset_fees: Amount,
}

impl From<EventLqtPositionVolume> for pb::EventLqtPositionVolume {
    fn from(value: EventLqtPositionVolume) -> Self {
        Self {
            epoch_index: value.epoch_index,
            asset_id: Some(value.asset_id.into()),
            position_id: Some(value.position_id.into()),
            volume_amount: Some(value.volume.into()),
            total_volume: Some(value.total_volume.into()),
            staking_token_in: Some(value.staking_token_in.into()),
            asset_in: Some(value.asset_in.into()),
            staking_fees: Some(value.staking_fees.into()),
            asset_fees: Some(value.asset_fees.into()),
        }
    }
}

impl TryFrom<pb::EventLqtPositionVolume> for EventLqtPositionVolume {
    type Error = anyhow::Error;

    fn try_from(value: pb::EventLqtPositionVolume) -> Result<Self, Self::Error> {
        Ok(EventLqtPositionVolume {
            epoch_index: value.epoch_index,
            asset_id: value
                .asset_id
                .ok_or(anyhow!("missing `asset_id`"))?
                .try_into()?,
            position_id: value
                .position_id
                .ok_or(anyhow!("missing `position_id`"))?
                .try_into()?,
            volume: value
                .volume_amount
                .ok_or(anyhow!("missing `volume`"))?
                .try_into()?,
            total_volume: value
                .total_volume
                .ok_or(anyhow!("missing `total_volume`"))?
                .try_into()?,
            staking_token_in: value
                .staking_token_in
                .ok_or(anyhow!("mising `staking_token_in`"))?
                .try_into()?,
            asset_in: value
                .asset_in
                .ok_or(anyhow!("mising `asset_in`"))?
                .try_into()?,
            staking_fees: value
                .staking_fees
                .ok_or(anyhow!("mising `staking_fees`"))?
                .try_into()?,
            asset_fees: value
                .asset_fees
                .ok_or(anyhow!("mising `asset_fees`"))?
                .try_into()?,
        })
    }
}

impl DomainType for EventLqtPositionVolume {
    type Proto = pb::EventLqtPositionVolume;
}
