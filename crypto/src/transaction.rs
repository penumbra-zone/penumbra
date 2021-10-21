use std::convert::{TryFrom, TryInto};

use ark_ff::Zero;
use bytes::Bytes;

use decaf377::FieldExt;

use penumbra_proto::{transaction, Protobuf};

use crate::{
    action::{error::ProtoError, Action},
    merkle,
    rdsa::{Binding, Signature},
    Fr,
};

mod builder;
pub use builder::Builder;

pub struct TransactionBody {
    pub actions: Vec<Action>,
    pub merkle_root: merkle::Root,
    pub expiry_height: u32,
    pub chain_id: String,
    pub fee: Fee,
}

impl TransactionBody {
    pub fn sign() -> Transaction {
        todo!()
    }
}

impl Protobuf<transaction::TransactionBody> for TransactionBody {}

impl From<TransactionBody> for transaction::TransactionBody {
    fn from(msg: TransactionBody) -> Self {
        transaction::TransactionBody {
            actions: msg.actions.into_iter().map(|x| x.into()).collect(),
            anchor: Bytes::copy_from_slice(&msg.merkle_root.0.to_bytes()),
            expiry_height: msg.expiry_height,
            chain_id: msg.chain_id,
            fee: Some(msg.fee.into()),
        }
    }
}

impl TryFrom<transaction::TransactionBody> for TransactionBody {
    type Error = ProtoError;

    fn try_from(proto: transaction::TransactionBody) -> anyhow::Result<Self, Self::Error> {
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

        let expiry_height = proto
            .expiry_height
            .try_into()
            .map_err(|_| ProtoError::TransactionBodyMalformed)?;

        let chain_id = proto
            .chain_id
            .try_into()
            .map_err(|_| ProtoError::TransactionBodyMalformed)?;

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

pub struct Fee(pub u64);

// temp: remove dead code when Transaction fields are read
#[allow(dead_code)]
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
            merkle_root,
            expiry_height: None,
            chain_id: None,
        }
    }
}

impl Protobuf<transaction::Fee> for Fee {}

impl From<Fee> for transaction::Fee {
    fn from(fee: Fee) -> Self {
        transaction::Fee { amount: fee.0 }
    }
}

impl From<transaction::Fee> for Fee {
    fn from(proto: transaction::Fee) -> Self {
        Fee(proto.amount)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use rand_core::OsRng;

    use crate::keys::SpendKey;
    use crate::memo::MemoPlaintext;
    use crate::{Fq, Value};

    // Not really a test - just to exercise the code path for now
    #[test]
    fn test_transaction_create() {
        let mut rng = OsRng;
        let sk_sender = SpendKey::generate(&mut rng);
        let fvk_sender = sk_sender.full_viewing_key();
        let ivk_sender = fvk_sender.outgoing();

        let sk_recipient = SpendKey::generate(&mut rng);
        let fvk_recipient = sk_recipient.full_viewing_key();
        let ivk_recipient = fvk_recipient.incoming();
        let (dest, _dtk_d) = ivk_recipient.payment_address(0u64.into());

        let merkle_root = merkle::Root(Fq::zero());
        let _transaction_builder = Transaction::build_with_root(merkle_root)
            .set_fee(20)
            .add_output(
                &mut rng,
                &dest,
                Value {
                    amount: 10,
                    asset_id: b"pen".as_ref().into(),
                },
                MemoPlaintext::default(),
                ivk_sender,
            );
        // Commented out since .finalize() will currently fail the test.
        //let transaction = transaction_builder.finalize(&mut rng);
    }
}
