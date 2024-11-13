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

impl EncryptedBackref {
    pub fn decrypt(&self, brk: &BackreferenceKey, nullifier: &Nullifier) -> Result<Backref> {
        // We might have a 0-length encrypted backref, which
        // is treated as a valid value and means that the note has no backref.
        if self.bytes.is_empty() {
            let zero_commitment = tct::StateCommitment::try_from([0u8; 32])?;
            return Ok(Backref::new(zero_commitment));
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

        Backref::try_from(note_commitment_bytes).map_err(|_| anyhow::anyhow!("decryption error"))
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
