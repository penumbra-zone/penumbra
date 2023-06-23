use ibc_types2::core::channel::{ChannelId, PortId};
use penumbra_crypto::{
    asset::{self, DenomMetadata},
    value, Address, Amount, Balance, EffectHash, EffectingData,
};
use penumbra_proto::{
    core::ibc::v1alpha1::{self as pb, FungibleTokenPacketData},
    DomainType, Message, TypeUrl,
};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::Ics20Withdrawal", into = "pb::Ics20Withdrawal")]
pub struct Ics20Withdrawal {
    // the chain ID of the destination chain for this Ics20 transfer
    pub destination_chain_id: String,
    // a transparent value consisting of an amount and an asset ID.
    pub amount: Amount,
    pub denom: asset::DenomMetadata,
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
impl Ics20Withdrawal {
    pub fn value(&self) -> value::Value {
        penumbra_crypto::Value {
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

    // stateless validation of an Ics20 withdrawal action.
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

impl EffectingData for Ics20Withdrawal {
    fn effect_hash(&self) -> EffectHash {
        let mut state = blake2b_simd::Params::default()
            .personal(b"PAH:ics20wthdrwl")
            .to_state();

        let destination_chain_id_hash =
            blake2b_simd::Params::default().hash(self.destination_chain_id.as_bytes());
        let destination_chain_address_hash =
            blake2b_simd::Params::default().hash(self.destination_chain_address.as_bytes());

        state.update(destination_chain_id_hash.as_bytes());
        state.update(&self.value().amount.to_le_bytes());
        state.update(&self.value().asset_id.to_bytes());
        state.update(destination_chain_address_hash.as_bytes());
        //This is safe because the return address has a constant length of 80 bytes.
        state.update(&self.return_address.to_vec());
        state.update(&self.timeout_height.to_le_bytes());
        state.update(&self.timeout_time.to_le_bytes());
        EffectHash(*state.finalize().as_array())
    }
}

impl TypeUrl for Ics20Withdrawal {
    const TYPE_URL: &'static str = "/penumbra.core.ibc.v1alpha1.Ics20Withdrawal";
}

impl DomainType for Ics20Withdrawal {
    type Proto = pb::Ics20Withdrawal;
}

impl From<Ics20Withdrawal> for pb::Ics20Withdrawal {
    fn from(w: Ics20Withdrawal) -> Self {
        pb::Ics20Withdrawal {
            destination_chain_id: w.destination_chain_id,
            amount: Some(w.amount.into()),
            denom: Some(w.denom.base_denom().into()),
            destination_chain_address: w.destination_chain_address,
            return_address: Some(w.return_address.into()),
            timeout_height: w.timeout_height,
            timeout_time: w.timeout_time,
            source_channel: w.source_channel.to_string(),
            source_port: w.source_port.to_string(),
        }
    }
}

impl TryFrom<pb::Ics20Withdrawal> for Ics20Withdrawal {
    type Error = anyhow::Error;
    fn try_from(s: pb::Ics20Withdrawal) -> Result<Self, Self::Error> {
        Ok(Self {
            destination_chain_id: s.destination_chain_id,
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
            timeout_height: s.timeout_height,
            timeout_time: s.timeout_time,
            source_channel: ChannelId::from_str(&s.source_channel)?,
            source_port: PortId::from_str(&s.source_port)?,
        })
    }
}

impl From<Ics20Withdrawal> for pb::FungibleTokenPacketData {
    fn from(w: Ics20Withdrawal) -> Self {
        pb::FungibleTokenPacketData {
            amount: w.value().amount.to_string(),
            denom: w.value().asset_id.to_string(), // NOTE: should this be a `Denom` instead?
            receiver: w.destination_chain_address,
            sender: w.return_address.to_string(),
        }
    }
}
