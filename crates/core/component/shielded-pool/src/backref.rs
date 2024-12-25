use anyhow::Result;
use chacha20poly1305::{
    aead::{Aead, NewAead},
    ChaCha20Poly1305, Nonce,
};

use penumbra_sdk_keys::BackreferenceKey;
use penumbra_sdk_sct::Nullifier;
use penumbra_sdk_tct as tct;

pub const ENCRYPTED_BACKREF_LEN: usize = 48;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Backref {
    note_commitment: tct::StateCommitment,
}

#[derive(Clone, Debug)]
pub struct EncryptedBackref {
    /// The inner bytes can either have 0 or `ENCRYPTED_BACKREF_LEN` bytes.
    bytes: Vec<u8>,
}

impl Backref {
    pub fn new(note_commitment: tct::StateCommitment) -> Self {
        Self { note_commitment }
    }

    pub fn encrypt(&self, brk: &BackreferenceKey, nullifier: &Nullifier) -> EncryptedBackref {
        let cipher = ChaCha20Poly1305::new(&brk.0);

        // Nonce is the first 12 bytes of the nullifier
        let nonce_bytes = &nullifier.to_bytes()[..12];
        let nonce = Nonce::from_slice(&nonce_bytes);

        let plaintext = self.note_commitment.0.to_bytes();

        let ciphertext = cipher
            .encrypt(nonce, plaintext.as_ref())
            .expect("encryption should succeed  ");

        EncryptedBackref { bytes: ciphertext }
    }
}

impl EncryptedBackref {
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    pub fn dummy() -> Self {
        Self { bytes: vec![] }
    }

    /// Decrypts the encrypted backref, returning a backref if the decryption is successful,
    /// or `None` if the encrypted backref is zero-length.
    pub fn decrypt(
        &self,
        brk: &BackreferenceKey,
        nullifier: &Nullifier,
    ) -> Result<Option<Backref>> {
        // We might have a 0-length encrypted backref, which
        // is treated as a valid value and means that the note has no backref.
        if self.is_empty() {
            return Ok(None);
        }

        let cipher = ChaCha20Poly1305::new(&brk.0);

        let nonce_bytes = &nullifier.to_bytes()[..12];
        let nonce = Nonce::from_slice(&nonce_bytes);

        let plaintext = cipher
            .decrypt(nonce, self.bytes.as_ref())
            .map_err(|_| anyhow::anyhow!("decryption error"))?;

        let note_commitment_bytes: [u8; 32] = plaintext
            .try_into()
            .map_err(|_| anyhow::anyhow!("decryption error"))?;

        let backref = Backref::try_from(note_commitment_bytes)
            .map_err(|_| anyhow::anyhow!("decryption error"))?;

        Ok(Some(backref))
    }
}

impl TryFrom<[u8; 32]> for Backref {
    type Error = anyhow::Error;

    fn try_from(bytes: [u8; 32]) -> Result<Self> {
        Ok(Self {
            note_commitment: tct::StateCommitment::try_from(bytes)
                .map_err(|_| anyhow::anyhow!("invalid note commitment"))?,
        })
    }
}

// EncryptedBackrefs can either have 0 or ENCRYPTED_BACKREF_LEN bytes.

impl TryFrom<[u8; ENCRYPTED_BACKREF_LEN]> for EncryptedBackref {
    type Error = anyhow::Error;

    fn try_from(bytes: [u8; ENCRYPTED_BACKREF_LEN]) -> Result<Self> {
        Ok(Self {
            bytes: bytes.to_vec(),
        })
    }
}

impl TryFrom<[u8; 0]> for EncryptedBackref {
    type Error = anyhow::Error;

    fn try_from(bytes: [u8; 0]) -> Result<Self> {
        Ok(Self {
            bytes: bytes.to_vec(),
        })
    }
}

impl From<EncryptedBackref> for Vec<u8> {
    fn from(encrypted_backref: EncryptedBackref) -> Vec<u8> {
        encrypted_backref.bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    use penumbra_sdk_asset::{asset, Value};
    use penumbra_sdk_keys::keys::{Bip44Path, SeedPhrase, SpendKey};

    use crate::{Note, Rseed};

    proptest! {
        #[test]
        fn encrypted_backref_zero_length(seed_phrase_randomness in any::<[u8; 32]>(), amount_to_send in any::<u64>(), rseed_randomness in any::<[u8; 32]>()) {
            let seed_phrase = SeedPhrase::from_randomness(&seed_phrase_randomness);
            let sk = SpendKey::from_seed_phrase_bip44(seed_phrase, &Bip44Path::new(0));
            let fvk = sk.full_viewing_key();
            let brk = fvk.backref_key();

            let ivk = fvk.incoming();
            let (sender, _dtk_d) = ivk.payment_address(0u32.into());

            let value_to_send = Value {
                amount: amount_to_send.into(),
                asset_id: asset::Cache::with_known_assets()
                    .get_unit("upenumbra")
                    .unwrap()
                    .id(),
            };
            let rseed = Rseed(rseed_randomness);

            let note = Note::from_parts(sender, value_to_send, rseed).expect("valid note");
            let note_commitment: penumbra_sdk_tct::StateCommitment = note.commit();
            let nk = *sk.nullifier_key();
            let mut sct = tct::Tree::new();

            sct.insert(tct::Witness::Keep, note_commitment).unwrap();
            let state_commitment_proof = sct.witness(note_commitment).unwrap();
            let nullifier = Nullifier::derive(&nk, state_commitment_proof.position(), &note_commitment);

            let encrypted_backref = EncryptedBackref::dummy();
            assert!(encrypted_backref.is_empty());
            assert_eq!(encrypted_backref.len(), 0);

            // Decrypting a zero-length encrypted backref should return `None`.
            let decrypted_backref = encrypted_backref.decrypt(&brk, &nullifier).unwrap();
            assert_eq!(decrypted_backref, None);
        }
    }

    proptest! {
        #[test]
        fn encrypted_backref_round_trip(seed_phrase_randomness in any::<[u8; 32]>(), amount_to_send in any::<u64>(), rseed_randomness in any::<[u8; 32]>()) {
            let seed_phrase = SeedPhrase::from_randomness(&seed_phrase_randomness);
            let sk = SpendKey::from_seed_phrase_bip44(seed_phrase, &Bip44Path::new(0));
            let fvk = sk.full_viewing_key();
            let brk = fvk.backref_key();

            let ivk = fvk.incoming();
            let (sender, _dtk_d) = ivk.payment_address(0u32.into());

            let value_to_send = Value {
                amount: amount_to_send.into(),
                asset_id: asset::Cache::with_known_assets()
                    .get_unit("upenumbra")
                    .unwrap()
                    .id(),
            };
            let rseed = Rseed(rseed_randomness);

            let note = Note::from_parts(sender, value_to_send, rseed).expect("valid note");
            let note_commitment: penumbra_sdk_tct::StateCommitment = note.commit();
            let nk = *sk.nullifier_key();
            let mut sct = tct::Tree::new();

            sct.insert(tct::Witness::Keep, note_commitment).unwrap();
            let state_commitment_proof = sct.witness(note_commitment).unwrap();
            let nullifier = Nullifier::derive(&nk, state_commitment_proof.position(), &note_commitment);

            let backref = Backref::new(note_commitment);
            let encrypted_backref = backref.encrypt(&brk, &nullifier);

            let decrypted_backref = encrypted_backref.decrypt(&brk, &nullifier).unwrap();

            assert_eq!(Some(backref), decrypted_backref);
        }
    }
}
