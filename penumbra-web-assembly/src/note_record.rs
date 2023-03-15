use penumbra_chain::NoteSource;
use penumbra_crypto::{
    asset, keys::AddressIndex, note, Address, FieldExt, Fq, Note, Nullifier, Rseed, Value,
};
use penumbra_proto::{view::v1alpha1 as pb, DomainType};
use penumbra_tct as tct;
use std::convert::{TryFrom, TryInto};

use serde::{Deserialize, Serialize};

/// Corresponds to the SpendableNoteRecord proto
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(try_from = "pb::SpendableNoteRecord", into = "pb::SpendableNoteRecord")]
pub struct SpendableNoteRecord {
    pub note_commitment: note::Commitment,
    pub note: Note,
    pub address_index: AddressIndex,
    pub nullifier: Nullifier,
    pub height_created: u64,
    pub height_spent: Option<u64>,
    pub position: tct::Position,
    pub source: NoteSource,
}

impl DomainType for SpendableNoteRecord {
    type Proto = pb::SpendableNoteRecord;
}
impl From<SpendableNoteRecord> for pb::SpendableNoteRecord {
    fn from(v: SpendableNoteRecord) -> Self {
        pb::SpendableNoteRecord {
            note_commitment: Some(v.note_commitment.into()),
            note: Some(v.note.into()),
            address_index: Some(v.address_index.into()),
            nullifier: Some(v.nullifier.into()),
            height_created: v.height_created,
            height_spent: v.height_spent,
            position: v.position.into(),
            source: Some(v.source.into()),
        }
    }
}

impl TryFrom<pb::SpendableNoteRecord> for SpendableNoteRecord {
    type Error = anyhow::Error;
    fn try_from(v: pb::SpendableNoteRecord) -> Result<Self, Self::Error> {
        Ok(SpendableNoteRecord {
            note_commitment: v
                .note_commitment
                .ok_or_else(|| anyhow::anyhow!("missing note commitment"))?
                .try_into()?,
            note: v
                .note
                .ok_or_else(|| anyhow::anyhow!("missing note"))?
                .try_into()?,
            address_index: v
                .address_index
                .ok_or_else(|| anyhow::anyhow!("missing address index"))?
                .try_into()?,
            nullifier: v
                .nullifier
                .ok_or_else(|| anyhow::anyhow!("missing nullifier"))?
                .try_into()?,
            height_created: v.height_created,
            height_spent: v.height_spent,
            position: v.position.into(),
            source: v
                .source
                .ok_or_else(|| anyhow::anyhow!("missing note source"))?
                .try_into()?,
        })
    }
}

