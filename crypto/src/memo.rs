use std::convert::{TryFrom, TryInto};

use anyhow::anyhow;
use chacha20poly1305::{
    aead::{Aead, NewAead},
    ChaCha20Poly1305, Key, Nonce,
};
use once_cell::sync::Lazy;

use crate::{
    ka,
    keys::{IncomingViewingKey, OutgoingViewingKey},
    note,
    note::{derive_symmetric_key, OVK_WRAPPED_LEN_BYTES},
    value, Address, Note,
};

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

impl TryFrom<&[u8]> for MemoPlaintext {
    type Error = anyhow::Error;

    fn try_from(input: &[u8]) -> Result<MemoPlaintext, Self::Error> {
        if input.len() > MEMO_LEN_BYTES {
            return Err(anyhow::anyhow!("provided memo exceeds maximum memo size"));
        }
        let mut mp = [0u8; MEMO_LEN_BYTES];
        mp[..input.len()].copy_from_slice(input);

        Ok(MemoPlaintext(mp))
    }
}

impl MemoPlaintext {
    /// Encrypt a memo, returning its ciphertext.
    pub fn encrypt(&self, esk: &ka::Secret, address: &Address) -> MemoCiphertext {
        let epk = esk.diversified_public(address.diversified_generator());
        let shared_secret = esk
            .key_agreement_with(address.transmission_key())
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

        let key = derive_symmetric_key(&shared_secret, epk);
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

    /// Decrypt a `MemoCiphertext` using the wrapped OVK to generate a plaintext `Memo`.
    pub fn decrypt_outgoing(
        ciphertext: MemoCiphertext,
        wrapped_ovk: [u8; OVK_WRAPPED_LEN_BYTES],
        cm: note::Commitment,
        cv: value::Commitment,
        ovk: &OutgoingViewingKey,
        epk: &ka::Public,
    ) -> Result<MemoPlaintext, anyhow::Error> {
        let (esk, transmission_key) = Note::decrypt_key(wrapped_ovk, cm, cv, ovk, epk)
            .map_err(|_| anyhow!("key decryption error"))?;
        let shared_secret = esk
            .key_agreement_with(&transmission_key)
            .map_err(|_| anyhow!("could not perform key agreement"))?;
        let key = derive_symmetric_key(&shared_secret, epk);
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

#[derive(Clone, Debug)]
pub struct MemoCiphertext(pub [u8; MEMO_CIPHERTEXT_LEN_BYTES]);

#[cfg(test)]
mod tests {
    use ark_ff::UniformRand;
    use rand_core::OsRng;

    use super::*;
    use crate::{
        asset,
        keys::{SeedPhrase, SpendKey},
        Value,
    };
    use decaf377::Fr;

    #[test]
    fn test_memo_encryption_and_decryption() {
        let mut rng = OsRng;

        let seed_phrase = SeedPhrase::generate(&mut rng);
        let sk = SpendKey::from_seed_phrase(seed_phrase, 0);
        let fvk = sk.full_viewing_key();
        let ivk = fvk.incoming();
        let (dest, _dtk_d) = ivk.payment_address(0u64.into());

        let mut memo_bytes = [0u8; MEMO_LEN_BYTES];
        memo_bytes[0..2].copy_from_slice(b"Hi");

        let esk = ka::Secret::new(&mut rng);

        let memo = MemoPlaintext(memo_bytes);

        let ciphertext = memo.encrypt(&esk, &dest);

        let epk = esk.diversified_public(dest.diversified_generator());
        let plaintext = MemoPlaintext::decrypt(ciphertext, ivk, &epk).expect("can decrypt memo");

        assert_eq!(plaintext, memo);
    }

    #[test]
    fn test_memo_encryption_and_sender_decryption() {
        let mut rng = OsRng;

        let seed_phrase = SeedPhrase::generate(&mut rng);
        let sk = SpendKey::from_seed_phrase(seed_phrase, 0);
        let fvk = sk.full_viewing_key();
        let ivk = fvk.incoming();
        let ovk = fvk.outgoing();
        let (dest, _dtk_d) = ivk.payment_address(0u64.into());

        let mut memo_bytes = [0u8; MEMO_LEN_BYTES];
        memo_bytes[0..2].copy_from_slice(b"Hi");

        let esk = ka::Secret::new(&mut rng);

        let value = Value {
            amount: 10,
            asset_id: asset::REGISTRY.parse_denom("upenumbra").unwrap().id(),
        };
        let note = Note::generate(&mut rng, &dest, value);

        let memo = MemoPlaintext(memo_bytes);

        let value_blinding = Fr::rand(&mut rng);
        let cv = note.value().commit(value_blinding);

        let wrapped_ovk = note.encrypt_key(&esk, ovk, cv);
        let ciphertext = memo.encrypt(&esk, &dest);

        let epk = esk.diversified_public(dest.diversified_generator());
        let plaintext =
            MemoPlaintext::decrypt_outgoing(ciphertext, wrapped_ovk, note.commit(), cv, ovk, &epk)
                .expect("can decrypt memo");

        assert_eq!(plaintext, memo);
    }
}
