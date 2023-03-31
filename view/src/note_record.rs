use penumbra_chain::NoteSource;
use penumbra_crypto::{
    asset, keys::AddressIndex, note, Address, FieldExt, Fq, Note, Nullifier, Rseed, Value,
};
use penumbra_proto::{view::v1alpha1 as pb, DomainType};
use penumbra_tct as tct;

use serde::{Deserialize, Serialize};
use sqlx::Row;

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

impl<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> for SpendableNoteRecord {
    fn from_row(row: &'r sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        // This is not a fun time.
        // Mostly on account of sqlx::Error.

        let address = Address::try_from(row.get::<'r, &[u8], _>("address")).map_err(|e| {
            sqlx::Error::ColumnDecode {
                index: "address".to_string(),
                source: e.into(),
            }
        })?;

        let address_index = AddressIndex::try_from(row.get::<'r, &[u8], _>("address_index"))
            .map_err(|e| sqlx::Error::ColumnDecode {
                index: "address_index".to_string(),
                source: e.into(),
            })?;

        let amount = <[u8; 16]>::try_from(row.get::<'r, &[u8], _>("amount")).map_err(|e| {
            sqlx::Error::ColumnDecode {
                index: "amount".to_string(),
                source: e.into(),
            }
        })?;

        let amount_u128: u128 = u128::from_be_bytes(amount);

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

        let rseed = Rseed(
            <[u8; 32]>::try_from(row.get::<'r, &[u8], _>("rseed")).map_err(|e| {
                sqlx::Error::ColumnDecode {
                    index: "rseed".to_string(),
                    source: e.into(),
                }
            })?,
        );

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

        let value = Value {
            amount: amount_u128.into(),
            asset_id,
        };
        let note =
            Note::from_parts(address, value, rseed).map_err(|e| sqlx::Error::ColumnDecode {
                index: "note".to_string(),
                source: e.into(),
            })?;

        let source = NoteSource::try_from(row.get::<'r, &[u8], _>("source")).map_err(|e| {
            sqlx::Error::ColumnDecode {
                index: "source".to_string(),
                source: e.into(),
            }
        })?;

        Ok(SpendableNoteRecord {
            note_commitment,
            note,
            address_index,
            nullifier,
            position,
            height_created,
            height_spent,
            source,
        })
    }
}
