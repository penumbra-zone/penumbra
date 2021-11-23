use std::convert::{TryFrom, TryInto};

use ark_ff::Zero;
use bytes::Bytes;

use decaf377::FieldExt;

use penumbra_proto::{
    transaction::Fee as ProtoFee, transaction::Transaction as ProtoTransaction,
    transaction::TransactionBody as ProtoTransactionBody, Message, Protobuf,
};

use crate::{
    action::{error::ProtoError, Action},
    asset, merkle,
    rdsa::{Binding, Signature, VerificationKey, VerificationKeyBytes},
    Fr, Value,
};

mod error;
pub use error::Error;

mod builder;
pub use builder::Builder;

mod genesis;
pub use genesis::GenesisBuilder;

#[derive(Clone)]
pub struct TransactionBody {
    pub actions: Vec<Action>,
    pub merkle_root: merkle::Root,
    pub expiry_height: u32,
    pub chain_id: String,
    pub fee: Fee,
}

impl From<TransactionBody> for Vec<u8> {
    fn from(transaction_body: TransactionBody) -> Vec<u8> {
        let protobuf_serialized: ProtoTransactionBody = transaction_body.into();
        protobuf_serialized.encode_to_vec()
    }
}

impl Protobuf<ProtoTransactionBody> for TransactionBody {}

impl From<TransactionBody> for ProtoTransactionBody {
    fn from(msg: TransactionBody) -> Self {
        ProtoTransactionBody {
            actions: msg.actions.into_iter().map(|x| x.into()).collect(),
            anchor: Bytes::copy_from_slice(&msg.merkle_root.0.to_bytes()),
            expiry_height: msg.expiry_height,
            chain_id: msg.chain_id,
            fee: Some(msg.fee.into()),
        }
    }
}

impl TryFrom<ProtoTransactionBody> for TransactionBody {
    type Error = ProtoError;

    fn try_from(proto: ProtoTransactionBody) -> anyhow::Result<Self, Self::Error> {
        let mut actions = Vec::<Action>::new();
        for action in proto.actions {
            actions.push(
                action
                    .try_into()
                    .map_err(|_| ProtoError::TransactionBodyMalformed)?,
            );
        }

        let merkle_root = proto.anchor[..]
            .try_into()
            .map_err(|_| ProtoError::TransactionBodyMalformed)?;

        let expiry_height = proto.expiry_height;

        let chain_id = proto.chain_id;

        let fee: Fee = proto
            .fee
            .ok_or(ProtoError::TransactionBodyMalformed)?
            .into();

        Ok(TransactionBody {
            actions,
            merkle_root,
            expiry_height,
            chain_id,
            fee,
        })
    }
}

#[derive(Clone, Debug)]
pub struct Fee(pub u64);

#[derive(Clone)]
pub struct Transaction {
    transaction_body: TransactionBody,
    binding_sig: Signature<Binding>,
}

impl Transaction {
    /// Start building a transaction relative to a given [`merkle::Root`].
    pub fn build_with_root(merkle_root: merkle::Root) -> Builder {
        Builder {
            actions: Vec::new(),
            fee: None,
            synthetic_blinding_factor: Fr::zero(),
            value_balance: decaf377::Element::default(),
            value_commitments: decaf377::Element::default(),
            merkle_root,
            expiry_height: None,
            chain_id: None,
        }
    }

    /// Build the genesis transactions.
    pub fn genesis_build_with_root(merkle_root: merkle::Root) -> GenesisBuilder {
        GenesisBuilder {
            actions: Vec::new(),
            fee: None,
            synthetic_blinding_factor: Fr::zero(),
            value_balance: decaf377::Element::default(),
            value_commitments: decaf377::Element::default(),
            merkle_root,
            expiry_height: None,
            chain_id: None,
        }
    }

    pub fn transaction_body(&self) -> TransactionBody {
        self.transaction_body.clone()
    }

    pub fn binding_sig(&self) -> Signature<Binding> {
        self.binding_sig
    }

    pub fn id(&self) -> [u8; 32] {
        use sha2::{Digest, Sha256};

        let tx_bytes: Vec<u8> = self.clone().try_into().expect("can serialize transaction");
        let mut id_bytes = [0; 32];
        id_bytes[..].copy_from_slice(Sha256::digest(&tx_bytes).as_slice());

        id_bytes
    }

    /// Verify the binding signature.
    pub fn verify_binding_sig(&self) -> bool {
        let mut value_commitments = decaf377::Element::default();
        for action in self.transaction_body.actions.clone() {
            match action {
                Action::Output(inner) => {
                    value_commitments -= inner.body.value_commitment.0;
                }
                Action::Spend(inner) => {
                    value_commitments += inner.body.value_commitment.0;
                }
            }
        }

        // Add fee into binding verification key computation.
        let pen_trace = b"pen";
        let pen_id = asset::Id::from(&pen_trace[..]);
        let fee_value = Value {
            amount: self.transaction_body.fee.0,
            asset_id: pen_id,
        };
        let fee_v_blinding = Fr::zero();
        let fee_value_commitment = fee_value.commit(fee_v_blinding);
        value_commitments -= fee_value_commitment.0;

        // This works for all non-genesis transactions. For transactions with
        // non-zero value balance, the binding verification key must be computed
        // as `(value_commitments - value_balance).compress().0.into()`.
        let binding_verification_key_bytes: VerificationKeyBytes<Binding> =
            value_commitments.compress().0.into();
        let binding_verification_key: VerificationKey<Binding> = binding_verification_key_bytes
            .try_into()
            .expect("verification key is valid");

        let transaction_body_serialized: Vec<u8> = self.transaction_body.clone().into();
        binding_verification_key
            .verify(&transaction_body_serialized, &self.binding_sig)
            .is_ok()
    }
}

impl Protobuf<ProtoTransaction> for Transaction {}

impl From<Transaction> for ProtoTransaction {
    fn from(msg: Transaction) -> Self {
        let sig_bytes: [u8; 64] = msg.binding_sig.into();
        ProtoTransaction {
            body: Some(msg.transaction_body.into()),
            binding_sig: Bytes::copy_from_slice(&sig_bytes),
        }
    }
}

impl From<&Transaction> for ProtoTransaction {
    fn from(msg: &Transaction) -> Self {
        msg.into()
    }
}

impl TryFrom<ProtoTransaction> for Transaction {
    type Error = ProtoError;

    fn try_from(proto: ProtoTransaction) -> anyhow::Result<Self, Self::Error> {
        let transaction_body = proto
            .body
            .ok_or(ProtoError::TransactionMalformed)?
            .try_into()
            .map_err(|_| ProtoError::TransactionBodyMalformed)?;

        let sig_bytes: [u8; 64] = proto.binding_sig[..]
            .try_into()
            .map_err(|_| ProtoError::TransactionMalformed)?;

        Ok(Transaction {
            transaction_body,
            binding_sig: sig_bytes.into(),
        })
    }
}

impl TryFrom<&[u8]> for Transaction {
    type Error = ProtoError;

    fn try_from(bytes: &[u8]) -> Result<Transaction, Self::Error> {
        let protobuf_serialized_proof =
            ProtoTransaction::decode(bytes).map_err(|_| ProtoError::TransactionMalformed)?;
        Ok(protobuf_serialized_proof
            .try_into()
            .map_err(|_| ProtoError::TransactionMalformed)?)
    }
}

impl TryFrom<Vec<u8>> for Transaction {
    type Error = ProtoError;

    fn try_from(bytes: Vec<u8>) -> Result<Transaction, Self::Error> {
        Ok(Self::try_from(&bytes[..])?)
    }
}

impl Into<Vec<u8>> for Transaction {
    fn into(self) -> Vec<u8> {
        let protobuf_serialized: ProtoTransaction = self.into();
        protobuf_serialized.encode_to_vec()
    }
}

impl Into<Vec<u8>> for &Transaction {
    fn into(self) -> Vec<u8> {
        let protobuf_serialized: ProtoTransaction = self.into();
        protobuf_serialized.encode_to_vec()
    }
}

impl Protobuf<ProtoFee> for Fee {}

impl From<Fee> for ProtoFee {
    fn from(fee: Fee) -> Self {
        ProtoFee { amount: fee.0 }
    }
}

impl From<ProtoFee> for Fee {
    fn from(proto: ProtoFee) -> Self {
        Fee(proto.amount)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::keys::SpendKey;
    use crate::memo::MemoPlaintext;
    use crate::merkle::{Tree, TreeExt};
    use crate::transaction::Error;
    use crate::{note, Fq, Note, Value};
    use incrementalmerkletree::Frontier;
    use rand_core::OsRng;

    #[test]
    fn test_transaction_single_output_fails_due_to_nonzero_value_balance() {
        let mut rng = OsRng;
        let sk_sender = SpendKey::generate(&mut rng);
        let fvk_sender = sk_sender.full_viewing_key();
        let ovk_sender = fvk_sender.outgoing();

        let sk_recipient = SpendKey::generate(&mut rng);
        let fvk_recipient = sk_recipient.full_viewing_key();
        let ivk_recipient = fvk_recipient.incoming();
        let (dest, _dtk_d) = ivk_recipient.payment_address(0u64.into());

        let merkle_root = merkle::Root(Fq::zero());
        let transaction = Transaction::build_with_root(merkle_root)
            .set_fee(20)
            .set_chain_id("Pen".to_string())
            .add_output(
                &mut rng,
                &dest,
                Value {
                    amount: 10,
                    asset_id: b"pen".as_ref().into(),
                },
                MemoPlaintext::default(),
                ovk_sender,
            )
            .finalize(&mut rng);

        assert!(transaction.is_err());
        assert_eq!(transaction.err(), Some(Error::NonZeroValueBalance));
    }

    #[test]
    fn test_transaction_succeeds_if_values_balance() {
        let mut rng = OsRng;
        let sk_sender = SpendKey::generate(&mut rng);
        let fvk_sender = sk_sender.full_viewing_key();
        let ovk_sender = fvk_sender.outgoing();

        let sk_recipient = SpendKey::generate(&mut rng);
        let fvk_recipient = sk_recipient.full_viewing_key();
        let ivk_recipient = fvk_recipient.incoming();
        let (dest, _dtk_d) = ivk_recipient.payment_address(0u64.into());

        let output_value = Value {
            amount: 10,
            asset_id: b"pen".as_ref().into(),
        };
        let spend_value = Value {
            amount: 20,
            asset_id: b"pen".as_ref().into(),
        };
        let note = Note::new(
            *dest.diversifier(),
            *dest.transmission_key(),
            spend_value,
            Fq::zero(),
        )
        .expect("transmission key is valid");
        let note_commitment = note.commit();

        let mut nct = merkle::BridgeTree::<note::Commitment, 32>::new(1);
        nct.append(&note_commitment);
        let anchor = nct.root2();
        nct.witness();
        let auth_path = nct.authentication_path(&note_commitment).unwrap();
        let merkle_path = (u64::from(auth_path.0) as usize, auth_path.1);

        let transaction = Transaction::build_with_root(anchor)
            .set_fee(10)
            .set_chain_id("Pen".to_string())
            .add_output(
                &mut rng,
                &dest,
                output_value,
                MemoPlaintext::default(),
                ovk_sender,
            )
            .add_spend(&mut rng, sk_sender, merkle_path, note, auth_path.0)
            .finalize(&mut rng)
            .expect("transaction created ok");

        let merkle_root = transaction.clone().transaction_body().merkle_root;
        for action in transaction.clone().transaction_body().actions {
            match action {
                Action::Output(inner) => {
                    assert!(inner.body.proof.verify(
                        inner.body.value_commitment,
                        inner.body.note_commitment,
                        inner.body.ephemeral_key
                    ));
                }
                Action::Spend(inner) => {
                    assert!(inner.verify_auth_sig());

                    assert!(inner.body.proof.verify(
                        merkle_root.clone(),
                        inner.body.value_commitment,
                        inner.body.nullifier.clone(),
                        inner.body.rk,
                    ));
                }
            }
        }
    }
}
