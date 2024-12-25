use anyhow::{anyhow, Context};
use penumbra_sdk_asset::Value;
use penumbra_sdk_keys::Address;
use penumbra_sdk_proto::{core::component::shielded_pool::v1 as pb, DomainType};
use penumbra_sdk_sct::Nullifier;
use prost::Name as _;

use crate::note::StateCommitment;

// // These are sort of like the proto/domain type From impls, because
// // we don't have separate domain types for the events (yet, possibly ever).
// Narrator: we did in fact need the separate domain types.

#[derive(Clone, Debug)]
pub struct EventSpend {
    pub nullifier: Nullifier,
}

impl TryFrom<pb::EventSpend> for EventSpend {
    type Error = anyhow::Error;

    fn try_from(value: pb::EventSpend) -> Result<Self, Self::Error> {
        fn inner(value: pb::EventSpend) -> anyhow::Result<EventSpend> {
            Ok(EventSpend {
                nullifier: value
                    .nullifier
                    .ok_or(anyhow!("missing `nullifier`"))?
                    .try_into()?,
            })
        }
        inner(value).context(format!("parsing {}", pb::EventSpend::NAME))
    }
}

impl From<EventSpend> for pb::EventSpend {
    fn from(value: EventSpend) -> Self {
        Self {
            nullifier: Some(value.nullifier.into()),
        }
    }
}

impl DomainType for EventSpend {
    type Proto = pb::EventSpend;
}

#[derive(Clone, Debug)]
pub struct EventOutput {
    pub note_commitment: StateCommitment,
}

impl TryFrom<pb::EventOutput> for EventOutput {
    type Error = anyhow::Error;

    fn try_from(value: pb::EventOutput) -> Result<Self, Self::Error> {
        fn inner(value: pb::EventOutput) -> anyhow::Result<EventOutput> {
            Ok(EventOutput {
                note_commitment: value
                    .note_commitment
                    .ok_or(anyhow!("missing `note_commitment`"))?
                    .try_into()?,
            })
        }
        inner(value).context(format!("parsing {}", pb::EventOutput::NAME))
    }
}

impl From<EventOutput> for pb::EventOutput {
    fn from(value: EventOutput) -> Self {
        Self {
            note_commitment: Some(value.note_commitment.into()),
        }
    }
}

impl DomainType for EventOutput {
    type Proto = pb::EventOutput;
}

#[derive(Clone, Debug)]
pub struct FungibleTokenTransferPacketMetadata {
    pub channel: String,
    pub sequence: u64,
}

impl TryFrom<pb::FungibleTokenTransferPacketMetadata> for FungibleTokenTransferPacketMetadata {
    type Error = anyhow::Error;

    fn try_from(value: pb::FungibleTokenTransferPacketMetadata) -> Result<Self, Self::Error> {
        fn inner(
            value: pb::FungibleTokenTransferPacketMetadata,
        ) -> anyhow::Result<FungibleTokenTransferPacketMetadata> {
            Ok(FungibleTokenTransferPacketMetadata {
                channel: value.channel,
                sequence: value.sequence,
            })
        }
        inner(value).context(format!(
            "parsing {}",
            pb::FungibleTokenTransferPacketMetadata::NAME
        ))
    }
}

impl From<FungibleTokenTransferPacketMetadata> for pb::FungibleTokenTransferPacketMetadata {
    fn from(value: FungibleTokenTransferPacketMetadata) -> Self {
        Self {
            channel: value.channel,
            sequence: value.sequence,
        }
    }
}

impl DomainType for FungibleTokenTransferPacketMetadata {
    type Proto = pb::FungibleTokenTransferPacketMetadata;
}

#[derive(Clone, Debug)]
pub struct EventOutboundFungibleTokenTransfer {
    pub value: Value,
    pub sender: Address,
    pub receiver: String,
    pub meta: FungibleTokenTransferPacketMetadata,
}

impl TryFrom<pb::EventOutboundFungibleTokenTransfer> for EventOutboundFungibleTokenTransfer {
    type Error = anyhow::Error;

    fn try_from(value: pb::EventOutboundFungibleTokenTransfer) -> Result<Self, Self::Error> {
        fn inner(
            value: pb::EventOutboundFungibleTokenTransfer,
        ) -> anyhow::Result<EventOutboundFungibleTokenTransfer> {
            Ok(EventOutboundFungibleTokenTransfer {
                value: value.value.ok_or(anyhow!("missing `value`"))?.try_into()?,
                sender: value
                    .sender
                    .ok_or(anyhow!("missing `sender`"))?
                    .try_into()?,
                receiver: value.receiver,
                meta: value.meta.ok_or(anyhow!("missing `meta`"))?.try_into()?,
            })
        }
        inner(value).context(format!(
            "parsing {}",
            pb::EventOutboundFungibleTokenTransfer::NAME
        ))
    }
}

impl From<EventOutboundFungibleTokenTransfer> for pb::EventOutboundFungibleTokenTransfer {
    fn from(value: EventOutboundFungibleTokenTransfer) -> Self {
        Self {
            value: Some(value.value.into()),
            sender: Some(value.sender.into()),
            receiver: value.receiver,
            meta: Some(value.meta.into()),
        }
    }
}

impl DomainType for EventOutboundFungibleTokenTransfer {
    type Proto = pb::EventOutboundFungibleTokenTransfer;
}

#[derive(Clone, Copy, Debug)]
#[repr(i32)]
pub enum FungibleTokenRefundReason {
    Unspecified = 0,
    Timeout = 1,
    Error = 2,
}

#[derive(Clone, Debug)]
pub struct EventOutboundFungibleTokenRefund {
    pub value: Value,
    pub sender: Address,
    pub receiver: String,
    pub reason: FungibleTokenRefundReason,
    pub meta: FungibleTokenTransferPacketMetadata,
}

impl TryFrom<pb::EventOutboundFungibleTokenRefund> for EventOutboundFungibleTokenRefund {
    type Error = anyhow::Error;

    fn try_from(value: pb::EventOutboundFungibleTokenRefund) -> Result<Self, Self::Error> {
        fn inner(
            value: pb::EventOutboundFungibleTokenRefund,
        ) -> anyhow::Result<EventOutboundFungibleTokenRefund> {
            use pb::event_outbound_fungible_token_refund::Reason;
            let reason = match value.reason() {
                Reason::Timeout => FungibleTokenRefundReason::Timeout,
                Reason::Error => FungibleTokenRefundReason::Error,
                Reason::Unspecified => FungibleTokenRefundReason::Unspecified,
            };
            Ok(EventOutboundFungibleTokenRefund {
                value: value.value.ok_or(anyhow!("missing `value`"))?.try_into()?,
                sender: value
                    .sender
                    .ok_or(anyhow!("missing `sender`"))?
                    .try_into()?,
                receiver: value.receiver,
                reason,
                meta: value.meta.ok_or(anyhow!("missing `meta`"))?.try_into()?,
            })
        }
        inner(value).context(format!(
            "parsing {}",
            pb::EventOutboundFungibleTokenRefund::NAME
        ))
    }
}

impl From<EventOutboundFungibleTokenRefund> for pb::EventOutboundFungibleTokenRefund {
    fn from(value: EventOutboundFungibleTokenRefund) -> Self {
        Self {
            value: Some(value.value.into()),
            sender: Some(value.sender.into()),
            receiver: value.receiver,
            reason: value.reason as i32,
            meta: Some(value.meta.into()),
        }
    }
}

impl DomainType for EventOutboundFungibleTokenRefund {
    type Proto = pb::EventOutboundFungibleTokenRefund;
}

#[derive(Clone, Debug)]
pub struct EventInboundFungibleTokenTransfer {
    pub value: Value,
    pub sender: String,
    pub receiver: Address,
    pub meta: FungibleTokenTransferPacketMetadata,
}

impl TryFrom<pb::EventInboundFungibleTokenTransfer> for EventInboundFungibleTokenTransfer {
    type Error = anyhow::Error;

    fn try_from(value: pb::EventInboundFungibleTokenTransfer) -> Result<Self, Self::Error> {
        fn inner(
            value: pb::EventInboundFungibleTokenTransfer,
        ) -> anyhow::Result<EventInboundFungibleTokenTransfer> {
            Ok(EventInboundFungibleTokenTransfer {
                value: value.value.ok_or(anyhow!("missing `value`"))?.try_into()?,
                sender: value.sender,
                receiver: value
                    .receiver
                    .ok_or(anyhow!("missing `receiver`"))?
                    .try_into()?,
                meta: value.meta.ok_or(anyhow!("missing `meta`"))?.try_into()?,
            })
        }
        inner(value).context(format!(
            "parsing {}",
            pb::EventInboundFungibleTokenTransfer::NAME
        ))
    }
}

impl From<EventInboundFungibleTokenTransfer> for pb::EventInboundFungibleTokenTransfer {
    fn from(value: EventInboundFungibleTokenTransfer) -> Self {
        Self {
            value: Some(value.value.into()),
            sender: value.sender,
            receiver: Some(value.receiver.into()),
            meta: Some(value.meta.into()),
        }
    }
}

impl DomainType for EventInboundFungibleTokenTransfer {
    type Proto = pb::EventInboundFungibleTokenTransfer;
}
