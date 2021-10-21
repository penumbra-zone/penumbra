use aes::Aes256;
use ark_ff::PrimeField;
use fpe::ff1;
use once_cell::sync::Lazy;

use crate::Fq;

pub const DIVERSIFIER_LEN_BYTES: usize = 11;

#[derive(Copy, Clone)]
pub struct Diversifier(pub [u8; DIVERSIFIER_LEN_BYTES]);

/// The domain separator used to generate diversified generators.
static DIVERSIFY_GENERATOR_DOMAIN_SEP: Lazy<Fq> = Lazy::new(|| {
    Fq::from_le_bytes_mod_order(blake2b_simd::blake2b(b"penumbra.diversifier.generator").as_bytes())
});

impl Diversifier {
    /// Generate the diversified basepoint associated to this diversifier.
    pub fn diversified_generator(&self) -> decaf377::Element {
        decaf377::Element::map_to_group_cdh(&poseidon377::hash_1(
            &DIVERSIFY_GENERATOR_DOMAIN_SEP,
            Fq::from_le_bytes_mod_order(&self.0[..]),
        ))
    }
}

impl AsRef<[u8; 11]> for Diversifier {
    fn as_ref(&self) -> &[u8; 11] {
        &self.0
    }
}

#[derive(Clone)]
pub struct DiversifierKey(pub(super) [u8; 32]);

impl DiversifierKey {
    pub fn diversifier_for_index(&self, index: &DiversifierIndex) -> Diversifier {
        let enc_index = ff1::FF1::<Aes256>::new(&self.0, 2)
            .expect("radix 2 is in range")
            .encrypt(b"", &ff1::BinaryNumeralString::from_bytes_le(&index.0))
            .expect("binary string is the configured radix (2)");

        let mut diversifier_bytes = [0; 11];
        diversifier_bytes.copy_from_slice(&enc_index.to_bytes_le());
        Diversifier(diversifier_bytes)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct DiversifierIndex(pub [u8; 11]);

impl From<u8> for DiversifierIndex {
    fn from(x: u8) -> Self {
        let mut bytes = [0; 11];
        bytes[0] = x;
        Self(bytes)
    }
}

impl From<u16> for DiversifierIndex {
    fn from(x: u16) -> Self {
        let mut bytes = [0; 11];
        bytes[0..2].copy_from_slice(&x.to_le_bytes());
        Self(bytes)
    }
}

impl From<u32> for DiversifierIndex {
    fn from(x: u32) -> Self {
        let mut bytes = [0; 11];
        bytes[0..4].copy_from_slice(&x.to_le_bytes());
        Self(bytes)
    }
}

impl From<u64> for DiversifierIndex {
    fn from(x: u64) -> Self {
        let mut bytes = [0; 11];
        bytes[0..8].copy_from_slice(&x.to_le_bytes());
        Self(bytes)
    }
}
