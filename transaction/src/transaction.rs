use std::convert::{TryFrom, TryInto};

use ark_ff::Zero;
use bytes::Bytes;
use decaf377::FieldExt;
use penumbra_crypto::{
    merkle::{self, NoteCommitmentTree, TreeExt},
    rdsa::{Binding, Signature, VerificationKey, VerificationKeyBytes},
    Fr, Value,
};
use penumbra_proto::{
    transaction::{
        Fee as ProtoFee, Transaction as ProtoTransaction, TransactionBody as ProtoTransactionBody,
    },
    Message, Protobuf,
};
use penumbra_stake::STAKING_TOKEN_ASSET_ID;

// TODO: remove & replace with anyhow
use crate::{action::error::ProtoError, Action, GenesisBuilder};

mod builder;
pub use builder::Builder;

#[derive(Clone, Debug)]
pub struct TransactionBody {
    pub actions: Vec<Action>,
    pub merkle_root: merkle::Root,
    pub expiry_height: u32,
    pub chain_id: String,
    pub fee: Fee,
}

impl TransactionBody {
    pub fn sighash(&self) -> [u8; 64] {
        use penumbra_proto::sighash::SigHashTransaction;

        let sighash_tx = SigHashTransaction::from(ProtoTransactionBody::from(self.clone()));
        let sighash_tx_bytes: Vec<u8> = sighash_tx.encode_to_vec();

        *blake2b_simd::Params::default()
            .personal(b"Penumbra_SigHash")
            .hash(&sighash_tx_bytes)
            .as_array()
    }
}

#[derive(Clone, Debug)]
pub struct Fee(pub u64);

#[derive(Clone, Debug)]
pub struct Transaction {
    pub transaction_body: TransactionBody,
    pub binding_sig: Signature<Binding>,
}

impl Transaction {
    /// Start building a transaction relative to a given [`merkle::Root`].
    pub fn build_with_root(merkle_root: merkle::Root) -> Builder {
        Builder {
            spends: Vec::new(),
            outputs: Vec::new(),
            delegations: Vec::new(),
            undelegations: Vec::new(),
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
    pub fn genesis_builder() -> GenesisBuilder {
        let merkle_root = NoteCommitmentTree::new(0).root2();
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

    /// Verify the binding signature.
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
        protobuf_serialized_proof
            .try_into()
            .map_err(|_| ProtoError::TransactionMalformed)
    }
}

impl TryFrom<Vec<u8>> for Transaction {
    type Error = ProtoError;

    fn try_from(bytes: Vec<u8>) -> Result<Transaction, Self::Error> {
        Self::try_from(&bytes[..])
    }
}

impl From<Transaction> for Vec<u8> {
    fn from(transaction: Transaction) -> Vec<u8> {
        let protobuf_serialized: ProtoTransaction = transaction.into();
        protobuf_serialized.encode_to_vec()
    }
}

impl From<&Transaction> for Vec<u8> {
    fn from(transaction: &Transaction) -> Vec<u8> {
        let protobuf_serialized: ProtoTransaction = transaction.into();
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
    use penumbra_crypto::{
        keys::{SeedPhrase, SpendKey, SpendSeed},
        memo::MemoPlaintext,
        Fq, Value,
    };
    use rand_core::OsRng;

    use super::*;
    use crate::Error;

    #[test]
    fn test_transaction_single_output_fails_due_to_nonzero_value_balance() {
        let mut rng = OsRng;
        let seed_phrase = SeedPhrase::generate(&mut rng);
        let spend_seed = SpendSeed::from_seed_phrase(seed_phrase, 0);
        let sk_sender = SpendKey::new(spend_seed);
        let fvk_sender = sk_sender.full_viewing_key();
        let ovk_sender = fvk_sender.outgoing();

        let seed_phrase = SeedPhrase::generate(&mut rng);
        let spend_seed = SpendSeed::from_seed_phrase(seed_phrase, 0);
        let sk_recipient = SpendKey::new(spend_seed);
        let fvk_recipient = sk_recipient.full_viewing_key();
        let ivk_recipient = fvk_recipient.incoming();
        let (dest, _dtk_d) = ivk_recipient.payment_address(0u64.into());

        let merkle_root = merkle::Root(Fq::zero());
        let transaction = Transaction::build_with_root(merkle_root)
            .set_fee(20)
            .set_chain_id("penumbra".to_string())
            .add_output(
                &mut rng,
                &dest,
                Value {
                    amount: 10,
                    asset_id: *STAKING_TOKEN_ASSET_ID,
                },
                MemoPlaintext::default(),
                ovk_sender,
            )
            .finalize(&mut rng);

        assert!(transaction.is_err());
        assert_eq!(transaction.err(), Some(Error::NonZeroValueBalance));
    }
}
