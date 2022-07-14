use penumbra_crypto::{
    asset,
    ka::Public,
    keys::{AddressIndex, Diversifier},
    note, FieldExt, Fq, Note, Nullifier, Value,
};
use penumbra_proto::{view as pb, Protobuf};
use penumbra_tct as tct;

use serde::{Deserialize, Serialize};
use sqlx::Row;

/// Corresponds to the NoteRecord proto
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(try_from = "pb::NoteRecord", into = "pb::NoteRecord")]
pub struct NoteRecord {
    pub note_commitment: note::Commitment,
    pub note: Note,
    pub address_index: AddressIndex,
    pub nullifier: Nullifier,
    pub height_created: u64,
    pub height_spent: Option<u64>,
    pub position: tct::Position,
}

impl Protobuf<pb::NoteRecord> for NoteRecord {}
impl From<NoteRecord> for pb::NoteRecord {
    fn from(v: NoteRecord) -> Self {
        pb::NoteRecord {
            note_commitment: Some(v.note_commitment.into()),
            note: Some(v.note.into()),
            address_index: Some(v.address_index.into()),
            nullifier: Some(v.nullifier.into()),
            height_created: v.height_created,
            height_spent: v.height_spent,
            position: v.position.into(),
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
        })
    }
}

impl<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> for NoteRecord {
    fn from_row(row: &'r sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        // This is not a fun time.
        // Mostly on account of sqlx::Error.

        let diversifier =
            Diversifier::try_from(row.get::<'r, &[u8], _>("diversifier")).map_err(|e| {
                sqlx::Error::ColumnDecode {
                    index: "diversifier".to_string(),
                    source: e.into(),
                }
            })?;

        let address_index = AddressIndex::try_from(row.get::<'r, &[u8], _>("address_index"))
            .map_err(|e| sqlx::Error::ColumnDecode {
                index: "address_index".to_string(),
                source: e.into(),
            })?;

        let transmission_key = Public(
            <[u8; 32]>::try_from(row.get::<'r, &[u8], _>("transmission_key")).map_err(|e| {
                sqlx::Error::ColumnDecode {
                    index: "transmission_key".to_string(),
                    source: e.into(),
                }
            })?,
        );

        let amount = row.get::<'r, i64, _>("amount") as u64;

        let asset_id = asset::Id(
            Fq::from_bytes(
                <[u8; 32]>::try_from(row.get::<'r, &[u8], _>("asset_id")).map_err(|e| {
                    sqlx::Error::ColumnDecode {
                        index: "asset_id".to_string(),
                        source: e.into(),
                    }
                })?,
            )
            .map_err(|e| sqlx::Error::ColumnDecode {
                index: "asset_id".to_string(),
                source: e.into(),
            })?,
        );

        let note_blinding = Fq::from_bytes(
            <[u8; 32]>::try_from(row.get::<'r, &[u8], _>("blinding_factor")).map_err(|e| {
                sqlx::Error::ColumnDecode {
                    index: "blinding_factor".to_string(),
                    source: e.into(),
                }
            })?,
        )
        .map_err(|e| sqlx::Error::ColumnDecode {
            index: "blinding_factor".to_string(),
            source: e.into(),
        })?;

        let note_commitment = note::Commitment::try_from(
            row.get::<'r, &[u8], _>("note_commitment"),
        )
        .map_err(|e| sqlx::Error::ColumnDecode {
            index: "note_commitment".to_string(),
            source: e.into(),
        })?;

        let nullifier = Nullifier::try_from(row.get::<'r, &[u8], _>("nullifier")).map_err(|e| {
            sqlx::Error::ColumnDecode {
                index: "nullifier".to_string(),
                source: e.into(),
            }
        })?;

        let height_created = row.get::<'r, i64, _>("height_created") as u64;
        let height_spent = row
            .get::<'r, Option<i64>, _>("height_spent")
            .map(|v| v as u64);
        let position = (row.get::<'r, i64, _>("position") as u64).into();

        let value = Value { amount, asset_id };
        let note =
            Note::from_parts(diversifier, transmission_key, value, note_blinding).map_err(|e| {
                sqlx::Error::ColumnDecode {
                    index: "note".to_string(),
                    source: e.into(),
                }
            })?;

        Ok(NoteRecord {
            note_commitment,
            note,
            address_index,
            nullifier,
            position,
            height_created,
            height_spent,
        })
    }
}
