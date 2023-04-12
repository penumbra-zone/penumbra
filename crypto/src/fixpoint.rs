use std::fmt::{Debug, Display};

mod div;
mod from;
mod ops;

#[cfg(test)]
mod tests;

use ark_r1cs_std::prelude::*;
use ark_relations::r1cs::{ConstraintSystemRef, SynthesisError};

use decaf377::{r1cs::FqVar, FieldExt, Fq};
use ethnum::U256;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct U128x128(U256);

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("overflow")]
    Overflow,
    #[error("underflow")]
    Underflow,
    #[error("division by zero")]
    DivisionByZero,
    #[error("attempted to convert invalid f64: {value:?} to a U128x128")]
    InvalidFloat64 { value: f64 },
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

    pub fn ratio<T: Into<Self>>(numerator: T, denominator: T) -> Result<Self, Error> {
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
    pub fn checked_mul(self, rhs: &Self) -> Result<Self, Error> {
        // It's important to use `into_words` because the `U256` type has an
        // unsafe API that makes the limb ordering dependent on the host
        // endianness.
        let (x1, x0) = self.0.into_words();
        let (y1, y0) = rhs.0.into_words();
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

        let (x1y1_hi, _x1y1_lo) = x1y1.into_words();
        if x1y1_hi != 0 {
            return Err(Error::Overflow);
        }

        x1y1.checked_shl(128)
            .and_then(|acc| acc.checked_add(x0y1))
            .and_then(|acc| acc.checked_add(x1y0))
            .and_then(|acc| acc.checked_add(x0y0 >> 128))
            .map(U128x128)
            .ok_or(Error::Overflow)
    }

    /// Performs checked division, returning `Some` if no overflow occurred.
    pub fn checked_div(self, rhs: &Self) -> Result<Self, Error> {
        if rhs.0 == U256::ZERO {
            return Err(Error::DivisionByZero);
        }

        // TEMP HACK: need to implement this properly
        let self_big = ibig::UBig::from_le_bytes(&self.0.to_le_bytes());
        let rhs_big = ibig::UBig::from_le_bytes(&rhs.0.to_le_bytes());
        // this is what we actually want to compute: 384-bit / 256-bit division.
        let q_big = (self_big << 128) / rhs_big;
        let q_big_bytes = q_big.to_le_bytes();
        let mut q_bytes = [0; 32];
        if q_big_bytes.len() > 32 {
            return Err(Error::Overflow);
        } else {
            q_bytes[..q_big_bytes.len()].copy_from_slice(&q_big_bytes);
        }
        let q = U256::from_le_bytes(q_bytes);

        Ok(U128x128(q))
    }

    /// Performs checked addition, returning `Some` if no overflow occurred.
    pub fn checked_add(self, rhs: &Self) -> Result<Self, Error> {
        self.0
            .checked_add(rhs.0)
            .map(U128x128)
            .ok_or(Error::Overflow)
    }

    /// Performs checked subtraction, returning `Some` if no underflow occurred.
    pub fn checked_sub(self, rhs: &Self) -> Result<Self, Error> {
        self.0
            .checked_sub(rhs.0)
            .map(U128x128)
            .ok_or(Error::Underflow)
    }

    /// Saturating integer subtraction. Computes self - rhs, saturating at the numeric bounds instead of overflowing.
    pub fn saturating_sub(self, rhs: &Self) -> Self {
        U128x128(self.0.saturating_sub(rhs.0))
    }
}

pub struct U128x128Var {
    pub lo_var: FqVar,
    pub hi_var: FqVar,
}

impl AllocVar<U128x128, Fq> for U128x128Var {
    fn new_variable<T: std::borrow::Borrow<U128x128>>(
        cs: impl Into<ark_relations::r1cs::Namespace<Fq>>,
        f: impl FnOnce() -> Result<T, SynthesisError>,
        mode: ark_r1cs_std::prelude::AllocationMode,
    ) -> Result<Self, SynthesisError> {
        let ns = cs.into();
        let cs = ns.cs();
        let inner: U128x128 = *f()?.borrow();
        let (lo, hi) = inner.0.into_words();
        let mut lo_bytes = [0u8; 32];
        lo_bytes.copy_from_slice(&lo.to_le_bytes()[..]);
        let lo_fq = Fq::from_bytes(lo_bytes).expect("can form field element from bytes");
        let lo_var = FqVar::new_variable(cs.clone(), || Ok(lo_fq), mode)?;

        let mut hi_bytes = [0u8; 32];
        hi_bytes.copy_from_slice(&hi.to_le_bytes()[..]);
        let hi_fq = Fq::from_bytes(hi_bytes).expect("can form field element from bytes");
        let hi_var = FqVar::new_variable(cs, || Ok(hi_fq), mode)?;
        Ok(Self { lo_var, hi_var })
    }
}

impl R1CSVar<Fq> for U128x128Var {
    type Value = U128x128;

    fn cs(&self) -> ark_relations::r1cs::ConstraintSystemRef<Fq> {
        self.lo_var.cs()
    }

    fn value(&self) -> Result<Self::Value, ark_relations::r1cs::SynthesisError> {
        let lo = self.lo_var.value()?;
        let lo_bytes = lo.to_bytes();
        let hi = self.hi_var.value()?;
        let hi_bytes = hi.to_bytes();

        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&lo_bytes[..]);
        bytes.copy_from_slice(&hi_bytes[..]);

        Ok(Self::Value::from_bytes(bytes))
    }
}

impl U128x128Var {
    pub fn checked_add(
        self,
        rhs: &Self,
        cs: ConstraintSystemRef<Fq>,
    ) -> Result<U128x128Var, SynthesisError> {
        // Result of checked addition can be Some or None
        let default = U128x128::from_bytes([0u8; 32]);
        let lhs = self.value().unwrap_or(default);
        let rhs = rhs.value().unwrap_or(default);
        // Result can be Some or None
        let result = (lhs + rhs).expect("result does not overflow");

        // Missing constraints
        U128x128Var::new_variable(cs, || Ok(result), AllocationMode::Witness)
    }

    //pub fn checked_sub()
}
