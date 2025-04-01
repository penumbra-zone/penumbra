use ibc_types::core::{channel::ChannelId, channel::PortId, client::Height as IbcHeight};
use penumbra_sdk_asset::{
    asset::{self, Metadata},
    Balance, Value,
};
use penumbra_sdk_keys::Address;
use penumbra_sdk_num::Amount;
use penumbra_sdk_proto::{
    penumbra::core::component::ibc::v1::{self as pb, FungibleTokenPacketData},
    DomainType,
};
use penumbra_sdk_txhash::{EffectHash, EffectingData};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[cfg(feature = "component")]
use penumbra_sdk_ibc::component::packet::{IBCPacket, Unchecked};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::Ics20Withdrawal", into = "pb::Ics20Withdrawal")]
pub struct Ics20Withdrawal {
    // a transparent value consisting of an amount and a denom.
    pub amount: Amount,
    pub denom: asset::Metadata,
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
    // the timestamp at which this transfer expires, in nanoseconds after unix epoch.
    pub timeout_time: u64,
    // the source channel used for the withdrawal
    pub source_channel: ChannelId,

    // Whether to use a "compat" (bech32, non-m) address for the return address in the withdrawal,
    // for compatibility with chains that expect to be able to parse the return address as bech32.
    pub use_compat_address: bool,

    // Arbitrary string data to be included in the `memo` field
    // of the ICS-20 FungibleTokenPacketData for this withdrawal.
    // Commonly used for packet forwarding support, or other protocols that may support usage of the memo field.
    pub ics20_memo: String,
    // Whether to use a transparent address for the return address in the withdrawal.
    pub use_transparent_address: bool,
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

        // in order to prevent clients from inadvertently identifying themselves by their clock
        // skew, enforce that timeout time is rounded to the nearest minute
        if self.timeout_time % 60_000_000_000 != 0 {
            anyhow::bail!(
                "withdrawal timeout timestamp {} is not rounded to one minute",
                self.timeout_time
            );
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

#[allow(deprecated)]
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
            use_compat_address: w.use_compat_address,
            ics20_memo: w.ics20_memo.to_string(),
            use_transparent_address: w.use_transparent_address,
        }
    }
}

#[allow(deprecated)]
impl TryFrom<pb::Ics20Withdrawal> for Ics20Withdrawal {
    type Error = anyhow::Error;
    fn try_from(s: pb::Ics20Withdrawal) -> Result<Self, Self::Error> {
        Ok(Self {
            amount: s
                .amount
                .ok_or_else(|| anyhow::anyhow!("missing amount"))?
                .try_into()?,
            denom: Metadata::default_for(
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
            use_compat_address: s.use_compat_address,
            ics20_memo: s.ics20_memo,
            use_transparent_address: s.use_transparent_address,
        })
    }
}

impl From<Ics20Withdrawal> for pb::FungibleTokenPacketData {
    fn from(w: Ics20Withdrawal) -> Self {
        let ordinary_return_address = w.return_address.to_string();

        let return_address = if w.use_transparent_address {
            w.return_address
                .encode_as_transparent_address()
                .unwrap_or_else(|| ordinary_return_address)
        } else {
            ordinary_return_address
        };

        pb::FungibleTokenPacketData {
            amount: w.value().amount.to_string(),
            denom: w.denom.to_string(),
            receiver: w.destination_chain_address,
            sender: return_address,
            memo: w.ics20_memo,
        }
    }
}
