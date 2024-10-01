use penumbra_asset::Value;
use penumbra_keys::Address;
use penumbra_proto::core::component::shielded_pool::v1::{
    EventInboundFungibleTokenTransfer, EventOutboundFungibleTokenRefund,
    EventOutboundFungibleTokenTransfer, EventOutput, EventSpend,
};
use penumbra_sct::Nullifier;

use crate::NotePayload;

// These are sort of like the proto/domain type From impls, because
// we don't have separate domain types for the events (yet, possibly ever).

pub fn spend(nullifier: &Nullifier) -> EventSpend {
    EventSpend {
        nullifier: Some((*nullifier).into()),
    }
}

pub fn output(note_payload: &NotePayload) -> EventOutput {
    EventOutput {
        note_commitment: Some(note_payload.note_commitment.into()),
    }
}

pub fn outbound_fungible_token_transfer(
    value: Value,
    sender: Address,
    receiver: String,
) -> EventOutboundFungibleTokenTransfer {
    EventOutboundFungibleTokenTransfer {
        value: Some(value.into()),
        sender: Some(sender.into()),
        receiver,
    }
}

pub fn inbound_fungible_token_transfer(
    value: Value,
    sender: String,
    receiver: Address,
) -> EventInboundFungibleTokenTransfer {
    EventInboundFungibleTokenTransfer {
        value: Some(value.into()),
        sender,
        receiver: Some(receiver.into()),
    }
}
