use anyhow::anyhow;
use chacha20poly1305::{
    aead::{Aead, NewAead},
    ChaCha20Poly1305, Key, Nonce,
};
use once_cell::sync::Lazy;
use std::convert::TryInto;

use crate::{ka, keys::IncomingViewingKey, note::derive_symmetric_key, Address};

pub const MEMO_CIPHERTEXT_LEN_BYTES: usize = 528;

// This is the `MEMO_CIPHERTEXT_LEN_BYTES` - MAC size (16 bytes).
pub const MEMO_LEN_BYTES: usize = 512;

/// The nonce used for memo encryption.
pub static MEMO_ENCRYPTION_NONCE: Lazy<[u8; 12]> = Lazy::new(|| {
    let nonce_bytes = 1u128.to_le_bytes();
    nonce_bytes[0..12].try_into().expect("nonce fits in array")
});

// The memo is stored separately from the `Note`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MemoPlaintext(pub [u8; MEMO_LEN_BYTES]);

impl Default for MemoPlaintext {
    fn default() -> MemoPlaintext {
        MemoPlaintext([0u8; MEMO_LEN_BYTES])
    }
}

impl MemoPlaintext {
    /// Encrypt a memo, returning its ciphertext.
    pub fn encrypt(&self, esk: &ka::Secret, address: &Address) -> MemoCiphertext {
        let epk = esk.diversified_public(address.diversified_generator());
        let shared_secret = esk
            .key_agreement_with(&address.transmission_key())
            .expect("key agreement succeeds");

        let key = derive_symmetric_key(&shared_secret, &epk);
        let cipher = ChaCha20Poly1305::new(Key::from_slice(key.as_bytes()));
        let nonce = Nonce::from_slice(&*MEMO_ENCRYPTION_NONCE);

        let encryption_result = cipher
            .encrypt(nonce, self.0.as_ref())
            .expect("memo encryption succeeded");

        let ciphertext: [u8; MEMO_CIPHERTEXT_LEN_BYTES] = encryption_result
            .try_into()
            .expect("memo encryption result fits in ciphertext len");

        MemoCiphertext(ciphertext)
    }

    /// Decrypt a `MemoCiphertext` to generate a plaintext `Memo`.
    pub fn decrypt(
        ciphertext: MemoCiphertext,
        ivk: &IncomingViewingKey,
        epk: &ka::Public,
    ) -> Result<MemoPlaintext, anyhow::Error> {
        let shared_secret = ivk
            .key_agreement_with(epk)
            .map_err(|_| anyhow!("could not perform key agreement"))?;

        let key = derive_symmetric_key(&shared_secret, &epk);
        let cipher = ChaCha20Poly1305::new(Key::from_slice(key.as_bytes()));
        let nonce = Nonce::from_slice(&*MEMO_ENCRYPTION_NONCE);
        let plaintext = cipher
            .decrypt(nonce, ciphertext.0.as_ref())
            .map_err(|_| anyhow!("decryption error"))?;

        let plaintext_bytes: [u8; MEMO_LEN_BYTES] = plaintext
            .try_into()
            .map_err(|_| anyhow!("could not fit plaintext into memo size"))?;

        Ok(MemoPlaintext(plaintext_bytes))
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

        let ciphertext = memo.encrypt(&esk, &dest);

        let epk = esk.diversified_public(&dest.diversified_generator());
        let plaintext = MemoPlaintext::decrypt(ciphertext, ivk, &epk).expect("can decrypt memo");

        assert_eq!(plaintext, memo);
    }
}
