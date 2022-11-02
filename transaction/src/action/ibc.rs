use ark_ff::Zero;
use ibc::core::ics24_host::identifier::{ChannelId, PortId};
use penumbra_crypto::{asset, value, Address, Amount, Balance, Fr};
use penumbra_proto::{
    core::ibc::v1alpha1::{self as pb, FungibleTokenPacketData},
    Message, Protobuf,
};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::{ActionView, TransactionPerspective};

use super::IsAction;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::Ics20Withdrawal", into = "pb::Ics20Withdrawal")]
pub struct ICS20Withdrawal {
    // the chain ID of the destination chain for this ICS20 transfer
    pub destination_chain_id: String,
    // a transparent value consisting of an amount and an asset ID.
    pub denom: asset::Denom,
    pub amount: Amount,
    // the address on the destination chain to send the transfer to
    pub destination_chain_address: String,
    // a "sender" penumbra address to use to return funds from this withdrawal.
    // this should be an ephemeral address
    pub return_address: Address,
    // the height (on Penumbra) at which this transfer expires (and funds are sent
    // back to the sender address?). NOTE: if funds are sent back to the sender,
    // we MUST verify a nonexistence proof before accepting the timeout, to
    // prevent relayer censorship attacks. The core IBC implementation does this
    // in its handling of validation of timeouts.
    pub timeout_height: u64,
    // the timestamp at which this transfer expires.
    pub timeout_time: u64,
    // the source port that identifies the channel used for the withdrawal
    pub source_port: PortId,
    // the source channel used for the withdrawal
    pub source_channel: ChannelId,
}

impl IsAction for ICS20Withdrawal {
    fn balance_commitment(&self) -> penumbra_crypto::balance::Commitment {
        self.balance().commit(Fr::zero())
    }

    fn view_from_perspective(&self, _txp: &TransactionPerspective) -> ActionView {
        ActionView::ICS20Withdrawal(self.to_owned())
    }
}

impl ICS20Withdrawal {
    pub fn value(&self) -> value::Value {
        value::Value {
            amount: self.amount,
            asset_id: self.denom.id(),
        }
    }

    pub fn balance(&self) -> Balance {
        -Balance::from(self.value())
    }

    pub fn packet_data(&self) -> Vec<u8> {
        let ftpd: FungibleTokenPacketData = self.clone().into();

        ftpd.encode_to_vec()
    }

    // stateless validation of an ICS20 withdrawal action.
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.timeout_height == 0 {
            anyhow::bail!("timeout height must be non-zero");
        }
        if self.timeout_time == 0 {
            anyhow::bail!("timeout time must be non-zero");
        }
        if self.source_port.as_str() != "transfer" {
            anyhow::bail!("source port for a withdrawal must be 'transfer'");
        }

        // NOTE: all strings are valid chain IDs, so we don't validate destination_chain_id here.

        // NOTE: we could validate the destination chain address as bech32 to prevent mistyped
        // addresses, but this would preclude sending to chains that don't use bech32 addresses.

        Ok(())
    }
}

impl Protobuf<pb::Ics20Withdrawal> for ICS20Withdrawal {}

impl From<ICS20Withdrawal> for pb::Ics20Withdrawal {
    fn from(w: ICS20Withdrawal) -> Self {
        pb::Ics20Withdrawal {
            destination_chain_id: w.destination_chain_id,
            denom: Some(w.denom.into()),
            amount: Some(w.amount.into()),
            destination_chain_address: w.destination_chain_address,
            return_address: Some(w.return_address.into()),
            timeout_height: w.timeout_height,
            timeout_time: w.timeout_time,
            source_channel: w.source_channel.to_string(),
            source_port: w.source_port.to_string(),
        }
    }
}

impl TryFrom<pb::Ics20Withdrawal> for ICS20Withdrawal {
    type Error = anyhow::Error;
    fn try_from(s: pb::Ics20Withdrawal) -> Result<Self, Self::Error> {
        Ok(Self {
            destination_chain_id: s.destination_chain_id,
            denom: s
                .denom
                .ok_or_else(|| anyhow::anyhow!("missing denom"))?
                .try_into()?,
            amount: s
                .amount
                .ok_or_else(|| anyhow::anyhow!("missing amount"))?
                .try_into()?,
            destination_chain_address: s.destination_chain_address,
            return_address: s
                .return_address
                .ok_or_else(|| anyhow::anyhow!("missing sender"))?
                .try_into()?,
            timeout_height: s.timeout_height,
            timeout_time: s.timeout_time,
            source_channel: ChannelId::from_str(&s.source_channel)?,
            source_port: PortId::from_str(&s.source_port)?,
        })
    }
}

impl From<ICS20Withdrawal> for pb::FungibleTokenPacketData {
    fn from(w: ICS20Withdrawal) -> Self {
        pb::FungibleTokenPacketData {
            amount: w.value().amount.to_string(),
            denom: w.value().asset_id.to_string(), // NOTE: should this be a `Denom` instead?
            receiver: w.destination_chain_address,
            sender: w.return_address.to_string(),
        }
    }
}
