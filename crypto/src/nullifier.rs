use std::convert::{TryFrom, TryInto};

use ark_ff::PrimeField;
use decaf377::FieldExt;
use derivative::Derivative;
use once_cell::sync::Lazy;

use crate::Fq;

#[derive(PartialEq, Eq, Derivative, Clone, Copy, Hash, PartialOrd, Ord)]
#[derivative(Debug)]
pub struct Nullifier(#[derivative(Debug(format_with = "crate::fmt_fq"))] pub Fq);

/// The domain separator used to derive nullifiers.
pub static NULLIFIER_DOMAIN_SEP: Lazy<Fq> = Lazy::new(|| {
    Fq::from_le_bytes_mod_order(blake2b_simd::blake2b(b"penumbra.nullifier").as_bytes())
});

impl Nullifier {
    pub fn to_bytes(&self) -> [u8; 32] {
        self.0.to_bytes()
    }
}

impl From<Nullifier> for [u8; 32] {
    fn from(nullifier: Nullifier) -> [u8; 32] {
        nullifier.0.to_bytes()
    }
}

impl TryFrom<&[u8]> for Nullifier {
    type Error = anyhow::Error;

    fn try_from(slice: &[u8]) -> Result<Nullifier, Self::Error> {
        let bytes: [u8; 32] = slice[..].try_into()?;
        let inner = Fq::from_bytes(bytes)?;
        Ok(Nullifier(inner))
    }
}

impl TryFrom<Vec<u8>> for Nullifier {
    type Error = anyhow::Error;

    fn try_from(vec: Vec<u8>) -> Result<Nullifier, Self::Error> {
        Self::try_from(&vec[..])
    }
}
