use anyhow::anyhow;
use cometindex::ContextualizedEvent;
use penumbra_auction::auction::{
    dutch::{DutchAuctionDescription, DutchAuctionState},
    AuctionId,
};
use penumbra_proto::{core::component::auction::v1 as pb, event::ProtoEvent};

#[derive(Clone, Copy)]
enum EventKind {
    DutchScheduled,
    DutchUpdated,
    DutchEnded,
    DutchWithdrawn,
}

impl EventKind {
    /// The string tag of this kind of event.
    ///
    /// This is used to match the raw ABCI events with this kind.
    fn tag(&self) -> &'static str {
        const TAGS: [&'static str; 4] = [
            "penumbra.core.component.auction.v1.EventDutchAuctionScheduled",
            "penumbra.core.component.auction.v1.EventDutchAuctionUpdated",
            "penumbra.core.component.auction.v1.EventDutchAuctionEnded",
            "penumbra.core.component.auction.v1.EventDutchAuctionWithdrawn",
        ];
        let ix: usize = match *self {
            EventKind::DutchScheduled => 0,
            EventKind::DutchUpdated => 1,
            EventKind::DutchEnded => 2,
            EventKind::DutchWithdrawn => 3,
        };
        TAGS[ix]
    }
}

impl TryFrom<&str> for EventKind {
    type Error = anyhow::Error;

    fn try_from(tag: &str) -> Result<Self, Self::Error> {
        use EventKind::*;

        for kind in [DutchScheduled, DutchUpdated, DutchEnded, DutchWithdrawn] {
            if tag == kind.tag() {
                return Ok(kind);
            }
        }
        return Err(anyhow!("unexpected event kind: {tag}"));
    }
}

#[derive(Debug, Clone)]
enum Event {
    DutchScheduled {
        id: AuctionId,
        description: DutchAuctionDescription,
    },
    DutchUpdated {
        id: AuctionId,
        state: DutchAuctionState,
    },
    DutchEnded {
        id: AuctionId,
        state: DutchAuctionState,
        // We don't have anything better to represent this with.
        reason: pb::event_dutch_auction_ended::Reason,
    },
    DutchWithdrawn {
        id: AuctionId,
        state: DutchAuctionState,
    },
}

impl TryFrom<&ContextualizedEvent> for Event {
    type Error = anyhow::Error;

    fn try_from(event: &ContextualizedEvent) -> Result<Self, Self::Error> {
        match EventKind::try_from(event.event.kind.as_str())? {
            EventKind::DutchScheduled => {
                let pe = pb::EventDutchAuctionScheduled::from_event(event.as_ref())?;
                let id = pe
                    .auction_id
                    .ok_or(anyhow!("missing auction id"))?
                    .try_into()?;
                let description = pe
                    .description
                    .ok_or(anyhow!("missing description"))?
                    .try_into()?;
                Ok(Event::DutchScheduled { id, description })
            }
            EventKind::DutchUpdated => {
                let pe = pb::EventDutchAuctionUpdated::from_event(event.as_ref())?;
                let id = pe
                    .auction_id
                    .ok_or(anyhow!("missing auction id"))?
                    .try_into()?;
                let state = pe.state.ok_or(anyhow!("missing state"))?.try_into()?;
                Ok(Event::DutchUpdated { id, state })
            }
            EventKind::DutchEnded => {
                let pe = pb::EventDutchAuctionEnded::from_event(event.as_ref())?;
                let id = pe
                    .auction_id
                    .ok_or(anyhow!("missing auction id"))?
                    .try_into()?;
                let state = pe.state.ok_or(anyhow!("missing state"))?.try_into()?;
                let reason = pe.reason.try_into()?;
                Ok(Event::DutchEnded { id, state, reason })
            }
            EventKind::DutchWithdrawn => {
                let pe = pb::EventDutchAuctionWithdrawn::from_event(event.as_ref())?;
                let id = pe
                    .auction_id
                    .ok_or(anyhow!("missing auction id"))?
                    .try_into()?;
                let state = pe.state.ok_or(anyhow!("missing state"))?.try_into()?;
                Ok(Event::DutchWithdrawn { id, state })
            }
        }
    }
}
