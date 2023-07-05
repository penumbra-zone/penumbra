use anyhow::Result;

use penumbra_crypto::{note, PayloadKey};
use penumbra_keys::keys::OutgoingViewingKey;

use super::{SwapPlaintext, SWAP_CIPHERTEXT_BYTES, SWAP_LEN_BYTES};

#[derive(Debug, Clone)]
pub struct SwapCiphertext(pub [u8; SWAP_CIPHERTEXT_BYTES]);

impl SwapCiphertext {
    pub fn decrypt(
        &self,
        ovk: &OutgoingViewingKey,
        commitment: note::StateCommitment,
    ) -> Result<SwapPlaintext> {
        let payload_key = PayloadKey::derive_swap(ovk, commitment);
        self.decrypt_with_payload_key(&payload_key, commitment)
    }

    pub fn decrypt_with_payload_key(
        &self,
        payload_key: &PayloadKey,
        commitment: note::StateCommitment,
    ) -> Result<SwapPlaintext> {
        let swap_ciphertext = self.0;
        let decryption_result = payload_key
            .decrypt_swap(swap_ciphertext.to_vec(), commitment)
            .map_err(|_| anyhow::anyhow!("unable to decrypt swap ciphertext"))?;

        // TODO: encapsulate plaintext encoding by making this a
        // pub(super) parse_decryption method on SwapPlaintext
        // and removing the TryFrom impls
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
