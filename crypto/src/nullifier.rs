use std::convert::{TryFrom, TryInto};

use ark_ff::Zero;
use decaf377::FieldExt;
use thiserror;

use crate::Fq;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Invalid nullifier")]
    InvalidNullifier,
}

pub struct Nullifier(pub Fq);

impl Nullifier {
    pub fn new() -> Self {
        // TODO! Zero is just a dummy value.
        Nullifier(Fq::zero())
    }
}

impl Into<[u8; 32]> for Nullifier {
    fn into(self) -> [u8; 32] {
        self.0.to_bytes()
    }
}

impl TryFrom<&[u8]> for Nullifier {
    type Error = Error;

    fn try_from(slice: &[u8]) -> Result<Nullifier, Self::Error> {
        let bytes: [u8; 32] = slice[..].try_into().map_err(|_| Error::InvalidNullifier)?;
        let inner = Fq::from_bytes(bytes).map_err(|_| Error::InvalidNullifier)?;
        Ok(Nullifier(inner))
    }
}
