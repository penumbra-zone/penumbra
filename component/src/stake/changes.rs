use anyhow::Result;
use penumbra_proto::{core::stake::v1alpha1 as pb, DomainType};
use penumbra_transaction::action::{Delegate, Undelegate};
use serde::{Deserialize, Serialize};

/// Data structure used to track queued delegation changes that have been
/// committed to the chain but not yet processed at the epoch boundary.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(try_from = "pb::DelegationChanges", into = "pb::DelegationChanges")]
pub struct DelegationChanges {
    pub delegations: Vec<Delegate>,
    pub undelegations: Vec<Undelegate>,
}

impl DomainType for DelegationChanges {
    type Proto = pb::DelegationChanges;
}

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
