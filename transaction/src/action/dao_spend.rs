use std::convert::{TryFrom, TryInto};

use anyhow::{Context, Error};
use ark_ff::Zero;
use decaf377::Fr;
use penumbra_crypto::{balance, Balance, Value};
use penumbra_proto::{core::governance::v1alpha1 as pb, DomainType};

use crate::{ActionView, TransactionPerspective};

use super::IsAction;

#[derive(Clone, Debug)]
pub struct DaoSpend {
    pub value: Value,
}

impl IsAction for DaoSpend {
    fn balance_commitment(&self) -> balance::Commitment {
        self.balance().commit(Fr::zero())
    }

    fn view_from_perspective(&self, _txp: &TransactionPerspective) -> ActionView {
        ActionView::DaoSpend(self.clone())
    }
}

impl DaoSpend {
    pub fn balance(&self) -> Balance {
        // Spends from the DAO produce value
        Balance::from(self.value)
    }
}

impl DomainType for DaoSpend {
    type Proto = pb::DaoSpend;
}

impl From<DaoSpend> for pb::DaoSpend {
    fn from(msg: DaoSpend) -> Self {
        pb::DaoSpend {
            value: Some(msg.value.into()),
        }
    }
}

impl TryFrom<pb::DaoSpend> for DaoSpend {
    type Error = Error;

    fn try_from(proto: pb::DaoSpend) -> anyhow::Result<Self, Self::Error> {
        let value = proto
            .value
            .ok_or_else(|| anyhow::anyhow!("missing value"))?
            .try_into()
            .context("malformed value")?;

        Ok(DaoSpend { value })
    }
}
