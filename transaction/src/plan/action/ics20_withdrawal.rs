use ibc::core::ics24_host::identifier::{ChannelId, PortId};
use penumbra_crypto::{asset, value, Address, Amount, Balance};
use penumbra_proto::{core::ibc::v1alpha1 as pb, DomainType};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::action::Ics20Withdrawal;

/// A planned [`Ics20Withdrawal`](Ics20Withdrawal).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(try_from = "pb::Ics20WithdrawalPlan", into = "pb::Ics20WithdrawalPlan")]
pub struct Ics20WithdrawalPlan {
    pub destination_chain_id: String,
    pub destination_chain_address: String,
    pub denom: asset::Denom,
    pub amount: Amount,
    pub source_channel: ChannelId,
    pub timeout_height: u64,
    pub timeout_timestamp: u64,

    pub return_address: Address,
}

impl Ics20WithdrawalPlan {
    // NOTE: these are duplicated from action::Ics20Withdrawal. should they be deduplicated?
    pub fn balance(&self) -> Balance {
        -Balance::from(self.value())
    }

    pub fn value(&self) -> value::Value {
        value::Value {
            amount: self.amount,
            asset_id: self.denom.id(),
        }
    }

    pub fn withdrawal_action(&self) -> Ics20Withdrawal {
        Ics20Withdrawal {
            destination_chain_id: self.destination_chain_id.clone(),
            destination_chain_address: self.destination_chain_address.clone(),
            amount: self.amount.clone(),
            denom: self.denom.clone(),
            return_address: self.return_address,

            timeout_height: self.timeout_height,
            timeout_time: self.timeout_timestamp,
            source_port: PortId::transfer(),
            source_channel: self.source_channel.clone(),
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
            denom: Some(msg.denom.into()),
            amount: Some(msg.amount.into()),
            timeout_height: msg.timeout_height,
            timeout_timestamp: msg.timeout_timestamp,
            source_channel: msg.source_channel.as_str().to_string(),
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
            denom: msg
                .denom
                .ok_or(anyhow::anyhow!("missing denom"))?
                .try_into()?,
            amount: msg
                .amount
                .ok_or(anyhow::anyhow!("missing amount"))?
                .try_into()?,
            timeout_height: msg.timeout_height,
            timeout_timestamp: msg.timeout_timestamp,
            source_channel: ChannelId::from_str(&msg.source_channel)?,
            return_address: msg
                .return_address
                .ok_or(anyhow::anyhow!("missing return_address"))?
                .try_into()?,
        })
    }
}

#[cfg(test)]
mod test {}
