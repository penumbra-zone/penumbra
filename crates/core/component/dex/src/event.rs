use tendermint::abci::{Event, EventAttributeIndexExt};

use crate::{
    lp::{
        action::{PositionClose, PositionOpen, PositionWithdraw},
        position::Position,
    },
    swap::Swap,
    swap_claim::SwapClaim,
};

pub fn swap(swap: &Swap) -> Event {
    Event::new(
        "action_swap",
        [
            ("trading_pair", swap.body.trading_pair.to_string()).index(),
            ("delta_1_i", swap.body.delta_1_i.to_string()).index(),
            ("delta_2_i", swap.body.delta_2_i.to_string()).index(),
            ("swap_commitment", swap.body.payload.commitment.to_string()).index(),
        ],
    )
}

pub fn swap_claim(swap_claim: &SwapClaim) -> Event {
    Event::new(
        "action_swap_claim",
        [
            (
                "trading_pair",
                swap_claim.body.output_data.trading_pair.to_string(),
            )
                .index(),
            (
                "output_1_commitment",
                swap_claim.body.output_1_commitment.to_string(),
            )
                .index(),
            (
                "output_2_commitment",
                swap_claim.body.output_2_commitment.to_string(),
            )
                .index(),
            ("nullifier", swap_claim.body.nullifier.to_string()).index(),
        ],
    )
}

pub fn position_open(action: &PositionOpen) -> Event {
    Event::new(
        "action_position_open",
        [
            ("position_id", action.position.id().to_string()).index(),
            ("trading_pair", action.position.phi.pair.to_string()).index(),
            // TODO: move into position manager and include in a "position updated" event?
            ("reserves_1", action.position.reserves.r1.to_string()).index(),
            ("reserves_2", action.position.reserves.r2.to_string()).index(),
            ("trading_fee", action.position.phi.component.fee.to_string()).index(),
            ("trading_p1", action.position.phi.component.p.to_string()).index(),
            ("trading_p2", action.position.phi.component.q.to_string()).index(),
        ],
    )
}

pub fn position_close(action: &PositionClose) -> Event {
    // TODO: should we have another event triggered by the position manager for when
    // the position is actually closed?
    Event::new(
        "action_position_close",
        [("position_id", action.position_id.to_string()).index()],
    )
}

pub fn position_withdraw(action: &PositionWithdraw, final_position_state: &Position) -> Event {
    Event::new(
        "action_position_withdraw",
        [
            ("position_id", action.position_id.to_string()).index(),
            // reserves not included in action so need to be passed in separately
            ("trading_pair", final_position_state.phi.pair.to_string()).index(),
            ("reserves_1", final_position_state.reserves.r1.to_string()).index(),
            ("reserves_2", final_position_state.reserves.r2.to_string()).index(),
        ],
    )
}
