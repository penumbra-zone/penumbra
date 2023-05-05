use std::fmt::{Debug, Display};

mod div;
mod from;
mod ops;

#[cfg(test)]
mod tests;

use ethnum::U256;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct U128x128(U256);

// binary repr of 2^128
const BASE: u64 = 0x47f0000000000000;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("attempted to convert non-integral value {value:?} to an integer")]
    NonIntegral { value: U128x128 },
    #[error("attempted to decode a slice of the wrong length {0}, expected 32")]
    SliceLength(usize),
}

impl Default for U128x128 {
    fn default() -> Self {
        Self::from(0u64)
    }
}

impl Debug for U128x128 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (integral, fractional) = self.0.into_words();
        f.debug_struct("U128x128")
            .field("integral", &integral)
            .field("fractional", &fractional)
            .finish()
    }
}

impl Display for U128x128 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", f64::from(*self))
    }
}

impl U128x128 {
    /// Encode this number as a 32-byte array.
    ///
    /// The encoding has the property that it preserves ordering, i.e., if `x <=
    /// y` (with numeric ordering) then `x.to_bytes() <= y.to_bytes()` (with the
    /// lex ordering on byte strings).
    pub fn to_bytes(self) -> [u8; 32] {
        // The U256 type has really weird endianness handling -- e.g., it reverses
        // the endianness of the inner u128s (??) -- so just do it manually.
        let mut bytes = [0u8; 32];
        let (hi, lo) = self.0.into_words();
        bytes[0..16].copy_from_slice(&hi.to_be_bytes());
        bytes[16..32].copy_from_slice(&lo.to_be_bytes());
        bytes
    }

    /// Decode this number from a 32-byte array.
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        // See above.
        let hi = u128::from_be_bytes(bytes[0..16].try_into().unwrap());
        let lo = u128::from_be_bytes(bytes[16..32].try_into().unwrap());
        Self(U256::from_words(hi, lo))
    }

    pub fn ratio<T: Into<Self>>(numerator: T, denominator: T) -> Option<Self> {
        numerator.into() / denominator.into()
    }

    /// Checks whether this number is integral, i.e., whether it has no fractional part.
    pub fn is_integral(&self) -> bool {
        let fractional_word = self.0.into_words().1;
        fractional_word == 0
    }

    /// Rounds the number down to the nearest integer.
    pub fn round_down(self) -> Self {
        let integral_word = self.0.into_words().0;
        Self(U256::from_words(integral_word, 0u128))
    }

    /// Rounds the number up to the nearest integer.
    pub fn round_up(&self) -> Self {
        let (integral, fractional) = self.0.into_words();
        if fractional == 0 {
            *self
        } else {
            Self(U256::from_words(integral + 1, 0u128))
        }
    }

    /// Performs checked multiplication, returning `Some` if no overflow occurred.
    pub fn checked_mul(self, rhs: &Self) -> Option<Self> {
        let [x0, x1] = self.0 .0;
        let [y0, y1] = rhs.0 .0;
        let x0 = U256::from(x0);
        let x1 = U256::from(x1);
        let y0 = U256::from(y0);
        let y1 = U256::from(y1);

        // x = (x0*2^-128 + x1)*2^128
        // y = (y0*2^-128 + y1)*2^128
        // x*y        = (x0*y0*2^-256 + (x0*y1 + x1*y0)*2^-128 + x1*y1)*2^256
        // x*y*2^-128 = (x0*y0*2^-256 + (x0*y1 + x1*y0)*2^-128 + x1*y1)*2^128
        //               ^^^^^
        //               we drop the low 128 bits of this term as rounding error

        let x0y0 = x0 * y0; // cannot overflow, widening mul
        let x0y1 = x0 * y1; // cannot overflow, widening mul
        let x1y0 = x1 * y0; // cannot overflow, widening mul
        let x1y1 = x1 * y1; // cannot overflow, widening mul

        x1y1.checked_shl(128)
            .and_then(|acc| acc.checked_add(x0y1))
            .and_then(|acc| acc.checked_add(x1y0))
            .and_then(|acc| acc.checked_add(x0y0 >> 128))
            .map(U128x128)
    }

    /// Performs checked division, returning `Some` if no overflow occurred.
    pub fn checked_div(self, rhs: &Self) -> Option<Self> {
        if rhs.0 == U256::ZERO {
            return None;
        }

        // TEMP HACK: need to implement this properly
        let self_big = ibig::UBig::from_le_bytes(&self.0.to_le_bytes());
        let rhs_big = ibig::UBig::from_le_bytes(&rhs.0.to_le_bytes());
        // this is what we actually want to compute: 384-bit / 256-bit division.
        let q_big = (self_big << 128) / rhs_big;
        let q_big_bytes = q_big.to_le_bytes();
        let mut q_bytes = [0; 32];
        if q_big_bytes.len() > 32 {
            return None;
        } else {
            q_bytes[..q_big_bytes.len()].copy_from_slice(&q_big_bytes);
        }
        let q = U256::from_le_bytes(q_bytes);

        Some(U128x128(q))
    }

    /// Performs checked addition, returning `Some` if no overflow occurred.
    pub fn checked_add(self, rhs: &Self) -> Option<Self> {
        self.0.checked_add(rhs.0).map(U128x128)
    }

    /// Performs checked subtraction, returning `Some` if no underflow occurred.
    pub fn checked_sub(self, rhs: &Self) -> Option<Self> {
        self.0.checked_sub(rhs.0).map(U128x128)
    }

    /// Saturating integer subtraction. Computes self - rhs, saturating at the numeric bounds instead of overflowing.
    pub fn saturating_sub(self, rhs: &Self) -> Self {
        U128x128(self.0.saturating_sub(rhs.0))
    }
}

impl TryFrom<f64> for U128x128 {
    type Error = anyhow::Error;

    fn try_from(value: f64) -> anyhow::Result<U128x128> {
        if value < 0.0 {
            Err(anyhow::anyhow!(
                "U128x128 cannot represent negative numbers"
            ))
        } else if value.is_infinite() {
            Err(anyhow::anyhow!("U128x128 cannot represent infinite values"))
        } else if value.is_nan() {
            Err(anyhow::anyhow!("U128x128 cannot represent NaN values"))
        } else {
            let integral = value as u128;
            let fractional = value.fract();

            // Convert the fractional part to an unsigned 128-bit integer.
            // 1. Multiply the fractional part by 2^128 to move the significant bits to the left.
            // 2. Round down the result to the nearest integer.
            // 3. Convert the rounded result to an unsigned 128-bit integer.
            let fractional_as_u128 =
                (fractional * f64::from_bits(0x47f0000000000000)).trunc() as u128;
            let combined = U256::from_words(integral, fractional_as_u128);
            Ok(U128x128(combined))
        }
    }
}
