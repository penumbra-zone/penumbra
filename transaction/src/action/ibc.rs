use penumbra_crypto::{balance, value, Address, Fr, Zero};
use penumbra_proto::{ibc as pb, Protobuf};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::Ics20Withdrawal", into = "pb::Ics20Withdrawal")]
pub struct ICS20Withdrawal {
    // the chain ID of the destination chain for this ICS20 transfer
    pub destination_chain_id: String,
    // a transparent value consisting of an amount and an asset ID.
    pub value: value::Value,
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
}

impl ICS20Withdrawal {
    pub fn value_commitment(&self) -> balance::Commitment {
        // we need to compute a value commitment to the transparent value of this withdrawal
        let withdrawal_value = self.value.commit(Fr::zero());

        // Consume from the withdrawal value. This should be negative because it consumes from the value
        // balance of the transaction. when we compute the binding verification key, we sum all of
        // the value commitments. Thus, there should be a spend that ouputs a positive value
        // balance to this tx prior to this withdrawal action.
        -withdrawal_value
    }
}

impl Protobuf<pb::Ics20Withdrawal> for ICS20Withdrawal {}

impl From<ICS20Withdrawal> for pb::Ics20Withdrawal {
    fn from(w: ICS20Withdrawal) -> Self {
        pb::Ics20Withdrawal {
            destination_chain_id: w.destination_chain_id,
            value: Some(w.value.into()),
            destination_chain_address: w.destination_chain_address,
            return_address: Some(w.return_address.into()),
            timeout_height: w.timeout_height,
            timeout_time: w.timeout_time,
        }
    }
}

impl TryFrom<pb::Ics20Withdrawal> for ICS20Withdrawal {
    type Error = anyhow::Error;
    fn try_from(s: pb::Ics20Withdrawal) -> Result<Self, Self::Error> {
        Ok(Self {
            destination_chain_id: s.destination_chain_id,
            value: s
                .value
                .ok_or_else(|| anyhow::anyhow!("missing value"))?
                .try_into()?,
            destination_chain_address: s.destination_chain_address,
            return_address: s
                .return_address
                .ok_or_else(|| anyhow::anyhow!("missing sender"))?
                .try_into()?,
            timeout_height: s.timeout_height,
            timeout_time: s.timeout_time,
        })
    }
}
