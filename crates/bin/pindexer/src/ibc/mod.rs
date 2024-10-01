use anyhow::anyhow;
use cometindex::ContextualizedEvent;
use penumbra_asset::Value;
use penumbra_keys::Address;
use penumbra_proto::{
    core::component::shielded_pool::v1::{
        self as pb, event_outbound_fungible_token_refund::Reason as RefundReason,
    },
    event::ProtoEvent as _,
};

/// The kind of event we might care about.
#[derive(Clone, Copy, Debug)]
enum EventKind {
    InboundTransfer,
    OutboundTransfer,
    OutboundRefund,
}

impl EventKind {
    fn tag(&self) -> &'static str {
        match self {
            Self::InboundTransfer => {
                "penumbra.core.component.shielded_pool.v1.EventInboundFungibleTokenTransfer"
            }
            Self::OutboundTransfer => {
                "penumbra.core.component.shielded_pool.v1.EventOutboundFungibleTokenTransfer"
            }
            Self::OutboundRefund => {
                "penumbra.core.component.shielded_pool.v1.EventOutboundFungibleTokenRefund"
            }
        }
    }
}

impl TryFrom<&str> for EventKind {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        for kind in [
            Self::InboundTransfer,
            Self::OutboundTransfer,
            Self::OutboundRefund,
        ] {
            if kind.tag() == value {
                return Ok(kind);
            }
        }
        Err(anyhow!("unexpected event kind: {value}"))
    }
}

/// Represents the event data that we care about.
#[derive(Debug, Clone)]
enum Event {
    InboundTransfer {
        receiver: Address,
        sender: String,
        value: Value,
    },
    OutboundTransfer {
        sender: Address,
        receiver: String,
        value: Value,
    },
    OutboundRefund {
        sender: Address,
        receiver: String,
        value: Value,
        reason: RefundReason,
    },
}

impl TryFrom<&ContextualizedEvent> for Event {
    type Error = anyhow::Error;

    fn try_from(event: &ContextualizedEvent) -> Result<Self, Self::Error> {
        match EventKind::try_from(event.event.kind.as_str())? {
            EventKind::InboundTransfer => {
                let pe = pb::EventInboundFungibleTokenTransfer::from_event(&event.event)?;
                Ok(Self::InboundTransfer {
                    receiver: pe.receiver.ok_or(anyhow!("missing receiver"))?.try_into()?,
                    sender: pe.sender,
                    value: pe.value.ok_or(anyhow!("missing value"))?.try_into()?,
                })
            }
            EventKind::OutboundTransfer => {
                let pe = pb::EventOutboundFungibleTokenTransfer::from_event(&event.event)?;
                Ok(Self::OutboundTransfer {
                    sender: pe.sender.ok_or(anyhow!("missing sender"))?.try_into()?,
                    receiver: pe.receiver,
                    value: pe.value.ok_or(anyhow!("missing value"))?.try_into()?,
                })
            }
            EventKind::OutboundRefund => {
                let pe = pb::EventOutboundFungibleTokenRefund::from_event(&event.event)?;
                let reason = pe.reason();
                Ok(Self::OutboundRefund {
                    sender: pe.sender.ok_or(anyhow!("missing sender"))?.try_into()?,
                    receiver: pe.receiver,
                    value: pe.value.ok_or(anyhow!("missing value"))?.try_into()?,
                    reason,
                })
            }
        }
    }
}
