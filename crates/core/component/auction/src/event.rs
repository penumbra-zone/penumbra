use crate::auction::dutch::{DutchAuctionDescription, DutchAuctionState};
use crate::auction::AuctionId;
use penumbra_sdk_asset::asset;
use penumbra_sdk_num::Amount;
use penumbra_sdk_proto::penumbra::core::component::auction::v1 as pb;

/// Event for a Dutch auction that has been scheduled.
pub fn dutch_auction_schedule_event(
    id: AuctionId,
    description: DutchAuctionDescription,
) -> pb::EventDutchAuctionScheduled {
    pb::EventDutchAuctionScheduled {
        auction_id: Some(id.into()),
        description: Some(description.into()),
    }
}

/// Event for an execution round of a Dutch auction.
pub fn dutch_auction_updated(
    id: AuctionId,
    state: DutchAuctionState,
) -> pb::EventDutchAuctionUpdated {
    pb::EventDutchAuctionUpdated {
        auction_id: Some(id.into()),
        state: Some(state.into()),
    }
}

/// Event for a Dutch auction that is ending because it has been closed by its owner.
pub fn dutch_auction_closed_by_user(
    id: AuctionId,
    state: DutchAuctionState,
) -> pb::EventDutchAuctionEnded {
    pb::EventDutchAuctionEnded {
        auction_id: Some(id.into()),
        state: Some(state.into()),
        reason: pb::event_dutch_auction_ended::Reason::ClosedByOwner as i32,
    }
}

/// Event for a Dutch auction that is ending because it has expired.
pub fn dutch_auction_expired(
    id: AuctionId,
    state: DutchAuctionState,
) -> pb::EventDutchAuctionEnded {
    pb::EventDutchAuctionEnded {
        auction_id: Some(id.into()),
        state: Some(state.into()),
        reason: pb::event_dutch_auction_ended::Reason::Expired as i32,
    }
}

/// Event for a Dutch auction that is ending because it has been completely filled.
pub fn dutch_auction_exhausted(
    id: AuctionId,
    state: DutchAuctionState,
) -> pb::EventDutchAuctionEnded {
    pb::EventDutchAuctionEnded {
        auction_id: Some(id.into()),
        state: Some(state.into()),
        reason: pb::event_dutch_auction_ended::Reason::Filled as i32,
    }
}

/// Event for a Dutch auction that is withdrawn by a user after ending.
pub fn dutch_auction_withdrawn(
    id: AuctionId,
    state: DutchAuctionState,
) -> pb::EventDutchAuctionWithdrawn {
    pb::EventDutchAuctionWithdrawn {
        auction_id: Some(id.into()),
        state: Some(state.into()),
    }
}

// Event for value flowing *into* the auction component.
pub fn auction_vcb_credit(
    asset_id: asset::Id,
    previous_balance: Amount,
    new_balance: Amount,
) -> pb::EventValueCircuitBreakerCredit {
    pb::EventValueCircuitBreakerCredit {
        asset_id: Some(asset_id.into()),
        previous_balance: Some(previous_balance.into()),
        new_balance: Some(new_balance.into()),
    }
}

// Event for value flowing *out of* the auction component.
pub fn auction_vcb_debit(
    asset_id: asset::Id,
    previous_balance: Amount,
    new_balance: Amount,
) -> pb::EventValueCircuitBreakerDebit {
    pb::EventValueCircuitBreakerDebit {
        asset_id: Some(asset_id.into()),
        previous_balance: Some(previous_balance.into()),
        new_balance: Some(new_balance.into()),
    }
}
