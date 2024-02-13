use penumbra_sct::Nullifier;

use penumbra_proto::core::component::shielded_pool::v1::{EventOutput, EventSpend};

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
