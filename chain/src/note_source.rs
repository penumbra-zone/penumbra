use anyhow::{anyhow, Result};
use penumbra_proto::{core::chain::v1alpha1 as pb, Protobuf};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(try_from = "pb::NoteSource", into = "pb::NoteSource")]
pub enum NoteSource {
    Transaction { id: [u8; 32] },
    Unknown,
    Genesis,
    FundingStreamReward { epoch_index: u64 },
    ProposalDepositRefund { proposal_id: u64 },
}

impl Default for NoteSource {
    fn default() -> Self {
        Self::Unknown
    }
}

const CODE_INDEX: usize = 23;

impl NoteSource {
    pub fn to_bytes(&self) -> [u8; 32] {
        match self {
            Self::Transaction { id } => *id,
            Self::Unknown => [0; 32],
            Self::Genesis => {
                let mut bytes = [0u8; 32];
                bytes[CODE_INDEX] = 1;
                bytes
            }
            Self::FundingStreamReward { epoch_index } => {
                let mut bytes = [0u8; 32];
                bytes[CODE_INDEX] = 2;
                bytes[24..].copy_from_slice(&epoch_index.to_le_bytes());
                bytes
            }
            Self::ProposalDepositRefund { proposal_id } => {
                let mut bytes = [0u8; 32];
                bytes[CODE_INDEX] = 3;
                bytes[24..].copy_from_slice(&proposal_id.to_le_bytes());
                bytes
            }
        }
    }
}

impl TryFrom<[u8; 32]> for NoteSource {
    type Error = anyhow::Error;
    fn try_from(bytes: [u8; 32]) -> Result<Self> {
        if bytes[..CODE_INDEX] != [0u8; CODE_INDEX][..] {
            Ok(Self::Transaction { id: bytes })
        } else {
            match (bytes[CODE_INDEX], &bytes[CODE_INDEX + 1..]) {
                (0, &[0, 0, 0, 0, 0, 0, 0, 0]) => Ok(Self::Unknown),
                (1, &[0, 0, 0, 0, 0, 0, 0, 0]) => Ok(Self::Genesis),
                (2, epoch_bytes) => {
                    let epoch_index =
                        u64::from_le_bytes(epoch_bytes.try_into().expect("slice is of length 8"));
                    Ok(Self::FundingStreamReward { epoch_index })
                }
                (3, proposal_id_bytes) => {
                    let proposal_id = u64::from_le_bytes(
                        proposal_id_bytes.try_into().expect("slice is of length 8"),
                    );
                    Ok(Self::ProposalDepositRefund { proposal_id })
                }
                (code, data) => Err(anyhow!(
                    "unknown note source with code {} and data {:?}",
                    code,
                    data
                )),
            }
        }
    }
}

impl TryFrom<&[u8]> for NoteSource {
    type Error = anyhow::Error;
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        <[u8; 32]>::try_from(value)?.try_into()
    }
}

impl Protobuf<pb::NoteSource> for NoteSource {}

impl TryFrom<pb::NoteSource> for NoteSource {
    type Error = anyhow::Error;
    fn try_from(note_source: pb::NoteSource) -> Result<Self> {
        <[u8; 32]>::try_from(note_source.inner)
            .map_err(|_| anyhow!("expected 32 bytes"))?
            .try_into()
    }
}

impl From<NoteSource> for pb::NoteSource {
    fn from(note_source: NoteSource) -> Self {
        pb::NoteSource {
            inner: note_source.to_bytes().to_vec(),
        }
    }
}

impl std::fmt::Debug for NoteSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NoteSource::Transaction { id } => {
                f.write_fmt(format_args!("NoteSource::Transaction({})", hex::encode(id)))
            }
            NoteSource::Genesis => f.write_fmt(format_args!("NoteSource::Genesis")),
            NoteSource::Unknown => f.write_fmt(format_args!("NoteSource::Unknown")),
            NoteSource::FundingStreamReward { epoch_index } => f.write_fmt(format_args!(
                "NoteSource::FundingStreamReward({})",
                epoch_index
            )),
            NoteSource::ProposalDepositRefund { proposal_id } => f.write_fmt(format_args!(
                "NoteSource::ProposalDepositRefund({})",
                proposal_id
            )),
        }
    }
}
