use penumbra_chain::NoteSource;
use penumbra_crypto::{
    asset, keys::AddressIndex, note, Address, FieldExt, Fq, IdentityKey, Note, PayloadKey, Value,
};
use penumbra_proto::{view::v1alpha1 as pb, Protobuf};

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
    pub address_index: AddressIndex,
    pub height_created: u64,
    pub unbonding_epoch: u64,
    pub identity_key: IdentityKey,
    pub source: NoteSource,
    pub payload_key: PayloadKey,
}

impl Protobuf<pb::QuarantinedNoteRecord> for QuarantinedNoteRecord {}
impl From<QuarantinedNoteRecord> for pb::QuarantinedNoteRecord {
    fn from(v: QuarantinedNoteRecord) -> Self {
        pb::QuarantinedNoteRecord {
            note_commitment: Some(v.note_commitment.into()),
            note: Some(v.note.into()),
            address_index: Some(v.address_index.into()),
            height_created: v.height_created,
            unbonding_epoch: v.unbonding_epoch,
            identity_key: Some(v.identity_key.into()),
            source: Some(v.source.into()),
            payload_key: Some(v.payload_key.into()),
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
            address_index: v
                .address_index
                .ok_or_else(|| anyhow::anyhow!("missing address index"))?
                .try_into()?,
            height_created: v.height_created,
            unbonding_epoch: v.unbonding_epoch,
            identity_key: v
                .identity_key
                .ok_or_else(|| anyhow::anyhow!("missing identity key"))?
                .try_into()?,
            source: v
                .source
                .ok_or_else(|| anyhow::anyhow!("missing note source"))?
                .try_into()?,
            payload_key: v
                .payload_key
                .ok_or_else(|| anyhow::anyhow!("missing payload key"))?
                .try_into()?,
        })
    }
}

impl<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> for QuarantinedNoteRecord {
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

        let value = Value {
            amount: amount.into(),
            asset_id,
        };
        let note = Note::from_parts(address, value, note_blinding).map_err(|e| {
            sqlx::Error::ColumnDecode {
                index: "note".to_string(),
                source: e.into(),
            }
        })?;

        let source = NoteSource::try_from(row.get::<'r, &[u8], _>("source")).map_err(|e| {
            sqlx::Error::ColumnDecode {
                index: "source".to_string(),
                source: e.into(),
            }
        })?;

        let payload_key =
            PayloadKey::try_from(row.get::<'r, Vec<u8>, _>("payload_key")).map_err(|e| {
                sqlx::Error::ColumnDecode {
                    index: "payload_key".to_string(),
                    source: e.into(),
                }
            })?;

        Ok(QuarantinedNoteRecord {
            note_commitment,
            note,
            address_index,
            height_created,
            unbonding_epoch,
            identity_key,
            source,
            payload_key,
        })
    }
}
