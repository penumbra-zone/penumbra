use core::fmt;

use crate::Error;

/// The maximum detection precision, chosen so that the message bits fit in 3 bytes.
pub(crate) const MAX_PRECISION: u8 = 24;

/// Represents the precision governing the false positive rate of detection.
///
/// This is usually measured in bits, where a precision of `n` bits yields false
/// positives with a rate of `2^-n`.
///
/// This type implements `TryFrom` for `u8`, `u32`, `u64`, and `i32`, which has the behavior of considering
/// the value as a number of bits, and converting if this number isn't too large.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Precision(u8);

impl Precision {
    pub const MAX: Precision = Precision(MAX_PRECISION);

    pub fn new(precision_bits: u8) -> Result<Self, Error> {
        if precision_bits > MAX_PRECISION {
            return Err(Error::PrecisionTooLarge(precision_bits.into()));
        }
        Ok(Self(precision_bits))
    }

    pub fn bits(&self) -> u8 {
        self.0
    }
}

impl Default for Precision {
    fn default() -> Self {
        Self(0)
    }
}

impl fmt::Display for Precision {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<u8> for Precision {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<u32> for Precision {
    type Error = Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        u8::try_from(value)
            .map_err(|_| Error::PrecisionTooLarge(value.into()))?
            .try_into()
    }
}

impl TryFrom<u64> for Precision {
    type Error = Error;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        u8::try_from(value)
            .map_err(|_| Error::PrecisionTooLarge(value))?
            .try_into()
    }
}

impl TryFrom<i32> for Precision {
    type Error = Error;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        u8::try_from(value)
            .map_err(|_| Error::PrecisionTooLarge(value as u64))?
            .try_into()
    }
}
