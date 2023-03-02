use std::convert::{TryFrom, TryInto};

use anyhow::{Context, Error};
use ark_ff::Zero;
use decaf377::Fr;
use penumbra_crypto::{balance, Address, Balance, Value};
use penumbra_proto::{core::governance::v1alpha1 as pb, DomainType};

use crate::{ActionView, TransactionPerspective};

use super::IsAction;

#[derive(Clone, Debug)]
pub struct DaoOutput {
    pub value: Value,
    pub address: Address,
}

impl IsAction for DaoOutput {
    fn balance_commitment(&self) -> balance::Commitment {
        // Outputs from the DAO require value
        self.balance().commit(Fr::zero())
    }

    fn view_from_perspective(&self, _txp: &TransactionPerspective) -> ActionView {
        ActionView::DaoOutput(self.clone())
    }
}

impl DaoOutput {
    pub fn balance(&self) -> Balance {
        // Outputs from the DAO require value
        -Balance::from(self.value)
    }
}

impl DomainType for DaoOutput {
    type Proto = pb::DaoOutput;
}

impl From<DaoOutput> for pb::DaoOutput {
    fn from(msg: DaoOutput) -> Self {
        pb::DaoOutput {
            value: Some(msg.value.into()),
            address: Some(msg.address.into()),
        }
    }
}

impl TryFrom<pb::DaoOutput> for DaoOutput {
    type Error = Error;

    fn try_from(proto: pb::DaoOutput) -> anyhow::Result<Self, Self::Error> {
        let value = proto
            .value
            .ok_or_else(|| anyhow::anyhow!("missing value"))?
            .try_into()
            .context("malformed value")?;
        let address = proto
            .address
            .ok_or_else(|| anyhow::anyhow!("missing address"))?
            .try_into()
            .context("malformed address")?;

        Ok(DaoOutput { value, address })
    }
}
