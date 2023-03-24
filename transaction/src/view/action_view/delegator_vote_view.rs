use penumbra_crypto::NoteView;
use penumbra_proto::{core::transaction::v1alpha1 as pbt, DomainType};
use serde::{Deserialize, Serialize};

use crate::action::DelegatorVote;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(try_from = "pbt::DelegatorVoteView", into = "pbt::DelegatorVoteView")]
#[allow(clippy::large_enum_variant)]
pub enum DelegatorVoteView {
    Visible {
        delegator_vote: DelegatorVote,
        note: NoteView,
    },
    Opaque {
        delegator_vote: DelegatorVote,
    },
}

impl DomainType for DelegatorVoteView {
    type Proto = pbt::DelegatorVoteView;
}

impl TryFrom<pbt::DelegatorVoteView> for DelegatorVoteView {
    type Error = anyhow::Error;

    fn try_from(v: pbt::DelegatorVoteView) -> Result<Self, Self::Error> {
        match v
            .delegator_vote
            .ok_or_else(|| anyhow::anyhow!("missing delegator vote field"))?
        {
            pbt::delegator_vote_view::DelegatorVote::Visible(x) => Ok(DelegatorVoteView::Visible {
                delegator_vote: x
                    .delegator_vote
                    .ok_or_else(|| anyhow::anyhow!("missing delegator vote field"))?
                    .try_into()?,
                note: x
                    .note
                    .ok_or_else(|| anyhow::anyhow!("missing note field"))?
                    .try_into()?,
            }),
            pbt::delegator_vote_view::DelegatorVote::Opaque(x) => Ok(DelegatorVoteView::Opaque {
                delegator_vote: x
                    .delegator_vote
                    .ok_or_else(|| anyhow::anyhow!("missing spend field"))?
                    .try_into()?,
            }),
        }
    }
}

impl From<DelegatorVoteView> for pbt::DelegatorVoteView {
    fn from(v: DelegatorVoteView) -> Self {
        use pbt::delegator_vote_view as dvv;
        match v {
            DelegatorVoteView::Visible {
                delegator_vote,
                note,
            } => Self {
                delegator_vote: Some(dvv::DelegatorVote::Visible(dvv::Visible {
                    delegator_vote: Some(delegator_vote.into()),
                    note: Some(note.into()),
                })),
            },
            DelegatorVoteView::Opaque { delegator_vote } => Self {
                delegator_vote: Some(dvv::DelegatorVote::Opaque(dvv::Opaque {
                    delegator_vote: Some(delegator_vote.into()),
                })),
            },
        }
    }
}

impl From<DelegatorVoteView> for DelegatorVote {
    fn from(v: DelegatorVoteView) -> Self {
        match v {
            DelegatorVoteView::Visible {
                delegator_vote,
                note: _,
            } => delegator_vote,
            DelegatorVoteView::Opaque { delegator_vote } => delegator_vote,
        }
    }
}
