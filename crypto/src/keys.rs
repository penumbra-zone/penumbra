use once_cell::sync::Lazy;

use ark_ff::PrimeField;
use decaf377;

use crate::Fq;

pub struct OutgoingViewingKey {}

pub struct Diversifier(pub [u8; 11]);

/// The domain separator used to generate diversified generators.
static DIVERSIFY_GENERATOR_DOMAIN_SEP: Lazy<Fq> = Lazy::new(|| {
    Fq::from_le_bytes_mod_order(blake2b_simd::blake2b(b"penumbra.diversifier.generator").as_bytes())
});

impl Diversifier {
    pub fn diversified_generator(&self) -> decaf377::Element {
        use crate::poseidon_hash::hash_1;
        let hash = hash_1(
            &DIVERSIFY_GENERATOR_DOMAIN_SEP,
            Fq::from_le_bytes_mod_order(&self.0[..]),
        );
        decaf377::Element::map_to_group_cdh(&hash)
    }
}

pub struct TransmissionKey(pub decaf377::Element);

pub struct EphemeralPublicKey(pub decaf377::Element);

pub struct AuthorizationKey {}
