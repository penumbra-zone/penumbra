use anyhow::anyhow;
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
    ) -> Result<MemoCiphertext, anyhow::Error> {
        let epk = esk.diversified_public(diversified_generator);
        let shared_secret = esk
            .key_agreement_with(&transmission_key)
            .map_err(|_| anyhow!("could not perform key agreement"))?;

        // Use Blake2b-256 to derive encryption key.
        let mut kdf_params = blake2b_simd::Params::new();
        kdf_params.hash_length(32);
        let mut kdf = kdf_params.to_state();
        kdf.update(&shared_secret.0);
        kdf.update(&epk.0);
        let kdf_output = kdf.finalize();
        let key = Key::from_slice(kdf_output.as_bytes());

        let cipher = ChaCha20Poly1305::new(key);
        let nonce = Nonce::from_slice(&[0u8; 12]);

        let encryption_result = cipher
            .encrypt(nonce, self.0.as_ref())
            .map_err(|_| anyhow!("encryption error!"))?;

        let ciphertext: [u8; MEMO_CIPHERTEXT_LEN_BYTES] = encryption_result
            .try_into()
            .map_err(|_| anyhow!("memo encryption result does not fit in ciphertext len"))?;

        Ok(MemoCiphertext(ciphertext))
    }
}

#[derive(Clone)]
pub struct MemoCiphertext(pub [u8; MEMO_CIPHERTEXT_LEN_BYTES]);

#[cfg(test)]
mod tests {
    use super::*;

    use crate::keys::SpendKey;
    use rand_core::OsRng;

    #[test]
    fn test_memo_encryption_and_decryption() {
        let mut rng = OsRng;

        let sk = SpendKey::generate(&mut rng);
        let fvk = sk.full_viewing_key();
        let ivk = fvk.incoming();
        let (dest, _dtk_d) = ivk.payment_address(0u64.into());

        let mut memo_bytes = [0u8; MEMO_LEN_BYTES];
        memo_bytes[0..2].copy_from_slice(b"Hi");

        let esk = ka::Secret::new(&mut rng);

        let memo = MemoPlaintext(memo_bytes);

        let _ciphertext = memo.encrypt(&esk, dest.transmission_key(), dest.diversified_generator());

        // TODO: Decryption
    }
}
