//! TODO: rename to something more generic ("minted notes"?) that can
//! be used with IBC transfers, and fix up the proto location

use anyhow::Result;
use penumbra_crypto::{Address, Amount};
use penumbra_proto::{stake as pb, Protobuf};
use serde::{Deserialize, Serialize};

/// A commission amount to be minted as part of processing the epoch transition.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::CommissionAmount", into = "pb::CommissionAmount")]
pub struct CommissionAmount {
    pub amount: Amount,
    pub destination: Address,
}

impl Protobuf<pb::CommissionAmount> for CommissionAmount {}

impl From<CommissionAmount> for pb::CommissionAmount {
    fn from(note: CommissionAmount) -> pb::CommissionAmount {
        pb::CommissionAmount {
            amount: Some(note.amount.into()),
            destination: Some(note.destination.into()),
        }
    }
}

impl TryFrom<pb::CommissionAmount> for CommissionAmount {
    type Error = anyhow::Error;
    fn try_from(note: pb::CommissionAmount) -> Result<CommissionAmount> {
        Ok(CommissionAmount {
            amount: note
                .amount
                .ok_or_else(|| anyhow::anyhow!("missing amount"))?
                .try_into()?,
            destination: note
                .destination
                .ok_or_else(|| anyhow::anyhow!("missing destination address"))?
                .try_into()?,
        })
    }
}

/// A list of commission amounts to be minted as part of processing the epoch transition.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(try_from = "pb::CommissionAmounts", into = "pb::CommissionAmounts")]
pub struct CommissionAmounts {
    pub notes: Vec<CommissionAmount>,
}

impl Protobuf<pb::CommissionAmounts> for CommissionAmounts {}

impl From<CommissionAmounts> for pb::CommissionAmounts {
    fn from(notes: CommissionAmounts) -> pb::CommissionAmounts {
        pb::CommissionAmounts {
            notes: notes.notes.into_iter().map(Into::into).collect(),
        }
    }
}

impl TryFrom<pb::CommissionAmounts> for CommissionAmounts {
    type Error = anyhow::Error;
    fn try_from(notes: pb::CommissionAmounts) -> Result<CommissionAmounts> {
        Ok(CommissionAmounts {
            notes: notes
                .notes
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
        })
    }
}
