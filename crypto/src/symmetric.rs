use anyhow::Result;
use chacha20poly1305::{
    aead::{Aead, NewAead},
    ChaCha20Poly1305, Key, Nonce,
};
use once_cell::sync::Lazy;

use crate::ka;

/// The nonce used for note encryption.
pub static NOTE_ENCRYPTION_NONCE: Lazy<[u8; 12]> = Lazy::new(|| [0u8; 12]);

/// The nonce used for memo encryption.
pub static MEMO_ENCRYPTION_NONCE: Lazy<[u8; 12]> = Lazy::new(|| {
    let nonce_bytes = 1u128.to_le_bytes();
    nonce_bytes[0..12].try_into().expect("nonce fits in array")
});

/// The nonce used for swap encryption.
pub static SWAP_ENCRYPTION_NONCE: Lazy<[u8; 12]> = Lazy::new(|| [9u8; 12]);

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
    pub fn encrypt(&self, plaintext: Vec<u8>, nonce: [u8; 12]) -> Vec<u8> {
        let cipher = ChaCha20Poly1305::new(&self.0);
        let nonce = Nonce::from_slice(&nonce);

        cipher
            .encrypt(nonce, plaintext.as_ref())
            .expect("encryption succeeded")
    }

    /// Decrypt a note, swap, or memo using the `PayloadKey`.
    pub fn decrypt(&self, ciphertext: Vec<u8>, nonce: [u8; 12]) -> Result<Vec<u8>> {
        let cipher = ChaCha20Poly1305::new(&self.0);
        let nonce = Nonce::from_slice(&nonce);

        cipher
            .decrypt(nonce, ciphertext.as_ref())
            .map_err(|_| anyhow::anyhow!("decryption error"))
    }
}
