use anyhow::Result;
use chacha20poly1305::{
    aead::{Aead, NewAead},
    ChaCha20Poly1305, Key, Nonce,
};

use crate::ka;

/// Represents the item to be encrypted/decrypted with the [`PayloadKey`].
pub enum PayloadKind {
    Note,
    Memo,
    Swap,
}

impl PayloadKind {
    pub(crate) fn nonce(&self) -> [u8; 12] {
        match self {
            Self::Note => [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            Self::Memo => [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            Self::Swap => [2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        }
    }
}

/// Represents a symmetric `ChaCha20Poly1305` key.
///
/// Used for encrypting and decrypting notes, memos and swaps.
pub struct PayloadKey(Key);

impl PayloadKey {
    /// Use Blake2b-256 to derive the symmetric key material for note and memo encryption.
    pub fn derive(shared_secret: &ka::SharedSecret, epk: &ka::Public) -> Self {
        let mut kdf_params = blake2b_simd::Params::new();
        kdf_params.hash_length(32);
        let mut kdf = kdf_params.to_state();
        kdf.update(&shared_secret.0);
        kdf.update(&epk.0);

        let key = kdf.finalize();
        Self(*Key::from_slice(key.as_bytes()))
    }

    /// Encrypt a note, swap, or memo using the `PayloadKey`.
    pub fn encrypt(&self, plaintext: Vec<u8>, kind: PayloadKind) -> Vec<u8> {
        let cipher = ChaCha20Poly1305::new(&self.0);
        let nonce_bytes = kind.nonce();
        let nonce = Nonce::from_slice(&nonce_bytes);

        cipher
            .encrypt(nonce, plaintext.as_ref())
            .expect("encryption succeeded")
    }

    /// Decrypt a note, swap, or memo using the `PayloadKey`.
    pub fn decrypt(&self, ciphertext: Vec<u8>, kind: PayloadKind) -> Result<Vec<u8>> {
        let cipher = ChaCha20Poly1305::new(&self.0);
        let nonce_bytes = kind.nonce();
        let nonce = Nonce::from_slice(&nonce_bytes);

        cipher
            .decrypt(nonce, ciphertext.as_ref())
            .map_err(|_| anyhow::anyhow!("decryption error"))
    }
}
