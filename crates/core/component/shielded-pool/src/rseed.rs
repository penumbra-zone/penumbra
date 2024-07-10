use decaf377::{Fq, Fr};
use decaf377_ka as ka;
use penumbra_keys::prf;
use rand::{CryptoRng, RngCore};

/// The rseed is a uniformly random 32-byte sequence included in the note plaintext.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rseed(pub [u8; 32]);

impl Rseed {
    /// Generate a new rseed from a random source.
    pub fn generate<R: RngCore + CryptoRng>(rng: &mut R) -> Self {
        let mut bytes = [0u8; 32];
        rng.fill_bytes(&mut bytes);
        Self(bytes)
    }

    /// Derive the ephemeral secret key from the rseed.
    pub fn derive_esk(&self) -> ka::Secret {
        let hash_result = prf::expand(b"Penumbra_DeriEsk", &self.0, &[4u8]);
        ka::Secret::new_from_field(Fr::from_le_bytes_mod_order(hash_result.as_bytes()))
    }

    /// Derive note commitment randomness from the rseed.
    pub fn derive_note_blinding(&self) -> Fq {
        let hash_result = prf::expand(b"Penumbra_DeriRcm", &self.0, &[5u8]);
        Fq::from_le_bytes_mod_order(hash_result.as_bytes())
    }

    pub fn to_bytes(&self) -> [u8; 32] {
        self.0
    }
}
