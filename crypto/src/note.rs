use std::convert::{TryFrom, TryInto};

use ark_ff::PrimeField;
use blake2b_simd;
use decaf377::FieldExt;
use once_cell::sync::Lazy;
use penumbra_proto::core::crypto::v1alpha1 as pb;
use rand::{CryptoRng, Rng};
use serde::{Deserialize, Serialize};
use thiserror;

mod r1cs;
pub use r1cs::{NoteVar, StateCommitmentVar};

pub use penumbra_tct::Commitment;

use crate::{
    asset, balance, fmd, ka,
    keys::{Diversifier, IncomingViewingKey, OutgoingViewingKey},
    symmetric::{OutgoingCipherKey, OvkWrappedKey, PayloadKey, PayloadKind},
    Address, AddressView, Fq, NotePayload, Rseed, Value, ValueView,
};

pub const NOTE_LEN_BYTES: usize = 160;
pub const NOTE_CIPHERTEXT_BYTES: usize = 176;

/// A plaintext Penumbra note.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(into = "pb::Note", try_from = "pb::Note")]
pub struct Note {
    /// The typed value recorded by this note.
    value: Value,
    /// A uniformly random 32-byte sequence used to derive an ephemeral secret key
    /// and note blinding factor.
    rseed: Rseed,
    /// The address controlling this note.
    address: Address,
    /// The s-component of the transmission key of the destination address.
    /// We store this separately to ensure that every `Note` is constructed
    /// with a valid transmission key (the `ka::Public` does not validate
    /// the curve point until it is used, since validation is not free).
    transmission_key_s: Fq,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(into = "pb::NoteView", try_from = "pb::NoteView")]
pub struct NoteView {
    pub value: ValueView,
    pub rseed: Rseed,
    pub address: AddressView,
}

impl NoteView {
    pub fn note(&self) -> Result<Note, Error> {
        self.clone().try_into()
    }
}

impl TryFrom<NoteView> for Note {
    type Error = Error;

    fn try_from(view: NoteView) -> Result<Self, Self::Error> {
        let value = view.value.value();
        let address = view.address.address();
        Note::from_parts(address, value, view.rseed)
    }
}

/// A note ciphertext.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(into = "pb::NoteCiphertext", try_from = "pb::NoteCiphertext")]
pub struct NoteCiphertext(pub [u8; NOTE_CIPHERTEXT_BYTES]);

/// The domain separator used to generate note commitments.
pub(crate) static NOTECOMMIT_DOMAIN_SEP: Lazy<Fq> = Lazy::new(|| {
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
    #[error("Invalid note ciphertext")]
    InvalidNoteCiphertext,
    #[error("Decryption error")]
    DecryptionError,
}

impl Note {
    pub fn from_parts(address: Address, value: Value, rseed: Rseed) -> Result<Self, Error> {
        Ok(Note {
            value,
            rseed,
            address,
            transmission_key_s: Fq::from_bytes(address.transmission_key().0)
                .map_err(|_| Error::InvalidTransmissionKey)?,
        })
    }

    pub fn payload(&self) -> NotePayload {
        NotePayload {
            note_commitment: self.commit(),
            ephemeral_key: self.ephemeral_public_key(),
            encrypted_note: self.encrypt(),
        }
    }

    /// Generate a fresh note representing the given value for the given destination address, with a
    /// random blinding factor.
    pub fn generate(rng: &mut (impl Rng + CryptoRng), address: &Address, value: Value) -> Self {
        let rseed = Rseed::generate(rng);
        Note::from_parts(address.clone(), value, rseed)
            .expect("transmission key in address is always valid")
    }

    pub fn address(&self) -> Address {
        self.address
    }

    pub fn diversified_generator(&self) -> decaf377::Element {
        self.address.diversifier().diversified_generator()
    }

    pub fn transmission_key(&self) -> &ka::Public {
        self.address.transmission_key()
    }

    pub fn transmission_key_s(&self) -> Fq {
        self.transmission_key_s
    }

    pub fn clue_key(&self) -> &fmd::ClueKey {
        self.address.clue_key()
    }

    pub fn diversifier(&self) -> &Diversifier {
        self.address.diversifier()
    }

    pub fn ephemeral_secret_key(&self) -> ka::Secret {
        self.rseed.derive_esk()
    }

    pub fn ephemeral_public_key(&self) -> ka::Public {
        self.ephemeral_secret_key()
            .diversified_public(&self.diversified_generator())
    }

    pub fn note_blinding(&self) -> Fq {
        self.rseed.derive_note_blinding()
    }

    pub fn value(&self) -> Value {
        self.value
    }

    pub fn asset_id(&self) -> asset::Id {
        self.value.asset_id
    }

    pub fn amount(&self) -> asset::Amount {
        self.value.amount
    }

    pub fn rseed(&self) -> Rseed {
        self.rseed
    }

    /// Encrypt a note, returning its ciphertext.
    pub fn encrypt(&self) -> NoteCiphertext {
        let esk = self.ephemeral_secret_key();
        let epk = esk.diversified_public(&self.diversified_generator());
        let shared_secret = esk
            .key_agreement_with(self.transmission_key())
            .expect("key agreement succeeded");

        let key = PayloadKey::derive(&shared_secret, &epk);
        let note_plaintext: Vec<u8> = self.into();
        let encryption_result = key.encrypt(note_plaintext, PayloadKind::Note);

        let ciphertext: [u8; NOTE_CIPHERTEXT_BYTES] = encryption_result
            .try_into()
            .expect("note encryption result fits in ciphertext len");

        NoteCiphertext(ciphertext)
    }

    /// Generate encrypted outgoing cipher key for use with this note.
    pub fn encrypt_key(&self, ovk: &OutgoingViewingKey, cv: balance::Commitment) -> OvkWrappedKey {
        let esk = self.ephemeral_secret_key();
        let epk = esk.diversified_public(&self.diversified_generator());
        let ock = OutgoingCipherKey::derive(ovk, cv, self.commit(), &epk);
        let shared_secret = esk
            .key_agreement_with(self.transmission_key())
            .expect("key agreement succeeded");

        let encryption_result = ock.encrypt(shared_secret.0.to_vec(), PayloadKind::Note);

        OvkWrappedKey(
            encryption_result
                .try_into()
                .expect("OVK encryption result fits in ciphertext len"),
        )
    }

    /// Decrypt wrapped OVK to generate the transmission key and ephemeral secret
    pub fn decrypt_key(
        wrapped_ovk: OvkWrappedKey,
        cm: Commitment,
        cv: balance::Commitment,
        ovk: &OutgoingViewingKey,
        epk: &ka::Public,
    ) -> Result<ka::SharedSecret, Error> {
        let ock = OutgoingCipherKey::derive(ovk, cv, cm, epk);

        let plaintext = ock
            .decrypt(wrapped_ovk.to_vec(), PayloadKind::Note)
            .map_err(|_| Error::DecryptionError)?;

        let shared_secret_bytes: [u8; 32] = plaintext[0..32]
            .try_into()
            .map_err(|_| Error::DecryptionError)?;
        let shared_secret: ka::SharedSecret = shared_secret_bytes
            .try_into()
            .map_err(|_| Error::DecryptionError)?;

        Ok(shared_secret)
    }

    /// Decrypt a note ciphertext using the wrapped OVK to generate a plaintext `Note`.
    pub fn decrypt_outgoing(
        ciphertext: &NoteCiphertext,
        wrapped_ovk: OvkWrappedKey,
        cm: Commitment,
        cv: balance::Commitment,
        ovk: &OutgoingViewingKey,
        epk: &ka::Public,
    ) -> Result<Note, Error> {
        let shared_secret =
            Note::decrypt_key(wrapped_ovk, cm, cv, ovk, epk).map_err(|_| Error::DecryptionError)?;

        let key = PayloadKey::derive(&shared_secret, epk);
        Note::decrypt_with_payload_key(ciphertext, &key, epk)
    }

    /// Decrypt a note ciphertext using the IVK and ephemeral public key to generate a plaintext `Note`.
    pub fn decrypt(
        ciphertext: &NoteCiphertext,
        ivk: &IncomingViewingKey,
        epk: &ka::Public,
    ) -> Result<Note, Error> {
        let shared_secret = ivk
            .key_agreement_with(epk)
            .map_err(|_| Error::DecryptionError)?;

        let key = PayloadKey::derive(&shared_secret, epk);
        Note::decrypt_with_payload_key(ciphertext, &key, epk)
    }

    /// Decrypt a note ciphertext using the [`PayloadKey`].
    pub fn decrypt_with_payload_key(
        ciphertext: &NoteCiphertext,
        payload_key: &PayloadKey,
        epk: &ka::Public,
    ) -> Result<Note, Error> {
        let plaintext = payload_key
            .decrypt(ciphertext.0.to_vec(), PayloadKind::Note)
            .map_err(|_| Error::DecryptionError)?;

        let plaintext_bytes: [u8; NOTE_LEN_BYTES] =
            plaintext.try_into().map_err(|_| Error::DecryptionError)?;

        let note: Note = plaintext_bytes
            .try_into()
            .map_err(|_| Error::DecryptionError)?;

        // Ephemeral public key integrity check. See ZIP 212 or Penumbra issue #1688.
        if note.ephemeral_public_key() != epk.clone() {
            return Err(Error::DecryptionError);
        }

        Ok(note)
    }

    /// Create the note commitment for this note.
    pub fn commit(&self) -> Commitment {
        self::commitment(
            self.note_blinding(),
            self.value,
            self.diversified_generator(),
            self.transmission_key_s,
            self.address.clue_key(),
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
    clue_key: &fmd::ClueKey,
) -> Commitment {
    let commit = poseidon377::hash_6(
        &NOTECOMMIT_DOMAIN_SEP,
        (
            note_blinding,
            value.amount.into(),
            value.asset_id.0,
            diversified_generator.vartime_compress_to_field(),
            transmission_key_s,
            Fq::from_le_bytes_mod_order(&clue_key.0[..]),
        ),
    );

    Commitment(commit)
}

/// Create a note commitment from the blinding factor, value, and address.
pub fn commitment_from_address(
    address: Address,
    value: Value,
    note_blinding: Fq,
) -> Result<Commitment, Error> {
    let transmission_key_s =
        Fq::from_bytes(address.transmission_key().0).map_err(|_| Error::InvalidTransmissionKey)?;
    let commit = poseidon377::hash_6(
        &NOTECOMMIT_DOMAIN_SEP,
        (
            note_blinding,
            value.amount.into(),
            value.asset_id.0,
            address.diversified_generator().vartime_compress_to_field(),
            transmission_key_s,
            Fq::from_le_bytes_mod_order(&address.clue_key().0[..]),
        ),
    );

    Ok(Commitment(commit))
}

impl std::fmt::Debug for Note {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Note")
            .field("value", &self.value)
            .field("address", &self.address())
            .field("rseed", &hex::encode(self.rseed.to_bytes()))
            .finish()
    }
}

impl TryFrom<pb::Note> for Note {
    type Error = anyhow::Error;
    fn try_from(msg: pb::Note) -> Result<Self, Self::Error> {
        let address = msg
            .address
            .ok_or_else(|| anyhow::anyhow!("missing value"))?
            .try_into()?;
        let value = msg
            .value
            .ok_or_else(|| anyhow::anyhow!("missing value"))?
            .try_into()?;
        let rseed = Rseed(msg.rseed.as_slice().try_into()?);

        Ok(Note::from_parts(address, value, rseed)?)
    }
}

impl From<Note> for pb::Note {
    fn from(msg: Note) -> Self {
        pb::Note {
            address: Some(msg.address().into()),
            value: Some(msg.value().into()),
            rseed: msg.rseed.to_bytes().to_vec(),
        }
    }
}

impl From<NoteView> for pb::NoteView {
    fn from(msg: NoteView) -> Self {
        pb::NoteView {
            address: Some(msg.address.into()),
            value: Some(msg.value.into()),
            rseed: msg.rseed.to_bytes().to_vec(),
        }
    }
}

impl TryFrom<pb::NoteView> for NoteView {
    type Error = anyhow::Error;
    fn try_from(msg: pb::NoteView) -> Result<Self, Self::Error> {
        let address = msg
            .address
            .ok_or_else(|| anyhow::anyhow!("missing value"))?
            .try_into()?;
        let value = msg
            .value
            .ok_or_else(|| anyhow::anyhow!("missing value"))?
            .try_into()?;
        let rseed = Rseed(msg.rseed.as_slice().try_into()?);

        Ok(NoteView {
            address,
            value,
            rseed,
        })
    }
}

impl From<&Note> for [u8; NOTE_LEN_BYTES] {
    fn from(note: &Note) -> [u8; NOTE_LEN_BYTES] {
        let mut bytes = [0u8; NOTE_LEN_BYTES];
        bytes[0..80].copy_from_slice(&note.address.to_vec());
        bytes[80..96].copy_from_slice(&note.value.amount.to_le_bytes());
        bytes[96..128].copy_from_slice(&note.value.asset_id.0.to_bytes());
        bytes[128..160].copy_from_slice(&note.rseed.to_bytes());
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
        bytes.extend_from_slice(&note.address().to_vec());
        bytes.extend_from_slice(&note.value.amount.to_le_bytes());
        bytes.extend_from_slice(&note.value.asset_id.0.to_bytes());
        bytes.extend_from_slice(&note.rseed.to_bytes());
        bytes
    }
}

impl TryFrom<&[u8]> for Note {
    type Error = Error;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() != NOTE_LEN_BYTES {
            return Err(Error::NoteDeserializationError);
        }

        let amount_bytes: [u8; 16] = bytes[80..96]
            .try_into()
            .map_err(|_| Error::NoteDeserializationError)?;
        let asset_id_bytes: [u8; 32] = bytes[96..128]
            .try_into()
            .map_err(|_| Error::NoteDeserializationError)?;
        let rseed_bytes: [u8; 32] = bytes[128..160]
            .try_into()
            .map_err(|_| Error::NoteDeserializationError)?;

        Note::from_parts(
            bytes[0..80]
                .try_into()
                .map_err(|_| Error::NoteDeserializationError)?,
            Value {
                amount: crate::asset::Amount::from_le_bytes(amount_bytes),
                asset_id: asset::Id(
                    Fq::from_bytes(asset_id_bytes).map_err(|_| Error::NoteDeserializationError)?,
                ),
            },
            Rseed(rseed_bytes),
        )
    }
}

impl TryFrom<[u8; NOTE_LEN_BYTES]> for Note {
    type Error = Error;

    fn try_from(bytes: [u8; NOTE_LEN_BYTES]) -> Result<Note, Self::Error> {
        (&bytes[..]).try_into()
    }
}

impl TryFrom<pb::NoteCiphertext> for NoteCiphertext {
    type Error = Error;

    fn try_from(msg: pb::NoteCiphertext) -> Result<Self, Self::Error> {
        if msg.inner.len() != NOTE_CIPHERTEXT_BYTES {
            return Err(Error::InvalidNoteCiphertext);
        }

        let inner_bytes: [u8; NOTE_CIPHERTEXT_BYTES] = msg
            .inner
            .try_into()
            .map_err(|_| Error::InvalidNoteCiphertext)?;

        Ok(NoteCiphertext(inner_bytes))
    }
}

impl From<NoteCiphertext> for pb::NoteCiphertext {
    fn from(msg: NoteCiphertext) -> Self {
        pb::NoteCiphertext {
            inner: msg.0.to_vec(),
        }
    }
}

#[cfg(test)]
mod tests {
    use ark_ff::UniformRand;
    use decaf377::Fr;
    use rand_core::OsRng;

    use super::*;
    use crate::keys::{SeedPhrase, SpendKey};

    #[test]
    fn note_encryption_and_decryption() {
        let mut rng = OsRng;

        let seed_phrase = SeedPhrase::generate(rng);
        let sk = SpendKey::from_seed_phrase(seed_phrase, 0);
        let fvk = sk.full_viewing_key();
        let ivk = fvk.incoming();
        let (dest, _dtk_d) = ivk.payment_address(0u32.into());

        let value = Value {
            amount: 10u64.into(),
            asset_id: asset::REGISTRY.parse_denom("upenumbra").unwrap().id(),
        };
        let note = Note::generate(&mut rng, &dest, value);

        let ciphertext = note.encrypt();

        let esk = note.ephemeral_secret_key();
        let epk = esk.diversified_public(dest.diversified_generator());
        let plaintext = Note::decrypt(&ciphertext, ivk, &epk).expect("can decrypt note");

        assert_eq!(plaintext, note);

        let seed_phrase = SeedPhrase::generate(rng);
        let sk2 = SpendKey::from_seed_phrase(seed_phrase, 0);
        let fvk2 = sk2.full_viewing_key();
        let ivk2 = fvk2.incoming();

        assert!(Note::decrypt(&ciphertext, ivk2, &epk).is_err());
    }

    #[test]
    fn note_encryption_and_sender_decryption() {
        let mut rng = OsRng;

        let seed_phrase = SeedPhrase::generate(rng);
        let sk = SpendKey::from_seed_phrase(seed_phrase, 0);
        let fvk = sk.full_viewing_key();
        let ivk = fvk.incoming();
        let ovk = fvk.outgoing();
        let (dest, _dtk_d) = ivk.payment_address(0u32.into());

        let value = Value {
            amount: 10u64.into(),
            asset_id: asset::REGISTRY.parse_denom("upenumbra").unwrap().id(),
        };
        let note = Note::generate(&mut rng, &dest, value);

        let value_blinding = Fr::rand(&mut rng);
        let cv = note.value.commit(value_blinding);

        let wrapped_ovk = note.encrypt_key(ovk, cv);
        let ciphertext = note.encrypt();

        let esk = note.ephemeral_secret_key();
        let epk = esk.diversified_public(dest.diversified_generator());
        let plaintext =
            Note::decrypt_outgoing(&ciphertext, wrapped_ovk, note.commit(), cv, ovk, &epk)
                .expect("can decrypt note");

        assert_eq!(plaintext, note);
    }

    #[test]
    fn note_decryption_fails_with_incorrect_epk() {
        let mut rng = OsRng;

        let seed_phrase = SeedPhrase::generate(rng);
        let sk = SpendKey::from_seed_phrase(seed_phrase, 0);
        let fvk = sk.full_viewing_key();
        let ivk = fvk.incoming();
        let (dest, _dtk_d) = ivk.payment_address(0u32.into());

        let value = Value {
            amount: 10u64.into(),
            asset_id: asset::REGISTRY.parse_denom("upenumbra").unwrap().id(),
        };
        let note = Note::generate(&mut rng, &dest, value);

        let ciphertext = note.encrypt();

        let wrong_esk = ka::Secret::new(&mut rng);
        let wrong_epk = wrong_esk.diversified_public(dest.diversified_generator());
        let decryption_result = Note::decrypt(&ciphertext, ivk, &wrong_epk);

        assert!(decryption_result.is_err());
    }
}
