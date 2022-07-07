use std::convert::{TryFrom, TryInto};

use anyhow::Error;
use ark_ff::Zero;
use bytes::Bytes;
use penumbra_crypto::{
    rdsa::{Binding, Signature, VerificationKey, VerificationKeyBytes},
    transaction::Fee,
    Fr, NotePayload, Nullifier, Value, STAKING_TOKEN_ASSET_ID,
};
use penumbra_proto::{ibc as pb_ibc, stake as pbs, transaction as pbt, Message, Protobuf};
use penumbra_tct as tct;

use crate::{
    action::{Delegate, Undelegate},
    Action,
};

#[derive(Clone, Debug)]
pub struct TransactionBody {
    pub actions: Vec<Action>,
    pub expiry_height: u64,
    pub chain_id: String,
    pub fee: Fee,
}

#[derive(Clone, Debug)]
pub struct Transaction {
    pub transaction_body: TransactionBody,
    pub binding_sig: Signature<Binding>,
    pub anchor: tct::Root,
}

impl Transaction {
    pub fn actions(&self) -> impl Iterator<Item = &Action> {
        self.transaction_body.actions.iter()
    }

    pub fn delegations(&self) -> impl Iterator<Item = &Delegate> {
        self.actions().filter_map(|action| {
            if let Action::Delegate(d) = action {
                Some(d)
            } else {
                None
            }
        })
    }

    pub fn undelegations(&self) -> impl Iterator<Item = &Undelegate> {
        self.actions().filter_map(|action| {
            if let Action::Undelegate(d) = action {
                Some(d)
            } else {
                None
            }
        })
    }

    pub fn ibc_actions(&self) -> impl Iterator<Item = &pb_ibc::IbcAction> {
        self.actions().filter_map(|action| {
            if let Action::IBCAction(ibc_action) = action {
                Some(ibc_action)
            } else {
                None
            }
        })
    }

    pub fn validator_definitions(&self) -> impl Iterator<Item = &pbs::ValidatorDefinition> {
        self.actions().filter_map(|action| {
            if let Action::ValidatorDefinition(d) = action {
                Some(d)
            } else {
                None
            }
        })
    }

    // TODO: make sure payloads from Swap actions included
    pub fn note_payloads(&self) -> Vec<NotePayload> {
        self.transaction_body
            .actions
            .iter()
            .filter_map(|action| {
                if let Action::Output(output) = action {
                    Some(output.body.note_payload.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    // TODO: make sure nullifiers from SwapClaim actions included
    pub fn spent_nullifiers(&self) -> Vec<Nullifier> {
        self.transaction_body
            .actions
            .iter()
            .filter_map(|action| {
                // Note: adding future actions that include nullifiers
                // will need to be matched here as well as Spends
                if let Action::Spend(spend) = action {
                    Some(spend.body.nullifier.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn transaction_body(&self) -> TransactionBody {
        self.transaction_body.clone()
    }

    pub fn binding_sig(&self) -> &Signature<Binding> {
        &self.binding_sig
    }

    pub fn id(&self) -> [u8; 32] {
        use sha2::{Digest, Sha256};

        let tx_bytes: Vec<u8> = self.clone().try_into().expect("can serialize transaction");
        let mut id_bytes = [0; 32];
        id_bytes[..].copy_from_slice(Sha256::digest(&tx_bytes).as_slice());

        id_bytes
    }

    /// Compute the binding verification key from the transaction data.
    pub fn binding_verification_key(&self) -> VerificationKey<Binding> {
        let mut value_commitments = decaf377::Element::default();
        for action in &self.transaction_body.actions {
            value_commitments += action.value_commitment().0;
        }

        // Add fee into binding verification key computation.
        let fee_value = Value {
            amount: self.transaction_body.fee.0,
            asset_id: *STAKING_TOKEN_ASSET_ID,
        };
        let fee_v_blinding = Fr::zero();
        let fee_value_commitment = fee_value.commit(fee_v_blinding);
        value_commitments -= fee_value_commitment.0;

        let binding_verification_key_bytes: VerificationKeyBytes<Binding> =
            value_commitments.compress().0.into();

        binding_verification_key_bytes
            .try_into()
            .expect("verification key is valid")
    }
}

impl From<TransactionBody> for Vec<u8> {
    fn from(transaction_body: TransactionBody) -> Vec<u8> {
        let protobuf_serialized: pbt::TransactionBody = transaction_body.into();
        protobuf_serialized.encode_to_vec()
    }
}

impl Protobuf<pbt::TransactionBody> for TransactionBody {}

impl From<TransactionBody> for pbt::TransactionBody {
    fn from(msg: TransactionBody) -> Self {
        pbt::TransactionBody {
            actions: msg.actions.into_iter().map(|x| x.into()).collect(),
            expiry_height: msg.expiry_height,
            chain_id: msg.chain_id,
            fee: Some(msg.fee.into()),
        }
    }
}

impl TryFrom<pbt::TransactionBody> for TransactionBody {
    type Error = Error;

    fn try_from(proto: pbt::TransactionBody) -> anyhow::Result<Self, Self::Error> {
        let mut actions = Vec::<Action>::new();
        for action in proto.actions {
            actions.push(
                action
                    .try_into()
                    .map_err(|_| anyhow::anyhow!("transaction body malformed"))?,
            );
        }

        let expiry_height = proto.expiry_height;

        let chain_id = proto.chain_id;

        let fee: Fee = proto
            .fee
            .ok_or_else(|| anyhow::anyhow!("transaction body malformed"))?
            .into();

        Ok(TransactionBody {
            actions,
            expiry_height,
            chain_id,
            fee,
        })
    }
}
impl Protobuf<pbt::Transaction> for Transaction {}

impl From<Transaction> for pbt::Transaction {
    fn from(msg: Transaction) -> Self {
        let sig_bytes: [u8; 64] = msg.binding_sig.into();
        pbt::Transaction {
            body: Some(msg.transaction_body.into()),
            anchor: Some(msg.anchor.into()),
            binding_sig: Bytes::copy_from_slice(&sig_bytes),
        }
    }
}

impl From<&Transaction> for pbt::Transaction {
    fn from(msg: &Transaction) -> Self {
        msg.into()
    }
}

impl TryFrom<pbt::Transaction> for Transaction {
    type Error = Error;

    fn try_from(proto: pbt::Transaction) -> anyhow::Result<Self, Self::Error> {
        let transaction_body = proto
            .body
            .ok_or_else(|| anyhow::anyhow!("transaction malformed"))?
            .try_into()
            .map_err(|_| anyhow::anyhow!("transaction body malformed"))?;

        let sig_bytes: [u8; 64] = proto.binding_sig[..]
            .try_into()
            .map_err(|_| anyhow::anyhow!("transaction malformed"))?;

        let anchor = proto
            .anchor
            .ok_or_else(|| anyhow::anyhow!("transaction malformed"))?
            .try_into()
            .map_err(|_| anyhow::anyhow!("transaction malformed"))?;

        Ok(Transaction {
            transaction_body,
            binding_sig: sig_bytes.into(),
            anchor,
        })
    }
}

impl TryFrom<&[u8]> for Transaction {
    type Error = Error;

    fn try_from(bytes: &[u8]) -> Result<Transaction, Self::Error> {
        let protobuf_serialized_proof = pbt::Transaction::decode(bytes)
            .map_err(|_| anyhow::anyhow!("transaction malformed"))?;
        protobuf_serialized_proof
            .try_into()
            .map_err(|_| anyhow::anyhow!("transaction malformed"))
    }
}

impl TryFrom<Vec<u8>> for Transaction {
    type Error = Error;

    fn try_from(bytes: Vec<u8>) -> Result<Transaction, Self::Error> {
        Self::try_from(&bytes[..])
    }
}

impl From<Transaction> for Vec<u8> {
    fn from(transaction: Transaction) -> Vec<u8> {
        let protobuf_serialized: pbt::Transaction = transaction.into();
        protobuf_serialized.encode_to_vec()
    }
}

impl From<&Transaction> for Vec<u8> {
    fn from(transaction: &Transaction) -> Vec<u8> {
        let protobuf_serialized: pbt::Transaction = transaction.into();
        protobuf_serialized.encode_to_vec()
    }
}
