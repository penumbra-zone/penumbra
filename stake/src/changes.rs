use anyhow::Result;
use penumbra_proto::{stake as pb, Protobuf};

use crate::{Delegate, Undelegate, ValidatorDefinition};

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
