use std::array::TryFromSliceError;

use crate::{error::Error, Precision};

/// A clue that allows probabilistic message detection.
#[derive(Debug, Clone)]
pub struct Clue(pub(crate) [u8; 68]);

impl Clue {
    /// The bits of precision for this `Clue`, if valid.
    pub fn precision(&self) -> Result<Precision, Error> {
        self.0[64].try_into()
    }
}

impl From<Clue> for Vec<u8> {
    fn from(value: Clue) -> Self {
        value.0.into()
    }
}

impl TryFrom<&[u8]> for Clue {
    type Error = TryFromSliceError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Ok(Self(value.try_into()?))
    }
}
