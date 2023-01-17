use decaf377::{FieldExt, Fr};
use penumbra_crypto::Note;
use penumbra_proto::{core::governance::v1alpha1 as pb, Protobuf};
use penumbra_tct as tct;
use serde::{Deserialize, Serialize};

/// A plan to vote as a delegator.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::DelegatorVotePlan", into = "pb::DelegatorVotePlan")]
pub struct DelegatorVotePlan {
    /// The proposal ID to vote on.
    pub proposal: u64,
    /// The vote to cast.
    pub vote: crate::action::Vote,
    /// A staked note that was spendable before the proposal started.
    pub staked_note: Note,
    /// The position of the staked note.
    pub position: tct::Position,
    /// The randomizer to use.
    pub randomizer: Fr,
}

impl From<DelegatorVotePlan> for pb::DelegatorVotePlan {
    fn from(inner: DelegatorVotePlan) -> Self {
        pb::DelegatorVotePlan {
            proposal: inner.proposal,
            vote: Some(inner.vote.into()),
            staked_note: Some(inner.staked_note.into()),
            position: inner.position.into(),
            randomizer: inner.randomizer.to_bytes().to_vec().into(),
        }
    }
}

impl TryFrom<pb::DelegatorVotePlan> for DelegatorVotePlan {
    type Error = anyhow::Error;

    fn try_from(value: pb::DelegatorVotePlan) -> Result<Self, Self::Error> {
        Ok(DelegatorVotePlan {
            proposal: value.proposal,
            vote: value
                .vote
                .ok_or_else(|| anyhow::anyhow!("missing vote in `DelegatorVotePlan`"))?
                .try_into()?,
            staked_note: value
                .staked_note
                .ok_or_else(|| anyhow::anyhow!("missing staked note in `DelegatorVotePlan`"))?
                .try_into()?,
            position: value.position.into(),
            randomizer: Fr::from_bytes(
                value
                    .randomizer
                    .try_into()
                    .map_err(|_| anyhow::anyhow!("invalid randomizer"))?,
            )?,
        })
    }
}

impl Protobuf<pb::DelegatorVotePlan> for DelegatorVotePlan {}
