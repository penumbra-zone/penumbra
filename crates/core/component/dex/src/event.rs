use tendermint::abci::{Event, EventAttributeIndexExt};

use crate::{
    lp::{
        action::{PositionClose, PositionOpen, PositionWithdraw},
        position::Position,
    },
    swap::Swap,
    swap_claim::SwapClaim,
};

use penumbra_proto::penumbra::core::component::dex::v1alpha1 as pb;

pub fn swap(swap: &Swap) -> pb::EventSwap {
    pb::EventSwap {
        trading_pair: Some(swap.body.trading_pair.into()),
        delta_1_i: Some(swap.body.delta_1_i.into()),
        delta_2_i: Some(swap.body.delta_2_i.into()),
        commitment: Some(swap.body.fee_commitment.into()),
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

pub fn position_open(position_open: &PositionOpen) -> pb::EventPositionOpen {
    pb::EventPositionOpen {
        position_id: Some(position_open.position.id().into()),
        trading_pair: Some(position_open.position.phi.pair.into()),
        reserves_1: Some(position_open.position.reserves.r1.into()),
        reserves_2: Some(position_open.position.reserves.r2.into()),
        trading_fee: position_open.position.phi.component.fee.into(),
    }
}

pub fn position_close(action: &PositionClose) -> Event {
    // TODO: should we have another event triggered by the position manager for when
    // the position is actually closed?
    Event::new(
        "action_position_close",
        [("position_id", action.position_id.to_string()).index()],
    )
}

pub fn position_withdraw(
    position_withdraw: &PositionWithdraw,
    final_position_state: &Position,
) -> pb::EventPositionWithdraw {
    pb::EventPositionWithdraw {
        position_id: Some(position_withdraw.position_id.into()),
        trading_pair: Some(final_position_state.phi.pair.into()),
        reserves_1: Some(final_position_state.reserves.r1.into()),
        reserves_2: Some(final_position_state.reserves.r2.into()),
    }
}
