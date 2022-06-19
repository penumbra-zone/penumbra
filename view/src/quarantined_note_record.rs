use penumbra_crypto::{
    asset,
    ka::Public,
    keys::{Diversifier, DiversifierIndex},
    note, FieldExt, Fq, IdentityKey, Note, Value,
};
use penumbra_proto::{view as pb, Protobuf};

use serde::{Deserialize, Serialize};
use sqlx::Row;

/// Corresponds to the QuarantinedNoteRecord proto
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(
    try_from = "pb::QuarantinedNoteRecord",
    into = "pb::QuarantinedNoteRecord"
)]
pub struct QuarantinedNoteRecord {
    pub note_commitment: note::Commitment,
    pub note: Note,
    pub diversifier_index: DiversifierIndex,
    pub height_created: u64,
    pub unbonding_epoch: u64,
    pub identity_key: IdentityKey,
}

impl Protobuf<pb::QuarantinedNoteRecord> for QuarantinedNoteRecord {}
impl From<QuarantinedNoteRecord> for pb::QuarantinedNoteRecord {
    fn from(v: QuarantinedNoteRecord) -> Self {
        pb::QuarantinedNoteRecord {
            note_commitment: Some(v.note_commitment.into()),
            note: Some(v.note.into()),
            diversifier_index: Some(v.diversifier_index.into()),
            height_created: v.height_created,
            unbonding_epoch: v.unbonding_epoch,
            identity_key: Some(v.identity_key.into()),
        }
    }
}

impl TryFrom<pb::QuarantinedNoteRecord> for QuarantinedNoteRecord {
    type Error = anyhow::Error;
    fn try_from(v: pb::QuarantinedNoteRecord) -> Result<Self, Self::Error> {
        Ok(QuarantinedNoteRecord {
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
            height_created: v.height_created,
            unbonding_epoch: v.unbonding_epoch,
            identity_key: v
                .identity_key
                .ok_or_else(|| anyhow::anyhow!("missing identity key"))?
                .try_into()?,
        })
    }
}

impl<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> for QuarantinedNoteRecord {
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

        let diversifier_index = DiversifierIndex::try_from(
            row.get::<'r, &[u8], _>("diversifier_index"),
        )
        .map_err(|e| sqlx::Error::ColumnDecode {
            index: "diversifier_index".to_string(),
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

        let height_created = row.get::<'r, i64, _>("height_created") as u64;

        let identity_key =
            IdentityKey::decode(row.get::<'r, &[u8], _>("identity_key")).map_err(|e| {
                sqlx::Error::ColumnDecode {
                    index: "identity_key".to_string(),
                    source: e.into(),
                }
            })?;

        let unbonding_epoch = row.get::<'r, i64, _>("unbonding_epoch") as u64;

        let value = Value { amount, asset_id };
        let note =
            Note::from_parts(diversifier, transmission_key, value, note_blinding).map_err(|e| {
                sqlx::Error::ColumnDecode {
                    index: "note".to_string(),
                    source: e.into(),
                }
            })?;

        Ok(QuarantinedNoteRecord {
            note_commitment,
            note,
            diversifier_index,
            height_created,
            unbonding_epoch,
            identity_key,
        })
    }
}
