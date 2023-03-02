use std::convert::{TryFrom, TryInto};

use anyhow::{Context, Error};
use ark_ff::Zero;
use decaf377::Fr;
use penumbra_crypto::{balance, Balance, Value};
use penumbra_proto::{core::governance::v1alpha1 as pb, DomainType};

use crate::{ActionView, TransactionPerspective};

use super::IsAction;

#[derive(Clone, Debug)]
pub struct DaoDeposit {
    pub value: Value,
}

impl IsAction for DaoDeposit {
    fn balance_commitment(&self) -> balance::Commitment {
        self.balance().commit(Fr::zero())
    }

    fn view_from_perspective(&self, _txp: &TransactionPerspective) -> ActionView {
        ActionView::DaoDeposit(self.clone())
    }
}

impl DaoDeposit {
    pub fn balance(&self) -> Balance {
        // Deposits into the DAO require value
        -Balance::from(self.value)
    }
}

impl DomainType for DaoDeposit {
    type Proto = pb::DaoDeposit;
}

impl From<DaoDeposit> for pb::DaoDeposit {
    fn from(msg: DaoDeposit) -> Self {
        pb::DaoDeposit {
            value: Some(msg.value.into()),
        }
    }
}

impl TryFrom<pb::DaoDeposit> for DaoDeposit {
    type Error = Error;

    fn try_from(proto: pb::DaoDeposit) -> anyhow::Result<Self, Self::Error> {
        let value = proto
            .value
            .ok_or_else(|| anyhow::anyhow!("missing value"))?
            .try_into()
            .context("malformed value")?;

        Ok(DaoDeposit { value })
    }
}
