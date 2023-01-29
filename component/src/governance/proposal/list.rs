use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

use penumbra_proto::{core::governance::v1alpha1 as pb, Protobuf};

/// A protobuf-represented duplicate-free set of proposal ids.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProposalList {
    pub proposals: BTreeSet<u64>,
}

impl Protobuf for ProposalList {
    type Proto = pb::ProposalList;
}

impl From<ProposalList> for pb::ProposalList {
    fn from(list: ProposalList) -> Self {
        pb::ProposalList {
            proposals: list.proposals.into_iter().collect(),
        }
    }
}

impl TryFrom<pb::ProposalList> for ProposalList {
    type Error = anyhow::Error;

    fn try_from(msg: pb::ProposalList) -> Result<Self, Self::Error> {
        Ok(ProposalList {
            proposals: msg.proposals.into_iter().collect(),
        })
    }
}
