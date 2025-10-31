use decaf377::{Fq, Fr};
use decaf377_ka as ka;
use once_cell::sync::Lazy;
use penumbra_sdk_keys::prf;
#[cfg(feature = "rand")]
use rand::{CryptoRng, RngCore};

pub static RCM_DOMAIN_SEP: Lazy<Fq> =
    Lazy::new(|| Fq::from_le_bytes_mod_order(b"cycles.derive.pool.rcm"));

/// The rseed is a uniformly random 32-byte sequence included in the note plaintext.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rseed(pub Fq);

impl Rseed {
    /// Generate a new rseed from a random source.
    #[cfg(feature = "rand")]
    pub fn generate<R: RngCore + CryptoRng>(rng: &mut R) -> Self {
        let mut bytes = [0u8; 32];
        rng.fill_bytes(&mut bytes);
        Self::from(bytes)
    }

    /// Derive the ephemeral secret key from the rseed.
    pub fn derive_esk(&self) -> ka::Secret {
        let hash_result = prf::expand(b"Penumbra_DeriEsk", &self.0.to_bytes(), &[4u8]);
        ka::Secret::new_from_field(Fr::from_le_bytes_mod_order(hash_result.as_bytes()))
    }

    /// Derive note commitment randomness from the rseed.
    pub fn derive_note_blinding(&self) -> Fq {
        poseidon377::hash_1(&RCM_DOMAIN_SEP, self.0)
    }

    pub fn to_bytes(&self) -> [u8; 32] {
        self.0.to_bytes()
    }
}

impl From<[u8; 32]> for Rseed {
    fn from(bytes: [u8; 32]) -> Self {
        Self(Fq::from_le_bytes_mod_order(&bytes))
    }
}

impl From<&[u8; 32]> for Rseed {
    fn from(bytes: &[u8; 32]) -> Self {
        Self(Fq::from_le_bytes_mod_order(bytes))
    }
}

impl From<&[u8]> for Rseed {
    fn from(bytes: &[u8]) -> Self {
        Self(Fq::from_le_bytes_mod_order(bytes))
    }
}
