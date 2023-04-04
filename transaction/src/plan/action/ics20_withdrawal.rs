use penumbra_crypto::{asset, value, Address, Amount, Balance};
use penumbra_proto::{core::ibc::v1alpha1 as pb, DomainType};
use rand_core::{CryptoRng, RngCore};
use serde::{Deserialize, Serialize};

/// A planned [`Ics20Withdrawal`](Ics20Withdrawal).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(try_from = "pb::Ics20WithdrawalPlan", into = "pb::Ics20WithdrawalPlan")]
pub struct Ics20WithdrawalPlan {
    pub destination_chain_id: String,
    pub destination_chain_address: String,
    pub asset_id: asset::Id,
    pub amount: Amount,

    pub return_address: Address,
}

impl Ics20WithdrawalPlan {
    pub fn new<R: RngCore + CryptoRng>(
        destination_chain_id: String,
        destination_chain_address: String,
        return_address: Address,
        asset_id: asset::Id,
        amount: Amount,
    ) -> Ics20WithdrawalPlan {
        Ics20WithdrawalPlan {
            destination_chain_id,
            destination_chain_address,
            asset_id,
            amount,
            return_address,
        }
    }

    // NOTE: these are duplicated from action::Ics20Withdrawal. should they be deduplicated?
    pub fn balance(&self) -> Balance {
        -Balance::from(self.value())
    }

    pub fn value(&self) -> value::Value {
        value::Value {
            amount: self.amount,
            asset_id: self.asset_id,
        }
    }
}

impl DomainType for Ics20WithdrawalPlan {
    type Proto = pb::Ics20WithdrawalPlan;
}

impl From<Ics20WithdrawalPlan> for pb::Ics20WithdrawalPlan {
    fn from(msg: Ics20WithdrawalPlan) -> Self {
        Self {
            destination_chain_id: msg.destination_chain_id,
            destination_chain_address: msg.destination_chain_address,
            asset_id: Some(msg.asset_id.into()),
            amount: Some(msg.amount.into()),
            return_address: Some(msg.return_address.into()),
        }
    }
}

impl TryFrom<pb::Ics20WithdrawalPlan> for Ics20WithdrawalPlan {
    type Error = anyhow::Error;
    fn try_from(msg: pb::Ics20WithdrawalPlan) -> Result<Self, Self::Error> {
        Ok(Self {
            destination_chain_id: msg.destination_chain_id,
            destination_chain_address: msg.destination_chain_address,
            asset_id: msg
                .asset_id
                .ok_or(anyhow::anyhow!("missing denom"))?
                .try_into()?,
            amount: msg
                .amount
                .ok_or(anyhow::anyhow!("missing amount"))?
                .try_into()?,
            return_address: msg
                .return_address
                .ok_or(anyhow::anyhow!("missing return_address"))?
                .try_into()?,
        })
    }
}

#[cfg(test)]
mod test {}
