use penumbra_chain::NoteSource;
use penumbra_dex::{swap::SwapPlaintext, BatchSwapOutputData};
use penumbra_proto::{view::v1alpha1 as pb, DomainType, TypeUrl};
use penumbra_sct::Nullifier;
use penumbra_tct as tct;
use penumbra_transaction::Id as TxId;

use r2d2_sqlite::rusqlite::Row;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(try_from = "pb::SwapRecord", into = "pb::SwapRecord")]
pub struct SwapRecord {
    pub swap_commitment: tct::StateCommitment,
    pub swap: SwapPlaintext,
    pub position: tct::Position,
    pub nullifier: Nullifier,
    pub output_data: BatchSwapOutputData,
    pub height_claimed: Option<u64>,
    pub swap_tx_id: TxId,
    pub claim_tx_id: Option<TxId>,
}

impl TypeUrl for SwapRecord {
    const TYPE_URL: &'static str = "/penumbra.view.v1alpha1.SwapRecord";
}

impl DomainType for SwapRecord {
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
            height_claimed: match msg.height_claimed {
                Some(h) => h,
                None => 0,
            },
            swap_tx_id: Some(msg.swap_tx_id.into()),
            claim_tx_id: msg.claim_tx_id.map(Into::into),
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
            height_claimed: if value.height_claimed > 0 {
                Some(value.height_claimed)
            } else {
                None
            },
            swap_tx_id: value
                .swap_tx_id
                .ok_or_else(|| anyhow::anyhow!("missing swap_tx_id"))?
                .try_into()?,
            claim_tx_id: value.claim_tx_id.map(|id| id.try_into()).transpose()?,
        })
    }
}

impl TryFrom<&Row<'_>> for SwapRecord {
    type Error = anyhow::Error;

    fn try_from(row: &Row<'_>) -> Result<Self, Self::Error> {
        Ok(Self {
            swap_commitment: row.get::<_, Vec<u8>>("swap_commitment")?[..].try_into()?,
            height_claimed: row.get("height_claimed")?,
            position: row.get::<_, u64>("position")?.into(),
            nullifier: row.get::<_, Vec<u8>>("nullifier")?[..].try_into()?,
            output_data: BatchSwapOutputData::decode(&row.get::<_, Vec<u8>>("output_data")?[..])?,
            swap: SwapPlaintext::decode(&row.get::<_, Vec<u8>>("swap")?[..])?,
            swap_tx_id: row.get::<_, Vec<u8>>("swap_tx_id")?[..].try_into()?,
            claim_tx_id: row
                .get::<_, Option<Vec<u8>>>("claim_tx_id")?
                .map(|bytes| bytes[..].try_into())
                .transpose()?,
        })
    }
}
