use std::convert::{TryFrom, TryInto};

use ark_ff::{PrimeField, UniformRand};
use blake2b_simd;
use chacha20poly1305::{
    aead::{Aead, NewAead},
    ChaCha20Poly1305, Key, Nonce,
};
use decaf377::FieldExt;
use once_cell::sync::Lazy;
use penumbra_proto::crypto as pb;
use rand::Rng;
use serde::{Deserialize, Serialize};
use thiserror;

pub use penumbra_tct::Commitment;

use crate::{
    asset, ka,
    keys::{Diversifier, IncomingViewingKey, OutgoingViewingKey},
    symmetric::{PayloadKey, NOTE_ENCRYPTION_NONCE},
    value, Fq, Value,
};

pub const NOTE_LEN_BYTES: usize = 120;
pub const NOTE_CIPHERTEXT_BYTES: usize = 136;
pub const OVK_WRAPPED_LEN_BYTES: usize = 80;

/// A plaintext Penumbra note.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(into = "pb::Note", try_from = "pb::Note")]
pub struct Note {
    /// The typed value recorded by this note.
    value: Value,
    /// A blinding factor that acts as a commitment trapdoor.
    note_blinding: Fq,
    /// The diversifier of the address controlling this note.
    diversifier: Diversifier,
    /// The diversified transmission key of the address controlling this note.
    transmission_key: ka::Public,
    /// The s-component of the transmission key of the destination address.
    /// We store this separately to ensure that every `Note` is constructed
    /// with a valid transmission key (the `ka::Public` does not validate
    /// the curve point until it is used, since validation is not free).
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
    pub fn from_parts(
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

    /// Generate a fresh note representing the given value for the given destination address, with a
    /// random blinding factor.
    pub fn generate(rng: &mut impl Rng, address: &crate::Address, value: Value) -> Self {
        let diversifier = *address.diversifier();
        let transmission_key = *address.transmission_key();
        let note_blinding = Fq::rand(rng);
        Note::from_parts(diversifier, transmission_key, value, note_blinding)
            .expect("transmission key in address is always valid")
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

        let key = PayloadKey::derive(&shared_secret, &epk);
        let note_plaintext: Vec<u8> = self.into();
        let encryption_result = key.encrypt(note_plaintext, *NOTE_ENCRYPTION_NONCE);

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
        let epk = esk.diversified_public(&self.diversified_generator());
        let kdf_output = derive_ock(ovk, cv, self.commit(), &epk);

        let ock = Key::from_slice(kdf_output.as_bytes());

        let mut op = Vec::new();
        op.extend_from_slice(&self.transmission_key().0);
        op.extend_from_slice(&esk.to_bytes());

        let cipher = ChaCha20Poly1305::new(ock);

        // Note: Here we use the same nonce as note encryption, however the keys are different.
        // For note encryption we derive a symmetric key from the shared secret and epk.
        // However, for encrypting the outgoing cipher key, we derive a symmetric key from the
        // sender's OVK, value commitment, note commitment, and the epk. Since the keys are
        // different, it is safe to use the same nonce.
        //
        // References:
        // * Section 5.4.3 of the ZCash protocol spec
        // * Section 2.3 RFC 7539
        let nonce = Nonce::from_slice(&*NOTE_ENCRYPTION_NONCE);

        let encryption_result = cipher
            .encrypt(nonce, op.as_ref())
            .expect("OVK encryption succeeded");

        let wrapped_ovk: [u8; OVK_WRAPPED_LEN_BYTES] = encryption_result
            .try_into()
            .expect("OVK encryption result fits in ciphertext len");

        wrapped_ovk
    }

    /// Decrypt wrapped OVK to generate the transmission key and ephemeral secret
    pub(crate) fn decrypt_key(
        wrapped_ovk: [u8; OVK_WRAPPED_LEN_BYTES],
        cm: Commitment,
        cv: value::Commitment,
        ovk: &OutgoingViewingKey,
        epk: &ka::Public,
    ) -> Result<(ka::Secret, ka::Public), Error> {
        let kdf_output = derive_ock(ovk, cv, cm, epk);
        let ock = Key::from_slice(kdf_output.as_bytes());

        let cipher = ChaCha20Poly1305::new(ock);
        let nonce = Nonce::from_slice(&*NOTE_ENCRYPTION_NONCE);

        let plaintext = cipher
            .decrypt(nonce, wrapped_ovk.as_ref())
            .expect("OVK decryption succeeded");

        let transmission_key_bytes: [u8; 32] = plaintext[0..32]
            .try_into()
            .map_err(|_| Error::DecryptionError)?;
        let esk_bytes: [u8; 32] = plaintext[32..64]
            .try_into()
            .map_err(|_| Error::DecryptionError)?;
        let esk: ka::Secret = esk_bytes.try_into().map_err(|_| Error::DecryptionError)?;

        Ok((esk, ka::Public(transmission_key_bytes)))
    }

    /// Decrypt a note ciphertext using the wrapped OVK to generate a plaintext `Note`.
    pub fn decrypt_outgoing(
        ciphertext: &[u8],
        wrapped_ovk: [u8; OVK_WRAPPED_LEN_BYTES],
        cm: Commitment,
        cv: value::Commitment,
        ovk: &OutgoingViewingKey,
        epk: &ka::Public,
    ) -> Result<Note, Error> {
        if ciphertext.len() != NOTE_CIPHERTEXT_BYTES {
            return Err(Error::DecryptionError);
        }

        let (esk, transmission_key) =
            Note::decrypt_key(wrapped_ovk, cm, cv, ovk, epk).map_err(|_| Error::DecryptionError)?;
        let shared_secret = esk
            .key_agreement_with(&transmission_key)
            .map_err(|_| Error::DecryptionError)?;
        let key = PayloadKey::derive(&shared_secret, &epk);
        let plaintext = key
            .decrypt(ciphertext.to_vec(), *NOTE_ENCRYPTION_NONCE)
            .map_err(|_| Error::DecryptionError)?;
        let plaintext_bytes: [u8; NOTE_LEN_BYTES] =
            plaintext.try_into().map_err(|_| Error::DecryptionError)?;

        plaintext_bytes
            .try_into()
            .map_err(|_| Error::DecryptionError)
    }

    /// Decrypt a note ciphertext using the IVK and ephemeral public key to generate a plaintext `Note`.
    pub fn decrypt(
        ciphertext: &[u8],
        ivk: &IncomingViewingKey,
        epk: &ka::Public,
    ) -> Result<Note, Error> {
        if ciphertext.len() != NOTE_CIPHERTEXT_BYTES {
            return Err(Error::DecryptionError);
        }

        let shared_secret = ivk
            .key_agreement_with(epk)
            .map_err(|_| Error::DecryptionError)?;

        let key = PayloadKey::derive(&shared_secret, &epk);
        let plaintext = key
            .decrypt(ciphertext.to_vec(), *NOTE_ENCRYPTION_NONCE)
            .map_err(|_| Error::DecryptionError)?;

        let plaintext_bytes: [u8; NOTE_LEN_BYTES] =
            plaintext.try_into().map_err(|_| Error::DecryptionError)?;

        plaintext_bytes
            .try_into()
            .map_err(|_| Error::DecryptionError)
    }

    /// Create the note commitment for this note.
    pub fn commit(&self) -> Commitment {
        self::commitment(
            self.note_blinding,
            self.value,
            self.diversified_generator(),
            self.transmission_key_s,
        )
    }

    pub fn to_bytes(&self) -> [u8; NOTE_LEN_BYTES] {
        self.into()
    }
}

/// Create a note commitment from its parts.
pub fn commitment(
    note_blinding: Fq,
    value: Value,
    diversified_generator: decaf377::Element,
    transmission_key_s: Fq,
) -> Commitment {
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

/// Use Blake2b-256 to derive an encryption key `ock` from the OVK and public fields.
pub(crate) fn derive_ock(
    ovk: &OutgoingViewingKey,
    cv: value::Commitment,
    cm: Commitment,
    epk: &ka::Public,
) -> blake2b_simd::Hash {
    let cv_bytes: [u8; 32] = cv.into();
    let cm_bytes: [u8; 32] = cm.into();

    let mut kdf_params = blake2b_simd::Params::new();
    kdf_params.hash_length(32);
    let mut kdf = kdf_params.to_state();
    kdf.update(&ovk.0);
    kdf.update(&cv_bytes);
    kdf.update(&cm_bytes);
    kdf.update(&epk.0);

    kdf.finalize()
}

impl std::fmt::Debug for Note {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Note")
            .field("value", &self.value)
            .field("diversifier", &self.diversifier())
            .field("transmission_key", &self.transmission_key())
            .field("note_blinding", &self.note_blinding())
            .finish()
    }
}

impl TryFrom<pb::Note> for Note {
    type Error = anyhow::Error;
    fn try_from(msg: pb::Note) -> Result<Self, Self::Error> {
        let diversifier = msg
            .diversifier
            .ok_or_else(|| anyhow::anyhow!("missing diversifier"))?
            .try_into()?;
        let transmission_key = ka::Public::try_from(msg.transmission_key.as_slice())?;
        let value = msg
            .value
            .ok_or_else(|| anyhow::anyhow!("missing value"))?
            .try_into()?;
        let note_blinding = Fq::from_bytes(msg.note_blinding.as_slice().try_into()?)?;

        Ok(Note::from_parts(
            diversifier,
            transmission_key,
            value,
            note_blinding,
        )?)
    }
}

impl From<Note> for pb::Note {
    fn from(msg: Note) -> Self {
        pb::Note {
            diversifier: Some(msg.diversifier().into()),
            transmission_key: msg.transmission_key().0.to_vec(),
            value: Some(msg.value().into()),
            note_blinding: msg.note_blinding().to_bytes().to_vec(),
        }
    }
}

impl From<&Note> for [u8; NOTE_LEN_BYTES] {
    fn from(note: &Note) -> [u8; NOTE_LEN_BYTES] {
        let mut bytes = [0u8; NOTE_LEN_BYTES];
        bytes[0..16].copy_from_slice(&note.diversifier.0);
        bytes[16..24].copy_from_slice(&note.value.amount.to_le_bytes());
        bytes[24..56].copy_from_slice(&note.value.asset_id.0.to_bytes());
        bytes[56..88].copy_from_slice(&note.note_blinding.to_bytes());
        bytes[88..120].copy_from_slice(&note.transmission_key.0);
        bytes
    }
}

impl From<Note> for [u8; NOTE_LEN_BYTES] {
    fn from(note: Note) -> [u8; NOTE_LEN_BYTES] {
        (&note).into()
    }
}

impl From<&Note> for Vec<u8> {
    fn from(note: &Note) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.extend_from_slice(&note.diversifier.0);
        bytes.extend_from_slice(&note.value.amount.to_le_bytes());
        bytes.extend_from_slice(&note.value.asset_id.0.to_bytes());
        bytes.extend_from_slice(&note.note_blinding.to_bytes());
        bytes.extend_from_slice(&note.transmission_key.0);
        bytes
    }
}

impl TryFrom<&[u8]> for Note {
    type Error = Error;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() != NOTE_LEN_BYTES {
            return Err(Error::NoteDeserializationError);
        }

        let amount_bytes: [u8; 8] = bytes[16..24]
            .try_into()
            .map_err(|_| Error::NoteDeserializationError)?;
        let asset_id_bytes: [u8; 32] = bytes[24..56]
            .try_into()
            .map_err(|_| Error::NoteDeserializationError)?;
        let note_blinding_bytes: [u8; 32] = bytes[56..88]
            .try_into()
            .map_err(|_| Error::NoteDeserializationError)?;

        Note::from_parts(
            bytes[0..16]
                .try_into()
                .map_err(|_| Error::NoteDeserializationError)?,
            bytes[88..120]
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

impl TryFrom<[u8; NOTE_LEN_BYTES]> for Note {
    type Error = Error;

    fn try_from(bytes: [u8; NOTE_LEN_BYTES]) -> Result<Note, Self::Error> {
        (&bytes[..]).try_into()
    }
}

#[cfg(test)]
mod tests {
    use decaf377::Fr;
    use rand_core::OsRng;

    use super::*;
    use crate::keys::{SeedPhrase, SpendKey};

    #[test]
    fn note_encryption_and_decryption() {
        let mut rng = OsRng;

        let seed_phrase = SeedPhrase::generate(&mut rng);
        let sk = SpendKey::from_seed_phrase(seed_phrase, 0);
        let fvk = sk.full_viewing_key();
        let ivk = fvk.incoming();
        let (dest, _dtk_d) = ivk.payment_address(0u64.into());

        let value = Value {
            amount: 10,
            asset_id: asset::REGISTRY.parse_denom("upenumbra").unwrap().id(),
        };
        let note = Note::generate(&mut rng, &dest, value);
        let esk = ka::Secret::new(&mut rng);

        let ciphertext = note.encrypt(&esk);

        let epk = esk.diversified_public(dest.diversified_generator());
        let plaintext = Note::decrypt(&ciphertext, ivk, &epk).expect("can decrypt note");

        assert_eq!(plaintext, note);

        let seed_phrase = SeedPhrase::generate(&mut rng);
        let sk2 = SpendKey::from_seed_phrase(seed_phrase, 0);
        let fvk2 = sk2.full_viewing_key();
        let ivk2 = fvk2.incoming();

        assert!(Note::decrypt(&ciphertext, ivk2, &epk).is_err());
    }

    #[test]
    fn note_encryption_and_sender_decryption() {
        let mut rng = OsRng;

        let seed_phrase = SeedPhrase::generate(&mut rng);
        let sk = SpendKey::from_seed_phrase(seed_phrase, 0);
        let fvk = sk.full_viewing_key();
        let ivk = fvk.incoming();
        let ovk = fvk.outgoing();
        let (dest, _dtk_d) = ivk.payment_address(0u64.into());

        let value = Value {
            amount: 10,
            asset_id: asset::REGISTRY.parse_denom("upenumbra").unwrap().id(),
        };
        let note = Note::generate(&mut rng, &dest, value);
        let esk = ka::Secret::new(&mut rng);

        let value_blinding = Fr::rand(&mut rng);
        let cv = note.value.commit(value_blinding);

        let wrapped_ovk = note.encrypt_key(&esk, ovk, cv);
        let ciphertext = note.encrypt(&esk);

        let epk = esk.diversified_public(dest.diversified_generator());
        let plaintext =
            Note::decrypt_outgoing(&ciphertext, wrapped_ovk, note.commit(), cv, ovk, &epk)
                .expect("can decrypt note");

        assert_eq!(plaintext, note);
    }
}
