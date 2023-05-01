use std::fmt::{Debug, Display};

mod div;
mod from;
mod ops;

#[cfg(test)]
mod tests;

use ark_ff::{BigInteger, Field, PrimeField};
use ark_r1cs_std::prelude::*;
use ark_r1cs_std::{bits::uint64::UInt64, ToConstraintFieldGadget};
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
    pub limbs: [UInt64<Fq>; 4],
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
        // let (lo, hi) = inner.0.into_words();

        // let mut lo_bytes = [0u8; 32];
        // lo_bytes.copy_from_slice(&lo.to_le_bytes()[..]);
        // let lo_fq = Fq::from_bytes(lo_bytes).expect("can form field element from bytes");
        // let lo_var = FqVar::new_variable(cs.clone(), || Ok(lo_fq), mode)?;

        // let mut hi_bytes = [0u8; 32];
        // hi_bytes.copy_from_slice(&hi.to_le_bytes()[..]);
        // let hi_fq = Fq::from_bytes(hi_bytes).expect("can form field element from bytes");
        // let hi_var = FqVar::new_variable(cs, || Ok(hi_fq), mode)?;
        // Ok(Self { lo_var, hi_var })
        todo!()
    }
}

impl R1CSVar<Fq> for U128x128Var {
    type Value = U128x128;

    fn cs(&self) -> ark_relations::r1cs::ConstraintSystemRef<Fq> {
        //self.lo_var.cs()
        todo!()
    }

    fn value(&self) -> Result<Self::Value, ark_relations::r1cs::SynthesisError> {
        // let lo = self.lo_var.value()?;
        // let lo_bytes = lo.to_bytes();
        // let hi = self.hi_var.value()?;
        // let hi_bytes = hi.to_bytes();

        // let mut bytes = [0u8; 32];
        // bytes.copy_from_slice(&lo_bytes[..]);
        // bytes.copy_from_slice(&hi_bytes[..]);

        // Ok(Self::Value::from_bytes(bytes))
        todo!()
    }
}

impl U128x128Var {
    pub fn checked_add(
        self,
        rhs: &Self,
        cs: ConstraintSystemRef<Fq>,
    ) -> Result<U128x128Var, SynthesisError> {
        todo!()
    }

    pub fn checked_sub(
        self,
        rhs: &Self,
        cs: ConstraintSystemRef<Fq>,
    ) -> Result<U128x128Var, SynthesisError> {
        todo!()
    }

    pub fn checked_mul(
        self,
        rhs: &Self,
        cs: ConstraintSystemRef<Fq>,
    ) -> Result<U128x128Var, SynthesisError> {
        // x = [x0, x1, x2, x3]
        // x = x0 + x1 * 2^64 + x2 * 2^128 + x3 * 2^192
        // y = [y0, y1, y2, y3]
        // y = y0 + y1 * 2^64 + y2 * 2^128 + y3 * 2^192
        let x0 = convert_uint64_to_fqvar(&self.limbs[0]);
        let x1 = convert_uint64_to_fqvar(&self.limbs[1]);
        let x2 = convert_uint64_to_fqvar(&self.limbs[2]);
        let x3 = convert_uint64_to_fqvar(&self.limbs[3]);

        let y0 = convert_uint64_to_fqvar(&rhs.limbs[0]);
        let y1 = convert_uint64_to_fqvar(&rhs.limbs[1]);
        let y2 = convert_uint64_to_fqvar(&rhs.limbs[2]);
        let y3 = convert_uint64_to_fqvar(&rhs.limbs[3]);

        // z = x * y
        // z = [z0, z1, z2, z3, z4, z5, z6, z7]
        // zi is 128 bits
        let z0 = x0.clone() * y0.clone();
        let z1 = x0.clone() * y1.clone() + x1.clone() * y0.clone();
        let z2 = x0.clone() * y2.clone() + x1.clone() * y1.clone() + x2.clone() * y0.clone();
        let z3 =
            x0 * y3.clone() + x1.clone() * y2.clone() + x2.clone() * y1.clone() + x3.clone() * y0;
        let z4 = x1 * y3.clone() + x2.clone() * y2.clone() + x3.clone() * y1;
        let z5 = x2 * y3.clone() + x3.clone() * y2;
        let z6 = x3 * y3;
        // z7 = 0
        // z = z0 + z1 * 2^64 + z2 * 2^128 + z3 * 2^192 + z4 * 2^256 + z5 * 2^320 + z6 * 2^384
        // z*2^-128 = z0*2^-128 + z1*2^-64 + z2 + z3*2^64 + z4*2^128 + z5*2^192 + z6*2^256
        //
        // w represents the limbs of the reduced result (z)
        // w = [w0, w1, w2, w3]
        // w0
        // wi are 64 bits like xi and yi
        //
        // ti represents some temporary value (indices not necessarily meaningful)
        let t0 = z0 + z1 * Fq::from(1u128 << 64);
        let t0_bits = fqvar_to_bits(t0, 193)?;
        // t0 fits in 193 bits
        // t0 we bit constrain to be 193 bits or less

        // t1 = (t0 >> 128) + z2
        let t1 = z2 + convert_le_bits_to_fqvar(&t0_bits[128..193]);
        // t1 fits in 129 bits
        let t1_bits = fqvar_to_bits(t1, 129)?;

        // w0 = t0 & 2^64 - 1
        let w0 = UInt64::from_bits_le(&t0_bits[0..64]);

        // t2 = (t1 >> 64) + z3
        let t2 = z3 + convert_le_bits_to_fqvar(&t1_bits[64..129]);
        // t2 fits in 129 bits
        let t2_bits = fqvar_to_bits(t2, 129)?;

        // w1 = t2 & 2^64 - 1
        let w1 = UInt64::from_bits_le(&t2_bits[0..64]);

        // t3 = (t2 >> 64) + z4
        let t3 = z4 + convert_le_bits_to_fqvar(&t2_bits[64..129]);
        // t3 fits in 129 bits
        let t3_bits = fqvar_to_bits(t3, 129)?;

        // w2 = t3 & 2^64 - 1
        let w2 = UInt64::from_bits_le(&t3_bits[0..64]);

        // t4 = (t3 >> 64) + z5
        let t4 = z5 + convert_le_bits_to_fqvar(&t3_bits[64..129]);
        // t4 fits in 64 bits
        let t4_bits = fqvar_to_bits(t4, 64)?;
        // If we didn't overflow, it will fit in 64 bits.

        // w3 = t4 & 2^64 - 1
        let w3 = UInt64::from_bits_le(&t4_bits[0..64]);

        // Overflow condition. Constrain z6 = 0.
        z6.enforce_equal(&FqVar::zero())?;

        // Internal rep: 4 Uint64
        Ok(U128x128Var {
            limbs: [w0, w1, w2, w3],
        })
    }

    pub fn checked_div(
        self,
        rhs: &Self,
        cs: ConstraintSystemRef<Fq>,
    ) -> Result<U128x128Var, SynthesisError> {
        // Similar to AmountVar::quo_rem
        // x = q * n + r
        // Constrain 0 <= r
        // Constrain r < n
        // n will be 256 bits wide
        // x will be 384 bits wide
        // so may need wide(r) multiplication

        // x = self (logical 128-bit)
        // q = rhs (logical 128-bit)
        // xbar = x * 2^128 (representative 256-bit)
        // qbar = q * 2^128 (representative 256-bit)

        // y = x / q
        // ybar = y * 2^128 (256 bit value)

        // xbar / qbar = x / q * 1
        // ybar = xbar / qbar * 2^128
        // xbar * 2^128 = qbar * ybar + r

        // we get ybar (256 bit)
        // r at most (256 bit)
        // division oracle = OOC computation

        // Need: 384-bit multiplication for qbar * ybar + r
        // Need: Constrain 256-bit values

        todo!()
    }
}

// Move to upstream?
// Add tests for the below functions

/// Convert Uint64 into an FqVar (make generic)
pub fn convert_uint64_to_fqvar(value: &UInt64<Fq>) -> FqVar {
    convert_le_bits_to_fqvar(&value.to_bits_le())
}

pub fn convert_le_bits_to_fqvar(value: &[Boolean<Fq>]) -> FqVar {
    let mut acc = FqVar::zero();
    for (i, bit) in value.into_iter().enumerate() {
        let bit = bit
            .to_constraint_field()
            .expect("can convert to FqVar")
            .into_iter()
            .next()
            .expect("should be one element in the vector");
        acc += bit * Fq::from(1u128 << i);
    }
    acc
}

/// Bit constrain for FqVar and number of bits
pub fn fqvar_to_bits(value: FqVar, n: usize) -> Result<Vec<Boolean<Fq>>, SynthesisError> {
    // Figure out what to do for setup phase in the below
    let inner = value.value()?;

    // Get only first n bits based on that value (OOC)
    let inner_bigint = inner.into_repr();
    let bits = &inner_bigint.to_bits_le()[0..n];

    // Allocate Boolean vars for first n bits
    let mut boolean_constraints = Vec::new();
    for bit in bits {
        let boolean = Boolean::new_witness(value.cs().clone(), || Ok(bit))?;
        boolean_constraints.push(boolean);
    }

    // Construct an FqVar from those n Boolean constraints, and constrain it to be equal to the original value
    let constructed_fqvar = convert_le_bits_to_fqvar(&boolean_constraints);
    constructed_fqvar.enforce_equal(&value)?;

    Ok(boolean_constraints)
}
