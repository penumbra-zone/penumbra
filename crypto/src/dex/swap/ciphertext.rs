use anyhow::Result;
use chacha20poly1305::{
    aead::{Aead, NewAead},
    ChaCha20Poly1305, Key, Nonce,
};

use crate::ka;

use super::{SwapPlaintext, SWAP_CIPHERTEXT_BYTES, SWAP_ENCRYPTION_NONCE, SWAP_LEN_BYTES};

#[derive(Debug, Clone)]
pub struct SwapCiphertext(pub [u8; SWAP_CIPHERTEXT_BYTES]);

impl SwapCiphertext {
    pub fn decrypt(
        &self,
        esk: &ka::Secret,
        transmission_key: ka::Public,
        diversified_basepoint: decaf377::Element,
    ) -> Result<SwapPlaintext> {
        let shared_secret = esk
            .key_agreement_with(&transmission_key)
            .expect("key agreement succeeds");
        let epk = esk.diversified_public(&diversified_basepoint);
        let key = SwapPlaintext::derive_symmetric_key(&shared_secret, &epk);
        let cipher = ChaCha20Poly1305::new(Key::from_slice(key.as_bytes()));
        let nonce = Nonce::from_slice(&*SWAP_ENCRYPTION_NONCE);

        let swap_ciphertext = self.0;
        let decryption_result = cipher
            .decrypt(nonce, swap_ciphertext.as_ref())
            .map_err(|_| anyhow::anyhow!("unable to decrypt swap ciphertext"))?;

        let plaintext: [u8; SWAP_LEN_BYTES] = decryption_result
            .try_into()
            .map_err(|_| anyhow::anyhow!("swap decryption result did not fit in plaintext len"))?;

        plaintext.try_into().map_err(|_| {
            anyhow::anyhow!("unable to convert swap plaintext bytes into SwapPlaintext")
        })
    }
}

impl TryFrom<[u8; SWAP_CIPHERTEXT_BYTES]> for SwapCiphertext {
    type Error = anyhow::Error;

    fn try_from(bytes: [u8; SWAP_CIPHERTEXT_BYTES]) -> Result<SwapCiphertext, Self::Error> {
        Ok(SwapCiphertext(bytes))
    }
}

impl TryFrom<&[u8]> for SwapCiphertext {
    type Error = anyhow::Error;

    fn try_from(slice: &[u8]) -> Result<SwapCiphertext, Self::Error> {
        Ok(SwapCiphertext(slice[..].try_into()?))
    }
}
