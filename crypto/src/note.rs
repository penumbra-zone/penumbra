use ark_ff::PrimeField;
use blake2b_simd;
use chacha20poly1305::{
    aead::{Aead, NewAead},
    ChaCha20Poly1305, Key, Nonce,
};
use decaf377::FieldExt;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::convert::{TryFrom, TryInto};
use thiserror;

use crate::{
    asset, ka,
    keys::{Diversifier, IncomingViewingKey, OutgoingViewingKey},
    value, Fq, Value,
};

pub const NOTE_LEN_BYTES: usize = 116;
pub const NOTE_CIPHERTEXT_BYTES: usize = 132;
pub const OVK_WRAPPED_LEN_BYTES: usize = 80;

/// The nonce used for note encryption.
pub static NOTE_ENCRYPTION_NONCE: Lazy<[u8; 12]> = Lazy::new(|| [0u8; 12]);

// Can add to this/make this an enum when we add additional types of notes.
pub const NOTE_TYPE: u8 = 0;

/// A plaintext Penumbra note.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Note {
    // Value (32-byte asset ID plus 8-byte amount).
    value: Value,

    // Commitment trapdoor. 32 bytes.
    note_blinding: Fq,

    // The diversifier of the destination address. 11 bytes.
    diversifier: Diversifier,

    // The diversified transmission key of the destination address. 32 bytes.
    transmission_key: ka::Public,

    // The s-component of the transmission key of the destination address.
    transmission_key_s: Fq,
}

/// The domain separator used to generate note commitments.
static NOTECOMMIT_DOMAIN_SEP: Lazy<Fq> = Lazy::new(|| {
    Fq::from_le_bytes_mod_order(blake2b_simd::blake2b(b"penumbra.notecommit").as_bytes())
});

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Invalid note commitment")]
    InvalidNoteCommitment,
    #[error("Invalid transmission key")]
    InvalidTransmissionKey,
    #[error("Note type unsupported")]
    NoteTypeUnsupported,
    #[error("Note deserialization error")]
    NoteDeserializationError,
    #[error("Decryption error")]
    DecryptionError,
}

impl Note {
    pub fn new(
        diversifier: Diversifier,
        transmission_key: ka::Public,
        value: Value,
        note_blinding: Fq,
    ) -> Result<Self, Error> {
        Ok(Note {
            value,
            note_blinding,
            diversifier,
            transmission_key,
            transmission_key_s: Fq::from_bytes(transmission_key.0)
                .map_err(|_| Error::InvalidTransmissionKey)?,
        })
    }

    pub fn diversified_generator(&self) -> decaf377::Element {
        self.diversifier.diversified_generator()
    }

    pub fn transmission_key(&self) -> ka::Public {
        self.transmission_key
    }

    pub fn transmission_key_s(&self) -> Fq {
        self.transmission_key_s
    }

    pub fn diversifier(&self) -> Diversifier {
        self.diversifier
    }

    pub fn note_blinding(&self) -> Fq {
        self.note_blinding
    }

    pub fn value(&self) -> Value {
        self.value
    }

    pub fn asset_id(&self) -> asset::Id {
        self.value.asset_id
    }

    pub fn amount(&self) -> u64 {
        self.value.amount
    }

    /// Encrypt a note, returning its ciphertext.
    pub fn encrypt(&self, esk: &ka::Secret) -> [u8; NOTE_CIPHERTEXT_BYTES] {
        let epk = esk.diversified_public(&self.diversified_generator());
        let shared_secret = esk
            .key_agreement_with(&self.transmission_key())
            .expect("key agreement succeeded");

        let key = derive_symmetric_key(&shared_secret, &epk);
        let cipher = ChaCha20Poly1305::new(Key::from_slice(key.as_bytes()));
        let nonce = Nonce::from_slice(&*NOTE_ENCRYPTION_NONCE);

        let note_plaintext: Vec<u8> = self.into();
        let encryption_result = cipher
            .encrypt(nonce, note_plaintext.as_ref())
            .expect("note encryption succeeded");

        let ciphertext: [u8; NOTE_CIPHERTEXT_BYTES] = encryption_result
            .try_into()
            .expect("note encryption result fits in ciphertext len");

        ciphertext
    }

    /// Generate encrypted outgoing cipher key for use with this note.
    pub fn encrypt_key(
        &self,
        esk: &ka::Secret,
        ovk: &OutgoingViewingKey,
        cv: value::Commitment,
    ) -> [u8; OVK_WRAPPED_LEN_BYTES] {
        let cv_bytes: [u8; 32] = cv.into();
        let cm_bytes: [u8; 32] = self.commit().into();
        let epk = esk.diversified_public(&self.diversified_generator());

        // Use Blake2b-256 to derive an encryption key `ock` from the value commitment,
        // note commitment, the ephemeral public key, and the outgoing viewing key.
        let mut kdf_params = blake2b_simd::Params::new();
        kdf_params.hash_length(32);
        let mut kdf = kdf_params.to_state();
        kdf.update(&ovk.0);
        kdf.update(&cv_bytes);
        kdf.update(&cm_bytes);
        kdf.update(&epk.0);
        let kdf_output = kdf.finalize();
        let ock = Key::from_slice(kdf_output.as_bytes());

        let mut op = Vec::new();
        op.extend_from_slice(&self.transmission_key().0);
        op.extend_from_slice(&esk.to_bytes());

        let cipher = ChaCha20Poly1305::new(ock);
        let nonce = Nonce::from_slice(&*NOTE_ENCRYPTION_NONCE);

        let encryption_result = cipher
            .encrypt(nonce, op.as_ref())
            .expect("OVK encryption succeeded");

        let wrapped_ovk: [u8; OVK_WRAPPED_LEN_BYTES] = encryption_result
            .try_into()
            .expect("OVK encryption result fits in ciphertext len");

        wrapped_ovk
    }

    /// Decrypt a note ciphertext to generate a plaintext `Note`.
    pub fn decrypt(
        ciphertext: [u8; NOTE_CIPHERTEXT_BYTES],
        ivk: &IncomingViewingKey,
        epk: &ka::Public,
    ) -> Result<Note, Error> {
        let shared_secret = ivk
            .key_agreement_with(epk)
            .map_err(|_| Error::DecryptionError)?;

        let key = derive_symmetric_key(&shared_secret, epk);
        let cipher = ChaCha20Poly1305::new(Key::from_slice(key.as_bytes()));
        let nonce = Nonce::from_slice(&[0u8; 12]);
        let plaintext = cipher
            .decrypt(nonce, ciphertext.as_ref())
            .map_err(|_| Error::DecryptionError)?;

        let plaintext_bytes: [u8; NOTE_LEN_BYTES] =
            plaintext.try_into().map_err(|_| Error::DecryptionError)?;

        plaintext_bytes
            .try_into()
            .map_err(|_| Error::DecryptionError)
    }

    pub fn commit(&self) -> Commitment {
        Commitment::new(
            self.note_blinding,
            self.value,
            self.diversified_generator(),
            self.transmission_key_s,
        )
    }
}

/// Use Blake2b-256 to derive the symmetric key material for note and memo encryption.
pub(crate) fn derive_symmetric_key(
    shared_secret: &ka::SharedSecret,
    epk: &ka::Public,
) -> blake2b_simd::Hash {
    let mut kdf_params = blake2b_simd::Params::new();
    kdf_params.hash_length(32);
    let mut kdf = kdf_params.to_state();
    kdf.update(&shared_secret.0);
    kdf.update(&epk.0);

    kdf.finalize()
}

impl From<Note> for [u8; NOTE_LEN_BYTES] {
    fn from(note: Note) -> [u8; NOTE_LEN_BYTES] {
        let mut bytes = [0u8; NOTE_LEN_BYTES];
        bytes[0] = NOTE_TYPE;
        bytes[1..12].copy_from_slice(&note.diversifier.0);
        bytes[12..20].copy_from_slice(&note.value.amount.to_le_bytes());
        bytes[20..52].copy_from_slice(&note.value.asset_id.0.to_bytes());
        bytes[52..84].copy_from_slice(&note.note_blinding.to_bytes());
        bytes[84..116].copy_from_slice(&note.transmission_key.0);
        bytes
    }
}

impl From<&Note> for [u8; NOTE_LEN_BYTES] {
    fn from(note: &Note) -> [u8; NOTE_LEN_BYTES] {
        note.into()
    }
}

impl From<&Note> for Vec<u8> {
    fn from(note: &Note) -> Vec<u8> {
        let mut bytes = vec![NOTE_TYPE];
        bytes.extend_from_slice(&note.diversifier.0);
        bytes.extend_from_slice(&note.value.amount.to_le_bytes());
        bytes.extend_from_slice(&note.value.asset_id.0.to_bytes());
        bytes.extend_from_slice(&note.note_blinding.to_bytes());
        bytes.extend_from_slice(&note.transmission_key.0);
        bytes
    }
}

impl TryFrom<[u8; NOTE_LEN_BYTES]> for Note {
    type Error = Error;

    fn try_from(bytes: [u8; NOTE_LEN_BYTES]) -> Result<Note, Self::Error> {
        if bytes[0] != NOTE_TYPE {
            return Err(Error::NoteTypeUnsupported);
        }

        let amount_bytes: [u8; 8] = bytes[12..20]
            .try_into()
            .map_err(|_| Error::NoteDeserializationError)?;
        let asset_id_bytes: [u8; 32] = bytes[20..52]
            .try_into()
            .map_err(|_| Error::NoteDeserializationError)?;
        let note_blinding_bytes: [u8; 32] = bytes[52..84]
            .try_into()
            .map_err(|_| Error::NoteDeserializationError)?;

        Note::new(
            bytes[1..12]
                .try_into()
                .map_err(|_| Error::NoteDeserializationError)?,
            bytes[84..116]
                .try_into()
                .map_err(|_| Error::NoteDeserializationError)?,
            Value {
                amount: u64::from_le_bytes(amount_bytes),
                asset_id: asset::Id(
                    Fq::from_bytes(asset_id_bytes).map_err(|_| Error::NoteDeserializationError)?,
                ),
            },
            Fq::from_bytes(note_blinding_bytes).map_err(|_| Error::NoteDeserializationError)?,
        )
    }
}

// Note commitment.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(into = "[u8; 32]", try_from = "[u8; 32]")]
pub struct Commitment(pub Fq);

impl Commitment {
    pub fn new(
        note_blinding: Fq,
        value: Value,
        diversified_generator: decaf377::Element,
        transmission_key_s: Fq,
    ) -> Self {
        let commit = poseidon377::hash_5(
            &NOTECOMMIT_DOMAIN_SEP,
            (
                note_blinding,
                value.amount.into(),
                value.asset_id.0,
                diversified_generator.compress_to_field(),
                transmission_key_s,
            ),
        );

        Commitment(commit)
    }
}

impl From<Commitment> for [u8; 32] {
    fn from(commitment: Commitment) -> [u8; 32] {
        commitment.0.to_bytes()
    }
}

impl Into<Vec<u8>> for Commitment {
    fn into(self) -> Vec<u8> {
        self.0.to_bytes().to_vec()
    }
}

impl TryFrom<[u8; 32]> for Commitment {
    type Error = Error;

    fn try_from(bytes: [u8; 32]) -> Result<Commitment, Self::Error> {
        let inner = Fq::from_bytes(bytes).map_err(|_| Error::InvalidNoteCommitment)?;

        Ok(Commitment(inner))
    }
}

impl TryFrom<&[u8]> for Commitment {
    type Error = Error;

    fn try_from(slice: &[u8]) -> Result<Commitment, Self::Error> {
        let bytes: [u8; 32] = slice[..]
            .try_into()
            .map_err(|_| Error::InvalidNoteCommitment)?;

        let inner = Fq::from_bytes(bytes).map_err(|_| Error::InvalidNoteCommitment)?;

        Ok(Commitment(inner))
    }
}

#[cfg(feature = "sqlx")]
mod sqlx_impls {
    use super::*;

    use sqlx::{Database, Decode, Encode, Postgres, Type};

    impl<'r> Decode<'r, Postgres> for Commitment {
        fn decode(
            value: <Postgres as sqlx::database::HasValueRef<'r>>::ValueRef,
        ) -> Result<Self, sqlx::error::BoxDynError> {
            let bytes = Vec::<u8>::decode(value)?;
            Commitment::try_from(&bytes[..]).map_err(Into::into)
        }
    }

    impl<'q> Encode<'q, Postgres> for Commitment {
        fn encode_by_ref(
            &self,
            buf: &mut <Postgres as sqlx::database::HasArguments<'q>>::ArgumentBuffer,
        ) -> sqlx::encode::IsNull {
            let bytes = self.0.to_bytes();
            (&bytes[..]).encode(buf)
        }
    }

    impl Type<Postgres> for Commitment {
        fn type_info() -> <Postgres as Database>::TypeInfo {
            <[u8] as Type<Postgres>>::type_info()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::keys::SpendKey;
    use ark_ff::UniformRand;
    use rand_core::OsRng;

    #[test]
    fn test_note_encryption_and_decryption() {
        let mut rng = OsRng;

        let sk = SpendKey::generate(&mut rng);
        let fvk = sk.full_viewing_key();
        let ivk = fvk.incoming();
        let (dest, _dtk_d) = ivk.payment_address(0u64.into());

        let value = Value {
            amount: 10,
            asset_id: b"pen".as_ref().into(),
        };
        let note = Note::new(
            *dest.diversifier(),
            *dest.transmission_key(),
            value,
            Fq::rand(&mut rng),
        )
        .expect("can create note");
        let esk = ka::Secret::new(&mut rng);

        let ciphertext = note.encrypt(&esk);

        let epk = esk.diversified_public(dest.diversified_generator());
        let plaintext = Note::decrypt(ciphertext, ivk, &epk).expect("can decrypt note");

        assert_eq!(plaintext, note);

        let sk2 = SpendKey::generate(&mut rng);
        let fvk2 = sk2.full_viewing_key();
        let ivk2 = fvk2.incoming();

        assert!(Note::decrypt(ciphertext, ivk2, &epk).is_err());
    }
}
