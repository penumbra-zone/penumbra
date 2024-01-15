use anyhow::anyhow;
use penumbra_proto::{core::component::sct::v1alpha1 as pb, DomainType};
use serde::{Deserialize, Serialize};

#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
#[serde(try_from = "pb::CommitmentSource", into = "pb::CommitmentSource")]
pub enum CommitmentSource {
    /// The state commitment was included in the genesis state.
    Genesis,
    /// The commitment was created by a transaction.
    Transaction {
        /// The transaction ID, if specified.
        ///
        /// Making this optional allows the `CompactBlock` to have "stripped" transaction sources,
        /// indicating to the client that they should download and inspect the block's transactions.
        id: Option<[u8; 32]>,
    },
    /// The commitment was created through a validator's funding stream.
    FundingStreamReward { epoch_index: u64 },
    /// The commitment was created through a `CommunityPoolOutput` in a governance-initated transaction.
    CommunityPoolOutput,
    /// The commitment was created by an inbound ICS20 transfer.
    Ics20Transfer {
        /// The sequence number of the transfer packet.
        packet_seq: u64,
        /// The channel the packet was sent on.
        channel_id: String,
        /// The sender address on the counterparty chain.
        sender: String,
    },
}

impl DomainType for CommitmentSource {
    type Proto = pb::CommitmentSource;
}

impl CommitmentSource {
    /// Convenience method for constructing a "stripped" transaction source.
    pub fn transaction() -> Self {
        CommitmentSource::Transaction { id: None }
    }

    /// Convenience method for stripping transaction hashes out of the source.
    pub fn stripped(&self) -> Self {
        match self {
            CommitmentSource::Transaction { .. } => CommitmentSource::Transaction { id: None },
            x => x.clone(),
        }
    }

    /// Get the transaction ID, if this source is a hydrated transaction source.
    pub fn id(&self) -> Option<[u8; 32]> {
        match self {
            CommitmentSource::Transaction { id: Some(id) } => Some(*id),
            _ => None,
        }
    }
}

impl From<CommitmentSource> for pb::CommitmentSource {
    fn from(value: CommitmentSource) -> Self {
        use pb::commitment_source::{self as pbcs, Source};

        Self {
            source: Some(match value {
                CommitmentSource::Genesis => Source::Genesis(pbcs::Genesis {}),
                CommitmentSource::Transaction { id } => Source::Transaction(pbcs::Transaction {
                    id: id.map(|bytes| bytes.to_vec()).unwrap_or_default(),
                }),
                CommitmentSource::FundingStreamReward { epoch_index } => {
                    Source::FundingStreamReward(pbcs::FundingStreamReward { epoch_index })
                }
                CommitmentSource::CommunityPoolOutput => {
                    Source::CommunityPoolOutput(pbcs::CommunityPoolOutput {})
                }
                CommitmentSource::Ics20Transfer {
                    packet_seq,
                    channel_id,
                    sender,
                } => Source::Ics20Transfer(pbcs::Ics20Transfer {
                    packet_seq,
                    channel_id,
                    sender,
                }),
            }),
        }
    }
}

impl TryFrom<pb::CommitmentSource> for CommitmentSource {
    type Error = anyhow::Error;

    fn try_from(value: pb::CommitmentSource) -> Result<Self, Self::Error> {
        use pb::commitment_source::Source;
        let source = value.source.ok_or_else(|| anyhow!("missing source"))?;

        Ok(match source {
            Source::Genesis(_) => Self::Genesis,
            Source::CommunityPoolOutput(_) => Self::CommunityPoolOutput,
            Source::FundingStreamReward(x) => Self::FundingStreamReward {
                epoch_index: x.epoch_index,
            },
            Source::Transaction(x) => {
                if x.id.is_empty() {
                    Self::Transaction { id: None }
                } else {
                    Self::Transaction {
                        id: Some(x.id.try_into().map_err(|id: Vec<u8>| {
                            anyhow!("expected 32-byte id array, got {} bytes", id.len())
                        })?),
                    }
                }
            }
            Source::Ics20Transfer(x) => Self::Ics20Transfer {
                packet_seq: x.packet_seq,
                channel_id: x.channel_id,
                sender: x.sender,
            },
        })
    }
}
