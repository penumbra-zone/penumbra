use ibc_types::core::{channel::ChannelId, channel::PortId, client::Height as IbcHeight};
use penumbra_asset::{
    asset::{self, DenomMetadata},
    Balance, Value,
};
use penumbra_keys::Address;
use penumbra_num::Amount;
use penumbra_proto::{
    penumbra::core::component::ibc::v1alpha1::{self as pb, FungibleTokenPacketData},
    DomainType,
};
use penumbra_txhash::{EffectHash, EffectingData};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[cfg(feature = "component")]
use penumbra_ibc::component::packet::{IBCPacket, Unchecked};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::Ics20Withdrawal", into = "pb::Ics20Withdrawal")]
pub struct Ics20Withdrawal {
    // a transparent value consisting of an amount and a denom.
    pub amount: Amount,
    pub denom: asset::DenomMetadata,
    // the address on the destination chain to send the transfer to
    pub destination_chain_address: String,
    // a "sender" penumbra address to use to return funds from this withdrawal.
    // this should be an ephemeral address
    pub return_address: Address,
    // the height (on Penumbra) at which this transfer expires (and funds are sent
    // back to the return address?). NOTE: if funds are sent back to the sender,
    // we MUST verify a nonexistence proof before accepting the timeout, to
    // prevent relayer censorship attacks. The core IBC implementation does this
    // in its handling of validation of timeouts.
    pub timeout_height: IbcHeight,
    // the timestamp at which this transfer expires.
    pub timeout_time: u64,
    // the source channel used for the withdrawal
    pub source_channel: ChannelId,
}

#[cfg(feature = "component")]
impl From<Ics20Withdrawal> for IBCPacket<Unchecked> {
    fn from(withdrawal: Ics20Withdrawal) -> Self {
        Self::new(
            PortId::transfer(),
            withdrawal.source_channel.clone(),
            withdrawal.timeout_height,
            withdrawal.timeout_time,
            withdrawal.packet_data(),
        )
    }
}

impl Ics20Withdrawal {
    pub fn value(&self) -> Value {
        Value {
            amount: self.amount,
            asset_id: self.denom.id(),
        }
    }

    pub fn balance(&self) -> Balance {
        -Balance::from(self.value())
    }

    pub fn packet_data(&self) -> Vec<u8> {
        let ftpd: FungibleTokenPacketData = self.clone().into();

        // In violation of the ICS20 spec, ibc-go encodes transfer packets as JSON.
        serde_json::to_vec(&ftpd).expect("can serialize FungibleTokenPacketData as JSON")
    }

    // stateless validation of an Ics20 withdrawal action.
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.timeout_time == 0 {
            anyhow::bail!("timeout time must be non-zero");
        }

        // NOTE: we could validate the destination chain address as bech32 to prevent mistyped
        // addresses, but this would preclude sending to chains that don't use bech32 addresses.

        Ok(())
    }
}

impl EffectingData for Ics20Withdrawal {
    fn effect_hash(&self) -> EffectHash {
        EffectHash::from_proto_effecting_data(&self.to_proto())
    }
}

impl DomainType for Ics20Withdrawal {
    type Proto = pb::Ics20Withdrawal;
}

impl From<Ics20Withdrawal> for pb::Ics20Withdrawal {
    fn from(w: Ics20Withdrawal) -> Self {
        pb::Ics20Withdrawal {
            amount: Some(w.amount.into()),
            denom: Some(w.denom.base_denom().into()),
            destination_chain_address: w.destination_chain_address,
            return_address: Some(w.return_address.into()),
            timeout_height: Some(w.timeout_height.into()),
            timeout_time: w.timeout_time,
            source_channel: w.source_channel.to_string(),
        }
    }
}

impl TryFrom<pb::Ics20Withdrawal> for Ics20Withdrawal {
    type Error = anyhow::Error;
    fn try_from(s: pb::Ics20Withdrawal) -> Result<Self, Self::Error> {
        Ok(Self {
            amount: s
                .amount
                .ok_or_else(|| anyhow::anyhow!("missing amount"))?
                .try_into()?,
            denom: DenomMetadata::default_for(
                &s.denom
                    .ok_or_else(|| anyhow::anyhow!("missing denom metadata"))?
                    .try_into()?,
            )
            .ok_or_else(|| anyhow::anyhow!("could not generate default denom metadata"))?,
            destination_chain_address: s.destination_chain_address,
            return_address: s
                .return_address
                .ok_or_else(|| anyhow::anyhow!("missing sender"))?
                .try_into()?,
            timeout_height: s
                .timeout_height
                .ok_or_else(|| anyhow::anyhow!("missing timeout height"))?
                .try_into()?,
            timeout_time: s.timeout_time,
            source_channel: ChannelId::from_str(&s.source_channel)?,
        })
    }
}

impl From<Ics20Withdrawal> for pb::FungibleTokenPacketData {
    fn from(w: Ics20Withdrawal) -> Self {
        pb::FungibleTokenPacketData {
            amount: w.value().amount.to_string(),
            denom: w.denom.to_string(),
            receiver: w.destination_chain_address,
            sender: w.return_address.to_string(),
        }
    }
}
