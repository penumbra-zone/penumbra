use std::fmt::{Debug, Display};

mod div;
mod from;
mod ops;

#[cfg(test)]
mod tests;

use ark_ff::{BigInteger, PrimeField, ToConstraintField, Zero};
use ark_r1cs_std::bits::uint64::UInt64;
use ark_r1cs_std::fields::fp::FpVar;
use ark_r1cs_std::prelude::*;
use ark_relations::r1cs::{ConstraintSystemRef, SynthesisError};

use decaf377::{r1cs::FqVar, Fq};
use ethnum::U256;

use self::div::stub_div_rem_u384_by_u256;

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

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct U128x128(U256);

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

    /// Performs checked multiplication, returning `Ok` if no overflow occurred.
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

    /// Performs checked division, returning `Ok` if no overflow occurred.
    pub fn checked_div(self, rhs: &Self) -> Result<Self, Error> {
        stub_div_rem_u384_by_u256(self.0, rhs.0).map(|(quo, rem)| U128x128(quo))
    }

    /// Performs checked addition, returning `Ok` if no overflow occurred.
    pub fn checked_add(self, rhs: &Self) -> Result<Self, Error> {
        self.0
            .checked_add(rhs.0)
            .map(U128x128)
            .ok_or(Error::Overflow)
    }

    /// Performs checked subtraction, returning `Ok` if no underflow occurred.
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

        let bytes = inner.to_bytes();
        // The U128x128 type uses a big-endian encoding
        let limb_3 = u64::from_be_bytes(bytes[0..8].try_into().unwrap());
        let limb_2 = u64::from_be_bytes(bytes[8..16].try_into().unwrap());
        let limb_1 = u64::from_be_bytes(bytes[16..24].try_into().unwrap());
        let limb_0 = u64::from_be_bytes(bytes[24..32].try_into().unwrap());

        let limb_0_var = UInt64::new_variable(cs.clone(), || Ok(limb_0), mode)?;
        let limb_1_var = UInt64::new_variable(cs.clone(), || Ok(limb_1), mode)?;
        let limb_2_var = UInt64::new_variable(cs.clone(), || Ok(limb_2), mode)?;
        let limb_3_var = UInt64::new_variable(cs, || Ok(limb_3), mode)?;

        Ok(Self {
            limbs: [limb_0_var, limb_1_var, limb_2_var, limb_3_var],
        })
    }
}

impl R1CSVar<Fq> for U128x128Var {
    type Value = U128x128;

    fn cs(&self) -> ark_relations::r1cs::ConstraintSystemRef<Fq> {
        self.limbs[0].cs()
    }

    fn value(&self) -> Result<Self::Value, ark_relations::r1cs::SynthesisError> {
        let x0 = self.limbs[0].value()?;
        let x1 = self.limbs[1].value()?;
        let x2 = self.limbs[2].value()?;
        let x3 = self.limbs[3].value()?;

        let mut bytes = [0u8; 32];
        bytes[0..8].copy_from_slice(x3.to_be_bytes().as_ref());
        bytes[8..16].copy_from_slice(x2.to_be_bytes().as_ref());
        bytes[16..24].copy_from_slice(x1.to_be_bytes().as_ref());
        bytes[24..32].copy_from_slice(x0.to_be_bytes().as_ref());

        Ok(Self::Value::from_bytes(bytes))
    }
}

impl U128x128Var {
    pub fn checked_add(
        self,
        rhs: &Self,
        cs: ConstraintSystemRef<Fq>,
    ) -> Result<U128x128Var, SynthesisError> {
        todo!();
    }

    pub fn checked_sub(
        self,
        rhs: &Self,
        cs: ConstraintSystemRef<Fq>,
    ) -> Result<U128x128Var, SynthesisError> {
        todo!();
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
        //let z0 = x0.clone() * y0.clone();
        let z0 = &x0 * &y0;
        let z1 = &x0 * &y1 + &x1 * &y0;
        let z2 = &x0 * &y2 + &x1 * &y1 + &x2 * &y0;
        let z3 = &x0 * &y3 + &x1 * &y2 + &x2 * &y1 + &x3 * &y0;
        let z4 = &x1 * &y3 + &x2 * &y2 + &x3 * &y1;
        let z5 = &x2 * &y3 + &x3 * &y2;
        let z6 = &x3 * &y3;
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
        let t0_bits = bit_constrain(t0, 193)?;
        // Constrain: t0 fits in 193 bits

        // t1 = (t0 >> 128) + z2
        let t1 = z2 + convert_le_bits_to_fqvar(&t0_bits[128..193]);
        // Constrain: t1 fits in 129 bits
        let t1_bits = bit_constrain(t1, 129)?;

        // w0 = t0 & 2^64 - 1
        let w0 = UInt64::from_bits_le(&t0_bits[0..64]);

        // t2 = (t1 >> 64) + z3
        let t2 = z3 + convert_le_bits_to_fqvar(&t1_bits[64..129]);
        // Constrain: t2 fits in 129 bits
        let t2_bits = bit_constrain(t2, 129)?;

        // w1 = t2 & 2^64 - 1
        let w1 = UInt64::from_bits_le(&t2_bits[0..64]);

        // t3 = (t2 >> 64) + z4
        let t3 = z4 + convert_le_bits_to_fqvar(&t2_bits[64..129]);
        // Constrain: t3 fits in 129 bits
        let t3_bits = bit_constrain(t3, 129)?;

        // w2 = t3 & 2^64 - 1
        let w2 = UInt64::from_bits_le(&t3_bits[0..64]);

        // t4 = (t3 >> 64) + z5
        let t4 = z5 + convert_le_bits_to_fqvar(&t3_bits[64..129]);
        // Constrain: t4 fits in 64 bits
        let t4_bits = bit_constrain(t4, 64)?;
        // If we didn't overflow, it will fit in 64 bits.

        // w3 = t4 & 2^64 - 1
        let w3 = UInt64::from_bits_le(&t4_bits[0..64]);

        // Overflow condition. Constrain: z6 = 0.
        z6.enforce_equal(&FqVar::zero())?;

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
        // x = q * y + r
        // Constrain 0 <= r
        // Constrain r < q
        // y will be 256 bits wide
        // x will be 384 bits wide

        // x = self (logical 128-bit)
        // y = rhs (logical 128-bit)
        // xbar = x * 2^128 (representative 256-bit)
        // ybar = y * 2^128 (representative 256-bit)

        // q = x / y
        // qbar = q * 2^128 (256 bit value)

        // xbar / ybar = x / y * 1
        // qbar = xbar / ybar * 2^128
        // xbar * 2^128 = qbar * ybar + r

        // use a division oracle to compute (qbar, r) out-of-circut
        // Constrain xbar * 2^128 = qbar * ybar + r
        // Constrain 0 <= r
        // Constrain r < qbar

        // OOC division
        let xbar_ooc = self.value().unwrap_or_default();
        let ybar_ooc = rhs.value().unwrap_or(U128x128::from(1u64));
        let Ok((quo_ooc, rem_ooc)) = stub_div_rem_u384_by_u256(xbar_ooc.0, ybar_ooc.0) else {
            return Err(SynthesisError::Unsatisfiable);
        };

        // Goal: Constrain xbar * 2^128 = qbar * ybar + r
        // We already have xbar as bits, so we have xbar * 2^128 "for free" by rearranging limbs
        // Need the bits of qbar * ybar + r => need bits of qbar, ybar, r + mul constraint

        let x = self;
        let y = rhs;
        let q = U128x128Var::new_witness(cs.clone(), || Ok(U128x128(quo_ooc)))?;
        // r isn't a U128x128, but we can reuse the codepath for constraining its bits as limb values
        let r = U128x128Var::new_witness(cs.clone(), || Ok(U128x128(rem_ooc)))?.limbs;

        let qbar = &q.limbs;
        let ybar = &y.limbs;
        let xbar = &x.limbs;

        // qbar = qbar0 + qbar1 * 2^64 + qbar2 * 2^128 + qbar3 * 2^192
        // ybar = ybar0 + ybar1 * 2^64 + ybar2 * 2^128 + ybar3 * 2^192
        //    r =    r0 +    r1 * 2^64 +    r2 * 2^128 +    r3 * 2^192

        let xbar0 = convert_uint64_to_fqvar(&xbar[0]);
        let xbar1 = convert_uint64_to_fqvar(&xbar[1]);
        let xbar2 = convert_uint64_to_fqvar(&xbar[2]);
        let xbar3 = convert_uint64_to_fqvar(&xbar[3]);

        let ybar0 = convert_uint64_to_fqvar(&ybar[0]);
        let ybar1 = convert_uint64_to_fqvar(&ybar[1]);
        let ybar2 = convert_uint64_to_fqvar(&ybar[2]);
        let ybar3 = convert_uint64_to_fqvar(&ybar[3]);

        let qbar0 = convert_uint64_to_fqvar(&qbar[0]);
        let qbar1 = convert_uint64_to_fqvar(&qbar[1]);
        let qbar2 = convert_uint64_to_fqvar(&qbar[2]);
        let qbar3 = convert_uint64_to_fqvar(&qbar[3]);

        let r0 = convert_uint64_to_fqvar(&r[0]);
        let r1 = convert_uint64_to_fqvar(&r[1]);
        let r2 = convert_uint64_to_fqvar(&r[2]);
        let r3 = convert_uint64_to_fqvar(&r[3]);

        // Let z = qbar * ybar + r.  Then z will be 513 bits in general; we want
        // to constrain it to be equal to xbar * 2^128 so we need the low 384
        // bits -- we'll constrain the low 128 as 0 and the upper 256 as xbar --
        // and constrain everything above as 0 (not necessarily as bit
        // constraints)

        // Write z as:
        //    z =    z0 +    z1 * 2^64 +    z2 * 2^128 +    z3 * 2^192 +    z4 * 2^256 +    z5 * 2^320 +    z6 * 2^384
        // Without carrying, the limbs of z are:
        // z0_raw = r0 + qbar0 * ybar0
        // z1_raw = r1 + qbar1 * ybar0 + qbar0 * ybar1
        // z2_raw = r2 + qbar2 * ybar0 + qbar1 * ybar1 + qbar0 * ybar2
        // z3_raw = r3 + qbar3 * ybar0 + qbar2 * ybar1 + qbar1 * ybar2 + qbar0 * ybar3
        // z4_raw =                      qbar3 * ybar1 + qbar2 * ybar2 + qbar1 * ybar3
        // z5_raw =                                      qbar3 * ybar2 + qbar2 * ybar3
        // z6_raw =                                                      qbar3 * ybar3

        let z0_raw = r0 + &qbar0 * &ybar0;
        let z1_raw = r1 + &qbar1 * &ybar0 + &qbar0 * &ybar1;
        let z2_raw = r2 + &qbar2 * &ybar0 + &qbar1 * &ybar1 + &qbar0 * &ybar2;
        let z3_raw = r3 + &qbar3 * &ybar0 + &qbar2 * &ybar1 + &qbar1 * &ybar2 + &qbar0 * &ybar3;
        let z4_raw = /*__________________*/ &qbar3 * &ybar1 + &qbar2 * &ybar2 + &qbar1 * &ybar3;
        let z5_raw = /*____________________________________*/ &qbar3 * &ybar2 + &qbar2 * &ybar3;
        let z6_raw = /*______________________________________________________*/ &qbar3 * &ybar3;
        /* ------------------------------------------------------------------------------------^ 384 + 128 = 512 */

        // These terms are overlapping, and we need to carry to compute the
        // canonical limbs.
        //
        // We want to constrain
        //    z =    z0 +    z1 * 2^64 +    z2 * 2^128 +    z3 * 2^192 +    z4 * 2^256 +    z5 * 2^320 +    z6 * 2^384
        // ==         0       0          xbar0           xbar1           xbar2           xbar3              0
        // We need to bit-constrain z0 and z1 to be able to compute the carry to
        // get the canonical z2, z3, z4, z5 values, but don't need bit constraints
        // for the upper terms, we just need to enforce that they're 0, without the
        // possibility of wrapping in the finite field.

        // z0 < 2^128 + 2^64 < 2^(128 + 1) => 129 bits
        let z0_bits = bit_constrain(z0_raw, 129)?; // no carry-in
        let z0 = convert_le_bits_to_fqvar(&z0_bits[0..64]);
        let c1 = convert_le_bits_to_fqvar(&z0_bits[64..]); // 65 bits

        // z1 < 2^64 + 2 * 2^128 + 2^65 < 3*2^128 < 2^(128 + 2) => 130 bits
        let z1_bits = bit_constrain(z1_raw + c1, 130)?; // carry-in c1
        let z1 = convert_le_bits_to_fqvar(&z1_bits[0..64]);
        let c2 = convert_le_bits_to_fqvar(&z1_bits[64..]); // 66 bits

        // z2 < 2^64 + 3 * 2^128 + 2^66 < 4*2^128 = 2^(128 + 2) => 130 bits
        let z2_bits = bit_constrain(z2_raw + c2, 130)?; // carry-in c2
        let z2 = convert_le_bits_to_fqvar(&z2_bits[0..64]);
        let c3 = convert_le_bits_to_fqvar(&z2_bits[64..]); // 66 bits

        // z3 < 2^64 + 4 * 2^128 + 2^66 < 5*2^128 = 2^(128 + 3) => 131 bits
        let z3_bits = bit_constrain(z3_raw + c3, 131)?; // carry-in c3
        let z3 = convert_le_bits_to_fqvar(&z3_bits[0..64]);
        let c4 = convert_le_bits_to_fqvar(&z3_bits[64..]); // 67 bits

        // z4 < 3 * 2^128 + 2^67 < 4*2^128 = 2^(128 + 2) => 130 bits
        let z4_bits = bit_constrain(z4_raw + c4, 130)?; // carry-in c4
        let z4 = convert_le_bits_to_fqvar(&z4_bits[0..64]);
        let c5 = convert_le_bits_to_fqvar(&z4_bits[64..]); // 66 bits

        // z5 < 2 * 2^128 + 2^66 < 3*2^128 = 2^(128 + 2) => 130 bits
        let z5_bits = bit_constrain(z5_raw + c5, 130)?; // carry-in c5
        let z5 = convert_le_bits_to_fqvar(&z5_bits[0..64]);
        let c6 = convert_le_bits_to_fqvar(&z5_bits[64..]); // 66 bits

        // z6_plus < 2^128 + 2^66 < 2*2^128 = 2^(128 + 1) => 129 bits
        // Since 129 bits < field modulus, we can enforce z6_plus = 0
        // without bit constraining.
        let z6_and_up = z6_raw + c6;
        // Note: 129 + 384 = 513 so this is all of the remaining bits

        // Repeat:
        // We want to constrain
        //    z =    z0 +    z1 * 2^64 +    z2 * 2^128 +    z3 * 2^192 +    z4 * 2^256 +    z5 * 2^320 +    z6 * 2^384
        // ==         0       0          xbar0           xbar1           xbar2           xbar3              0
        z0.enforce_equal(&FqVar::zero())?;
        z1.enforce_equal(&FqVar::zero())?;
        z2.enforce_equal(&xbar0)?;
        z3.enforce_equal(&xbar1)?;
        z4.enforce_equal(&xbar2)?;
        z5.enforce_equal(&xbar3)?;
        z6_and_up.enforce_equal(&FqVar::zero())?;

        Ok(q)
    }
}

impl EqGadget<Fq> for U128x128Var {
    fn is_eq(&self, other: &Self) -> Result<Boolean<Fq>, SynthesisError> {
        let limb_1_eq = self.limbs[0].is_eq(&other.limbs[0])?;
        let limb_2_eq = self.limbs[1].is_eq(&other.limbs[1])?;
        let limb_3_eq = self.limbs[2].is_eq(&other.limbs[2])?;
        let limb_4_eq = self.limbs[3].is_eq(&other.limbs[3])?;

        let limb_12_eq = limb_1_eq.and(&limb_2_eq)?;
        let limb_34_eq = limb_3_eq.and(&limb_4_eq)?;

        limb_12_eq.and(&limb_34_eq)
    }
}

impl ToConstraintField<Fq> for U128x128 {
    fn to_field_elements(&self) -> Option<Vec<Fq>> {
        let bytes = self.to_bytes();
        let limb_3 = u64::from_be_bytes(bytes[0..8].try_into().unwrap());
        let limb_2 = u64::from_be_bytes(bytes[8..16].try_into().unwrap());
        let limb_1 = u64::from_be_bytes(bytes[16..24].try_into().unwrap());
        let limb_0 = u64::from_be_bytes(bytes[24..32].try_into().unwrap());

        let field_elements = vec![
            Fq::from(limb_0),
            Fq::from(limb_1),
            Fq::from(limb_2),
            Fq::from(limb_3),
        ];
        Some(field_elements)
    }
}

/// Convert Uint64 into an FqVar
pub fn convert_uint64_to_fqvar<F: PrimeField>(value: &UInt64<F>) -> FpVar<F> {
    convert_le_bits_to_fqvar(&value.to_bits_le())
}

/// Modular exponentiation
pub fn mod_exp<F: PrimeField>(f: F, exp: usize) -> F {
    let mut acc = F::from(1u32);
    for _ in 0..exp {
        acc *= f;
    }
    acc
}

/// Convert little-endian boolean constraints into a field element
pub fn convert_le_bits_to_fqvar<F: PrimeField>(value: &[Boolean<F>]) -> FpVar<F> {
    let mut acc = FpVar::<F>::zero();
    for (i, bit) in value.iter().enumerate() {
        acc += FpVar::<F>::from(bit.clone()) * FpVar::<F>::constant(mod_exp(F::from(2_u32), i));
    }
    acc
}

/// Bit constrain for FqVar and return number of bits
pub fn bit_constrain(value: FqVar, n: usize) -> Result<Vec<Boolean<Fq>>, SynthesisError> {
    let inner = value.value().unwrap_or(Fq::zero());

    // Get only first n bits based on that value (OOC)
    let inner_bigint = inner.into_bigint();
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

#[cfg(test)]
mod test {
    use ark_groth16::{r1cs_to_qap::LibsnarkReduction, Groth16, ProvingKey, VerifyingKey};
    use ark_relations::r1cs::ConstraintSynthesizer;
    use ark_snark::SNARK;
    use decaf377::Bls12_377;
    use proptest::prelude::*;
    use rand_core::OsRng;

    use crate::proofs::groth16::ParameterSetup;

    use super::*;

    fn u128x128_strategy() -> BoxedStrategy<U128x128> {
        any::<[u8; 32]>().prop_map(U128x128::from_bytes).boxed()
    }

    proptest! {
        #[test]
        fn test_convert_uint64_to_fqvar(
            num in any::<u64>(),
        ) {
            let num_var: UInt64<Fq> = UInt64::constant(num);
            let expected_field_element = FqVar::constant(Fq::from(num));
            let field_element = convert_uint64_to_fqvar(&num_var);
            assert_eq!(field_element.value().unwrap(), expected_field_element.value().unwrap());
        }
    }

    proptest! {
        #[test]
        fn multiply(
            a in u128x128_strategy(),
            b in u128x128_strategy(),
        ) {
            let result = a.checked_mul(&b);
            // If the result overflows, the circuit will be unsatisfiable at proving time.
            if result.is_err() {
                return Ok(());
            }

            let expected_c = result.unwrap();

            let circuit = TestMultiplicationCircuit {
                a,
                b,
                c: expected_c,
            };

            let (pk, vk) = TestMultiplicationCircuit::generate_prepared_test_parameters();
            let mut rng = OsRng;

            let proof = Groth16::<Bls12_377, LibsnarkReduction>::prove(&pk, circuit, &mut rng)
            .expect("should be able to form proof");

            let proof_result = Groth16::<Bls12_377, LibsnarkReduction>::verify_with_processed_vk(
                &vk,
                &expected_c.to_field_elements().unwrap(),
                &proof,
            );
            assert!(proof_result.is_ok());
        }
    }

    struct TestMultiplicationCircuit {
        a: U128x128,
        b: U128x128,

        // c = a * b
        pub c: U128x128,
    }

    impl ConstraintSynthesizer<Fq> for TestMultiplicationCircuit {
        fn generate_constraints(
            self,
            cs: ConstraintSystemRef<Fq>,
        ) -> ark_relations::r1cs::Result<()> {
            let a_var = U128x128Var::new_witness(cs.clone(), || Ok(self.a))?;
            let b_var = U128x128Var::new_witness(cs.clone(), || Ok(self.b))?;
            let c_public_var = U128x128Var::new_input(cs.clone(), || Ok(self.c))?;
            let c_var = a_var.checked_mul(&b_var, cs)?;
            c_var.enforce_equal(&c_public_var)?;
            Ok(())
        }
    }

    impl ParameterSetup for TestMultiplicationCircuit {
        fn generate_test_parameters() -> (ProvingKey<Bls12_377>, VerifyingKey<Bls12_377>) {
            let num: [u8; 32] = [0u8; 32];
            let a = U128x128::from_bytes(num);
            let b = U128x128::from_bytes(num);
            let circuit = TestMultiplicationCircuit {
                a,
                b,
                c: a.checked_mul(&b).unwrap(),
            };
            let (pk, vk) = Groth16::<Bls12_377, LibsnarkReduction>::circuit_specific_setup(
                circuit, &mut OsRng,
            )
            .expect("can perform circuit specific setup");
            (pk, vk)
        }
    }

    proptest! {
        #[ignore]
        #[test]
        fn add(
            a in u128x128_strategy(),
            b in u128x128_strategy(),
        ) {
            let result = a.checked_add(&b);
            // If the addition overflows, the circuit will be unsatisfiable at proving time.
            if result.is_err() {
                return Ok(());
            }

            let expected_c = result.unwrap();

            let circuit = TestAdditionCircuit {
                a,
                b,
                c: expected_c,
            };

            let (pk, vk) = TestAdditionCircuit::generate_prepared_test_parameters();
            let mut rng = OsRng;

            let proof = Groth16::<Bls12_377, LibsnarkReduction>::prove(&pk, circuit, &mut rng)
            .expect("should be able to form proof");

            let proof_result = Groth16::<Bls12_377, LibsnarkReduction>::verify_with_processed_vk(
                &vk,
                &expected_c.to_field_elements().unwrap(),
                &proof,
            );
            assert!(proof_result.is_ok());
        }
    }

    struct TestAdditionCircuit {
        a: U128x128,
        b: U128x128,

        // c = a + b
        pub c: U128x128,
    }

    impl ConstraintSynthesizer<Fq> for TestAdditionCircuit {
        fn generate_constraints(
            self,
            cs: ConstraintSystemRef<Fq>,
        ) -> ark_relations::r1cs::Result<()> {
            let a_var = U128x128Var::new_witness(cs.clone(), || Ok(self.a))?;
            let b_var = U128x128Var::new_witness(cs.clone(), || Ok(self.b))?;
            let c_public_var = U128x128Var::new_input(cs.clone(), || Ok(self.c))?;
            let c_var = a_var.checked_add(&b_var, cs)?;
            c_var.enforce_equal(&c_public_var)?;
            Ok(())
        }
    }

    impl ParameterSetup for TestAdditionCircuit {
        fn generate_test_parameters() -> (ProvingKey<Bls12_377>, VerifyingKey<Bls12_377>) {
            let num: [u8; 32] = [0u8; 32];
            let a = U128x128::from_bytes(num);
            let b = U128x128::from_bytes(num);
            let circuit = TestAdditionCircuit {
                a,
                b,
                c: a.checked_add(&b).unwrap(),
            };
            let (pk, vk) = Groth16::<Bls12_377, LibsnarkReduction>::circuit_specific_setup(
                circuit, &mut OsRng,
            )
            .expect("can perform circuit specific setup");
            (pk, vk)
        }
    }
}
