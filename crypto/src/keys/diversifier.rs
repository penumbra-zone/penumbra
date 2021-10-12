use ark_ff::PrimeField;
use decaf377;
use once_cell::sync::Lazy;
use rand_core::{CryptoRng, RngCore};

use crate::Fq;

pub const DIVERSIFIER_LEN_BYTES: usize = 11;

#[derive(Copy, Clone)]
pub struct Diversifier(pub [u8; DIVERSIFIER_LEN_BYTES]);

/// The domain separator used to generate diversified generators.
static DIVERSIFY_GENERATOR_DOMAIN_SEP: Lazy<Fq> = Lazy::new(|| {
    Fq::from_le_bytes_mod_order(blake2b_simd::blake2b(b"penumbra.diversifier.generator").as_bytes())
});

impl Diversifier {
    /// Generate a new random diversifier.
    pub fn generate<R: RngCore + CryptoRng>(mut rng: R) -> Self {
        // TODO: Switch to Poseidon based diversifier
        let mut diversifier = [0u8; DIVERSIFIER_LEN_BYTES];
        rng.fill_bytes(&mut diversifier);
        Diversifier(diversifier)
    }

    /// Generate the diversified basepoint.
    pub fn diversified_generator(&self) -> decaf377::Element {
        use crate::poseidon_hash::hash_1;
        let hash = hash_1(
            &DIVERSIFY_GENERATOR_DOMAIN_SEP,
            Fq::from_le_bytes_mod_order(&self.0[..]),
        );
        decaf377::Element::map_to_group_cdh(&hash)
    }
}
