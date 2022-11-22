use std::convert::{TryFrom, TryInto};

use ark_ff::PrimeField;
use decaf377::FieldExt;
use once_cell::sync::Lazy;
use penumbra_proto::{core::crypto::v1alpha1 as pb, Protobuf};
use serde::{Deserialize, Serialize};

use crate::Fq;

#[derive(PartialEq, Eq, Clone, Copy, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "pb::Nullifier", into = "pb::Nullifier")]
pub struct Nullifier(pub Fq);

impl Nullifier {
    pub fn parse_hex(str: &str) -> Result<Nullifier, anyhow::Error> {
        let bytes = hex::decode(str)?;
        Nullifier::try_from(&bytes[..])
    }
}

impl Protobuf<pb::Nullifier> for Nullifier {}

impl From<Nullifier> for pb::Nullifier {
    fn from(n: Nullifier) -> Self {
        pb::Nullifier {
            inner: n.0.to_bytes().to_vec(),
        }
    }
}

impl TryFrom<pb::Nullifier> for Nullifier {
    type Error = anyhow::Error;
    fn try_from(n: pb::Nullifier) -> Result<Self, Self::Error> {
        n.inner.as_slice().try_into()
    }
}

/// The domain separator used to derive nullifiers.
pub static NULLIFIER_DOMAIN_SEP: Lazy<Fq> = Lazy::new(|| {
    Fq::from_le_bytes_mod_order(blake2b_simd::blake2b(b"penumbra.nullifier").as_bytes())
});

impl std::fmt::Display for Nullifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&hex::encode(self.to_bytes()))
    }
}

impl std::fmt::Debug for Nullifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Nullifier")
            .field(&hex::encode(self.to_bytes()))
            .finish()
    }
}

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
