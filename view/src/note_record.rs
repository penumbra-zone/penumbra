use penumbra_crypto::{keys::DiversifierIndex, note, Note, Nullifier};
use penumbra_proto::{view as pb, Protobuf};

use serde::{Deserialize, Serialize};

/// Corresponds to the NoteRecord proto
#[derive(Serialize, Deserialize, Clone)]
#[serde(try_from = "pb::NoteRecord", into = "pb::NoteRecord")]
pub struct NoteRecord {
    pub note_commitment: note::Commitment,
    pub note: Note,
    pub diversifier_index: DiversifierIndex,
    pub nullifier: Nullifier,
    pub height_created: u64,
    pub height_spent: Option<u64>,
}

impl Protobuf<pb::NoteRecord> for NoteRecord {}
impl From<NoteRecord> for pb::NoteRecord {
    fn from(v: NoteRecord) -> Self {
        pb::NoteRecord {
            note_commitment: Some(v.note_commitment.into()),
            note: Some(v.note.into()),
            diversifier_index: Some(v.diversifier_index.into()),
            nullifier: Some(v.nullifier.into()),
            height_created: v.height_created,
            height_spent: v.height_spent,
        }
    }
}

impl TryFrom<pb::NoteRecord> for NoteRecord {
    type Error = anyhow::Error;
    fn try_from(v: pb::NoteRecord) -> Result<Self, Self::Error> {
        Ok(NoteRecord {
            note_commitment: v
                .note_commitment
                .ok_or_else(|| anyhow::anyhow!("missing note commitment"))?
                .try_into()?,
            note: v
                .note
                .ok_or_else(|| anyhow::anyhow!("missing note"))?
                .try_into()?,
            diversifier_index: v
                .diversifier_index
                .ok_or_else(|| anyhow::anyhow!("missing diversifier index"))?
                .try_into()?,
            nullifier: v
                .nullifier
                .ok_or_else(|| anyhow::anyhow!("missing nullifier"))?
                .try_into()?,
            height_created: v.height_created,
            height_spent: v.height_spent,
        })
    }
}
