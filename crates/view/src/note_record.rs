use penumbra_asset::Value;
use penumbra_chain::NoteSource;
use penumbra_keys::keys::AddressIndex;
use penumbra_proto::{view::v1alpha1 as pb, DomainType, TypeUrl};
use penumbra_sct::Nullifier;
use penumbra_shielded_pool::{note, Note, Rseed};
use penumbra_tct as tct;

use r2d2_sqlite::rusqlite::Row;
use serde::{Deserialize, Serialize};

/// Corresponds to the SpendableNoteRecord proto
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(try_from = "pb::SpendableNoteRecord", into = "pb::SpendableNoteRecord")]
pub struct SpendableNoteRecord {
    pub note_commitment: note::StateCommitment,
    pub note: Note,
    pub address_index: AddressIndex,
    pub nullifier: Nullifier,
    pub height_created: u64,
    pub height_spent: Option<u64>,
    pub position: tct::Position,
    pub source: NoteSource,
}

impl TypeUrl for SpendableNoteRecord {
    const TYPE_URL: &'static str = "/penumbra.view.v1alpha1.SpendableNoteRecord";
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

impl TryFrom<&Row<'_>> for SpendableNoteRecord {
    type Error = anyhow::Error;

    fn try_from(row: &Row<'_>) -> Result<Self, Self::Error> {
        Ok(SpendableNoteRecord {
            address_index: row.get::<_, Vec<u8>>("address_index")?[..].try_into()?,
            nullifier: row.get::<_, Vec<u8>>("nullifier")?[..].try_into()?,
            height_created: row.get("height_created")?,
            height_spent: row.get("height_spent")?,
            position: row.get::<_, u64>("position")?.into(),
            source: row.get::<_, Vec<u8>>("source")?[..].try_into()?,
            note_commitment: row.get::<_, Vec<u8>>("note_commitment")?[..].try_into()?,
            note: Note::from_parts(
                row.get::<_, Vec<u8>>("address")?[..].try_into()?,
                Value {
                    amount: u128::from_be_bytes(row.get::<_, [u8; 16]>("amount")?).into(),
                    asset_id: row.get::<_, Vec<u8>>("asset_id")?[..].try_into()?,
                },
                Rseed(row.get::<_, [u8; 32]>("rseed")?),
            )?,
        })
    }
}
