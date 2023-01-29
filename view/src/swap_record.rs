use penumbra_chain::NoteSource;
use penumbra_crypto::{
    dex::{swap::SwapPlaintext, BatchSwapOutputData},
    Nullifier,
};
use penumbra_proto::{view::v1alpha1 as pb, Protobuf};
use penumbra_tct as tct;

use serde::{Deserialize, Serialize};
use sqlx::Row;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(try_from = "pb::SwapRecord", into = "pb::SwapRecord")]
pub struct SwapRecord {
    pub swap_commitment: tct::Commitment,
    pub swap: SwapPlaintext,
    pub position: tct::Position,
    pub nullifier: Nullifier,
    pub output_data: BatchSwapOutputData,
    pub height_claimed: Option<u64>,
    pub source: NoteSource,
}

impl Protobuf for SwapRecord {
    type Proto = pb::SwapRecord;
}
impl From<SwapRecord> for pb::SwapRecord {
    fn from(msg: SwapRecord) -> Self {
        pb::SwapRecord {
            swap_commitment: Some(msg.swap_commitment.into()),
            swap: Some(msg.swap.into()),
            position: msg.position.into(),
            nullifier: Some(msg.nullifier.into()),
            output_data: Some(msg.output_data.into()),
            height_claimed: msg.height_claimed,
            source: Some(msg.source.into()),
        }
    }
}

impl TryFrom<pb::SwapRecord> for SwapRecord {
    type Error = anyhow::Error;
    fn try_from(value: pb::SwapRecord) -> Result<Self, Self::Error> {
        Ok(Self {
            swap_commitment: value
                .swap_commitment
                .ok_or_else(|| anyhow::anyhow!("missing swap_commitment"))?
                .try_into()?,
            swap: value
                .swap
                .ok_or_else(|| anyhow::anyhow!("missing swap"))?
                .try_into()?,
            position: value.position.into(),
            nullifier: value
                .nullifier
                .ok_or_else(|| anyhow::anyhow!("missing nullifier"))?
                .try_into()?,
            output_data: value
                .output_data
                .ok_or_else(|| anyhow::anyhow!("missing output_data"))?
                .try_into()?,
            height_claimed: value.height_claimed,
            source: value
                .source
                .ok_or_else(|| anyhow::anyhow!("missing source"))?
                .try_into()?,
        })
    }
}

impl<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> for SwapRecord {
    fn from_row(row: &'r sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        // This is not a fun time.
        // Mostly on account of sqlx::Error.

        let swap_commitment = tct::Commitment::try_from(row.get::<'r, &[u8], _>("swap_commitment"))
            .map_err(|e| sqlx::Error::ColumnDecode {
                index: "swap_commitment".to_string(),
                source: e.into(),
            })?;
        let swap = SwapPlaintext::decode(row.get::<'r, &[u8], _>("swap")).map_err(|e| {
            sqlx::Error::ColumnDecode {
                index: "swap".to_string(),
                source: e.into(),
            }
        })?;
        let output_data = BatchSwapOutputData::decode(row.get::<'r, &[u8], _>("output_data"))
            .map_err(|e| sqlx::Error::ColumnDecode {
                index: "output_data".to_string(),
                source: e.into(),
            })?;
        let nullifier = Nullifier::try_from(row.get::<'r, &[u8], _>("nullifier")).map_err(|e| {
            sqlx::Error::ColumnDecode {
                index: "nullifier".to_string(),
                source: e.into(),
            }
        })?;
        let height_claimed = row
            .get::<'r, Option<i64>, _>("height_claimed")
            .map(|v| v as u64);
        let position = (row.get::<'r, i64, _>("position") as u64).into();
        let source = NoteSource::try_from(row.get::<'r, &[u8], _>("source")).map_err(|e| {
            sqlx::Error::ColumnDecode {
                index: "source".to_string(),
                source: e.into(),
            }
        })?;

        Ok(SwapRecord {
            swap_commitment,
            swap,
            output_data,
            nullifier,
            height_claimed,
            position,
            source,
        })
    }
}
