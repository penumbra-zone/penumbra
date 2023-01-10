use crate::{ka, Fq};

/// The rseed is a uniformly random 32-byte sequence included in the note plaintext.
#[derive(Clone, Copy, Debug)]
pub struct Rseed(pub [u8; 32]);

impl Rseed {
    /// Generate a new rseed from a random source.
    pub fn generate(&self) -> Self {
        todo!()
    }

    /// Derive the ephemeral secret key from the rseed.
    pub fn derive_esk(&self) -> ka::Secret {
        todo!()
    }

    /// Derive note commitment randomness from the rseed.
    pub fn derive_note_blinding(&self) -> Fq {
        todo!()
    }
}
