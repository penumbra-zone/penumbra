use chacha20poly1305::{
    aead::{Aead, NewAead},
    ChaCha20Poly1305, Key, Nonce,
};
use std::convert::TryInto;

use crate::ka;

pub const MEMO_CIPHERTEXT_LEN_BYTES: usize = 528;

// This is the `MEMO_CIPHERTEXT_LEN_BYTES` - MAC size (16 bytes).
pub const MEMO_LEN_BYTES: usize = 512;

// The memo is stored separately from the `Note`.
#[derive(Clone)]
pub struct MemoPlaintext(pub [u8; MEMO_LEN_BYTES]);

impl Default for MemoPlaintext {
    fn default() -> MemoPlaintext {
        MemoPlaintext([0u8; MEMO_LEN_BYTES])
    }
}

impl MemoPlaintext {
    // Encrypt a memo, returning its ciphertext.
    pub fn encrypt(
        &self,
        esk: &ka::Secret,
        transmission_key: &ka::Public,
        diversified_generator: &decaf377::Element,
    ) -> MemoCiphertext {
        let epk = esk.diversified_public(diversified_generator);
        let shared_secret = esk
            .key_agreement_with(&transmission_key)
            .expect("key agreement success");

        let mut kdf_input = Vec::new();
        kdf_input.copy_from_slice(&shared_secret.0);
        kdf_input.copy_from_slice(&epk.0);
        let kdf_output = blake2b_simd::blake2b(&kdf_input);
        let key = Key::from_slice(kdf_output.as_bytes());

        let cipher = ChaCha20Poly1305::new(key);
        let nonce = Nonce::from_slice(&[0u8; 12]);

        let encryption_result = cipher
            .encrypt(nonce, self.0.as_ref())
            .expect("encryption failure!");

        let ciphertext: [u8; MEMO_CIPHERTEXT_LEN_BYTES] = encryption_result
            .try_into()
            .expect("can fit in ciphertext len");

        MemoCiphertext(ciphertext)
    }
}

#[derive(Clone)]
pub struct MemoCiphertext(pub [u8; MEMO_CIPHERTEXT_LEN_BYTES]);
