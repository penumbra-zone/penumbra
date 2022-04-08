use anyhow::Result;
use penumbra_crypto::Address;
use penumbra_proto::{stake as pb, Protobuf};
use serde::{Deserialize, Serialize};

use crate::{Delegate, Undelegate};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingRewardNote {
    pub amount: u64,
    pub destination: Address,
}

impl Protobuf<pb::PendingRewardNote> for PendingRewardNote {}

impl From<PendingRewardNote> for pb::PendingRewardNote {
    fn from(note: PendingRewardNote) -> pb::PendingRewardNote {
        pb::PendingRewardNote {
            amount: note.amount.into(),
            destination: Some(note.destination.into()),
        }
    }
}

impl TryFrom<pb::PendingRewardNote> for PendingRewardNote {
    type Error = anyhow::Error;
    fn try_from(note: pb::PendingRewardNote) -> Result<PendingRewardNote> {
        Ok(PendingRewardNote {
            amount: note.amount.into(),
            destination: note.destination.unwrap().try_into()?,
        })
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RewardNotes {
    pub notes: Vec<PendingRewardNote>,
}

impl Protobuf<pb::RewardNotes> for RewardNotes {}

impl From<RewardNotes> for pb::RewardNotes {
    fn from(notes: RewardNotes) -> pb::RewardNotes {
        pb::RewardNotes {
            notes: notes.notes.into_iter().map(Into::into).collect(),
        }
    }
}

impl TryFrom<pb::RewardNotes> for RewardNotes {
    type Error = anyhow::Error;
    fn try_from(notes: pb::RewardNotes) -> Result<RewardNotes> {
        Ok(RewardNotes {
            notes: notes
                .notes
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct DelegationChanges {
    pub delegations: Vec<Delegate>,
    pub undelegations: Vec<Undelegate>,
}

impl Protobuf<pb::DelegationChanges> for DelegationChanges {}

impl From<DelegationChanges> for pb::DelegationChanges {
    fn from(changes: DelegationChanges) -> pb::DelegationChanges {
        pb::DelegationChanges {
            delegations: changes.delegations.into_iter().map(Into::into).collect(),
            undelegations: changes.undelegations.into_iter().map(Into::into).collect(),
        }
    }
}

impl TryFrom<pb::DelegationChanges> for DelegationChanges {
    type Error = anyhow::Error;
    fn try_from(changes: pb::DelegationChanges) -> Result<DelegationChanges> {
        Ok(DelegationChanges {
            delegations: changes
                .delegations
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_>>()?,
            undelegations: changes
                .undelegations
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_>>()?,
        })
    }
}

impl std::iter::Sum for DelegationChanges {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut sum = DelegationChanges::default();
        for changes in iter {
            sum.delegations.extend(changes.delegations);
            sum.undelegations.extend(changes.undelegations);
        }
        sum
    }
}
