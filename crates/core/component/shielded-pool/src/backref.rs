use anyhow::Result;
use chacha20poly1305::{
    aead::{Aead, NewAead},
    ChaCha20Poly1305, Nonce,
};

use penumbra_keys::BackreferenceKey;
use penumbra_sct::Nullifier;
use penumbra_tct as tct;

pub const ENCRYPTED_BACKREF_LEN: usize = 48;

pub struct Backref {
    pub note_commitment: tct::StateCommitment,
}

#[derive(Clone, Debug)]
pub struct EncryptedBackref {
    pub bytes: Vec<u8>,
}

impl Backref {
    pub fn encrypt(
        &self,
        brk: &BackreferenceKey,
        nullifier: &Nullifier,
    ) -> Result<EncryptedBackref> {
        let cipher = ChaCha20Poly1305::new(&brk.0);

        // Nonce is the first 12 bytes of the nullifier
        let nonce_bytes = &nullifier.to_bytes()[..12];
        let nonce = Nonce::from_slice(&nonce_bytes);

        let plaintext = self.note_commitment.0.to_bytes();

        let ciphertext = cipher
            .encrypt(nonce, plaintext.as_ref())
            .map_err(|_| anyhow::anyhow!("encryption error"))?;

        Ok(EncryptedBackref { bytes: ciphertext })
    }
}
