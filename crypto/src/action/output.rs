use ark_ff::UniformRand;
use bytes::Bytes;
use rand_core::{CryptoRng, RngCore};
use std::convert::{TryFrom, TryInto};
use thiserror;

use penumbra_proto::{transaction, Protobuf};

use crate::{
    addresses::PaymentAddress,
    ka,
    memo::MemoCiphertext,
    note,
    proofs::transparent::{OutputProof, OUTPUT_PROOF_LEN_BYTES},
    value, Fq, Fr, Note, Value,
};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Error converting from protobuf: field is missing")]
    ProtobufMissingFieldError,
    #[error("OutputBody could not be converted from protobuf")]
    ProtobufOutputBodyMalformed,
    #[error("Memo ciphertext malformed")]
    MemoCiphertextMalformed,
    #[error("Outgoing viewing key malformed")]
    OutgoingViewingKeyMalformed,
    #[error("Value commitment malformed")]
    ValueCommitmentMalformed,
    #[error("Encrypted note malformed")]
    EncryptedNoteMalformed,
    #[error("Ephemeral public key malformed")]
    EphemeralPubKeyMalformed,
    #[error("Note commitment malformed")]
    NoteCommitmentMalformed,
}

pub const OVK_WRAPPED_LEN_BYTES: usize = 80;
pub const NOTE_ENCRYPTION_BYTES: usize = 80;

pub struct Output {
    pub body: Body,
    pub encrypted_memo: MemoCiphertext,
    pub ovk_wrapped_key: [u8; OVK_WRAPPED_LEN_BYTES],
}

impl Protobuf<transaction::Output> for Output {}

impl From<Output> for transaction::Output {
    fn from(msg: Output) -> Self {
        transaction::Output {
            body: Some(msg.body.into()),
            encrypted_memo: Bytes::copy_from_slice(&msg.encrypted_memo.0),
            ovk_wrapped_key: Bytes::copy_from_slice(&msg.ovk_wrapped_key),
        }
    }
}

impl TryFrom<transaction::Output> for Output {
    type Error = Error;

    fn try_from(proto: transaction::Output) -> Result<Self, Self::Error> {
        if proto.body.is_none() {
            return Err(Error::ProtobufMissingFieldError);
        }

        let body = Body::try_from(proto.body.unwrap());

        if body.is_err() {
            return Err(Error::ProtobufOutputBodyMalformed);
        }

        let encrypted_memo: MemoCiphertext = match proto.encrypted_memo[..].try_into() {
            Err(_) => return Err(Error::MemoCiphertextMalformed),
            Ok(inner) => MemoCiphertext(inner),
        };

        let ovk_wrapped_key: [u8; OVK_WRAPPED_LEN_BYTES] =
            match proto.ovk_wrapped_key[..].try_into() {
                Err(_) => return Err(Error::OutgoingViewingKeyMalformed),
                Ok(inner) => inner,
            };

        Ok(Output {
            body: body.unwrap(),
            encrypted_memo,
            ovk_wrapped_key,
        })
    }
}

pub struct Body {
    pub value_commitment: value::Commitment,
    pub note_commitment: note::Commitment,
    pub ephemeral_key: ka::Public,
    pub encrypted_note: [u8; NOTE_ENCRYPTION_BYTES],
    pub proof: OutputProof,
}

impl Body {
    pub fn new<R: RngCore + CryptoRng>(
        rng: &mut R,
        value: Value,
        v_blinding: Fr,
        dest: &PaymentAddress,
    ) -> Body {
        // TODO: p. 43 Spec. Decide whether to do leadByte 0x01 method or 0x02 or other.
        let value_commitment = value.commit(v_blinding);

        let note_blinding = Fq::rand(rng);

        let note = Note::new(dest, value, note_blinding);
        let note_commitment = note.commit();
        // TODO: Encrypt note here and add to a field in the Body struct (later).
        // TEMP
        let encrypted_note = [0u8; NOTE_ENCRYPTION_BYTES];

        let esk = ka::Secret::new(rng);
        let ephemeral_key = esk.diversified_public(note.dest.diversified_generator());

        let proof = OutputProof {};

        Self {
            value_commitment,
            note_commitment,
            ephemeral_key,
            encrypted_note,
            proof,
        }
    }
}

impl Protobuf<transaction::OutputBody> for Body {}

impl From<Body> for transaction::OutputBody {
    fn from(msg: Body) -> Self {
        let cv_bytes: [u8; 32] = msg.value_commitment.into();
        let cm_bytes: [u8; 32] = msg.note_commitment.into();
        let proof_bytes: [u8; OUTPUT_PROOF_LEN_BYTES] = msg.proof.into();
        transaction::OutputBody {
            cv: Bytes::copy_from_slice(&cv_bytes),
            cm: Bytes::copy_from_slice(&cm_bytes),
            ephemeral_key: Bytes::copy_from_slice(&msg.ephemeral_key.0),
            encrypted_note: Bytes::copy_from_slice(&msg.encrypted_note),
            zkproof: Bytes::copy_from_slice(&proof_bytes),
        }
    }
}

impl TryFrom<transaction::OutputBody> for Body {
    type Error = Error;

    fn try_from(proto: transaction::OutputBody) -> Result<Self, Self::Error> {
        let proto_cv_bytes: [u8; 32] = match proto.cv[..].try_into() {
            Err(_) => return Err(Error::ValueCommitmentMalformed),
            Ok(inner) => inner,
        };
        let value_commitment: value::Commitment = match proto_cv_bytes.try_into() {
            Err(_) => return Err(Error::ValueCommitmentMalformed),
            Ok(inner) => inner,
        };

        let cm_bytes: [u8; 32] = match proto.cm[..].try_into() {
            Err(_) => return Err(Error::NoteCommitmentMalformed),
            Ok(inner) => inner,
        };
        let note_commitment: note::Commitment = match cm_bytes.try_into() {
            Err(_) => return Err(Error::NoteCommitmentMalformed),
            Ok(inner) => inner,
        };

        // xx additional validation here of the pubkey bytes before constructing ka::Public?
        let ephemeral_key_bytes: [u8; 32] = match proto.ephemeral_key[..].try_into() {
            Err(_) => return Err(Error::EphemeralPubKeyMalformed),
            Ok(inner) => inner,
        };
        let ephemeral_key = ka::Public(ephemeral_key_bytes);

        let encrypted_note: [u8; NOTE_ENCRYPTION_BYTES] = match proto.encrypted_note[..].try_into()
        {
            Err(_) => return Err(Error::EncryptedNoteMalformed),
            Ok(inner) => inner,
        };

        // Nothing in this proof yet.
        let proof = OutputProof {};

        Ok(Body {
            value_commitment,
            note_commitment,
            ephemeral_key,
            encrypted_note,
            proof,
        })
    }
}
