use crate::{
    lp::{
        action::PositionClose,
        position::{self, Position},
    },
    swap::Swap,
    swap_claim::SwapClaim,
    BatchSwapOutputData, CandlestickData, DirectedTradingPair, SwapExecution,
};
use anyhow::{anyhow, Context};
use prost::Name;

use penumbra_asset::asset;
use penumbra_num::Amount;
use penumbra_proto::{penumbra::core::component::dex::v1 as pb, DomainType};

pub fn swap(swap: &Swap) -> pb::EventSwap {
    pb::EventSwap {
        trading_pair: Some(swap.body.trading_pair.into()),
        delta_1_i: Some(swap.body.delta_1_i.into()),
        delta_2_i: Some(swap.body.delta_2_i.into()),
        swap_commitment: Some(swap.body.payload.commitment.into()),
    }
}

pub fn swap_claim(swap_claim: &SwapClaim) -> pb::EventSwapClaim {
    pb::EventSwapClaim {
        trading_pair: Some(swap_claim.body.output_data.trading_pair.into()),
        output_1_commitment: Some(swap_claim.body.output_1_commitment.into()),
        output_2_commitment: Some(swap_claim.body.output_2_commitment.into()),
        nullifier: Some(swap_claim.body.nullifier.into()),
    }
}

pub fn position_open(position: &Position) -> pb::EventPositionOpen {
    pb::EventPositionOpen {
        position_id: Some(position.id().into()),
        trading_pair: Some(position.phi.pair.into()),
        reserves_1: Some(position.reserves.r1.into()),
        reserves_2: Some(position.reserves.r2.into()),
        trading_fee: position.phi.component.fee,
        position: Some(position.clone().into()),
    }
}

pub fn position_close_by_id(id: position::Id) -> pb::EventPositionClose {
    pb::EventPositionClose {
        position_id: Some(id.into()),
    }
}

pub fn position_close(action: &PositionClose) -> pb::EventPositionClose {
    pb::EventPositionClose {
        position_id: Some(action.position_id.into()),
    }
}

pub fn queue_position_close(action: &PositionClose) -> pb::EventQueuePositionClose {
    pb::EventQueuePositionClose {
        position_id: Some(action.position_id.into()),
    }
}

pub fn position_withdraw(
    position_id: position::Id,
    final_position_state: &Position,
) -> pb::EventPositionWithdraw {
    let sequence = if let position::State::Withdrawn { sequence, .. } = final_position_state.state {
        sequence + 1
    } else {
        0
    };
    pb::EventPositionWithdraw {
        position_id: Some(position_id.into()),
        trading_pair: Some(final_position_state.phi.pair.into()),
        reserves_1: Some(final_position_state.reserves.r1.into()),
        reserves_2: Some(final_position_state.reserves.r2.into()),
        sequence,
    }
}

pub fn position_execution(
    prev_state: &Position,
    new_state: &Position,
    context: DirectedTradingPair,
) -> pb::EventPositionExecution {
    pb::EventPositionExecution {
        position_id: Some(new_state.id().into()),
        trading_pair: Some(new_state.phi.pair.into()),
        reserves_1: Some(new_state.reserves.r1.into()),
        reserves_2: Some(new_state.reserves.r2.into()),
        prev_reserves_1: Some(prev_state.reserves.r1.into()),
        prev_reserves_2: Some(prev_state.reserves.r2.into()),
        context: Some(context.into()),
    }
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
