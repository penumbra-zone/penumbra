use std::{
    fmt::{Debug, Display},
    iter::zip,
};

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

use crate::{Amount, AmountVar};

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
        let hi = u128::from_be_bytes(bytes[0..16].try_into().expect("slice is 16 bytes"));
        let lo = u128::from_be_bytes(bytes[16..32].try_into().expect("slice is 16 bytes"));
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
    pub fn round_up(&self) -> Result<Self, Error> {
        let (integral, fractional) = self.0.into_words();
        if fractional == 0 {
            Ok(*self)
        } else {
            let integral = integral.checked_add(1).ok_or(Error::Overflow)?;
            Ok(Self(U256::from_words(integral, 0u128)))
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
        stub_div_rem_u384_by_u256(self.0, rhs.0).map(|(quo, _rem)| U128x128(quo))
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

    /// Multiply an amount by this fraction, then round down.
    pub fn apply_to_amount(self, rhs: &Amount) -> Result<Amount, Error> {
        let mul = (Self::from(rhs) * self)?;
        let out = mul
            .round_down()
            .try_into()
            .expect("converting integral U128xU128 into Amount will succeed");
        Ok(out)
    }
}

#[derive(Clone)]
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

        // TODO: in the case of a constant U128x128Var, this will allocate
        // witness vars intsead of constants, but we don't have much use for
        // constant U128x128Vars anyways, so this efficiency loss shouldn't be a
        // problem.

        let (hi_128, lo_128) = inner.0.into_words();
        let hi_128_var = FqVar::new_variable(cs.clone(), || Ok(Fq::from(hi_128)), mode)?;
        let lo_128_var = FqVar::new_variable(cs.clone(), || Ok(Fq::from(lo_128)), mode)?;

        // Now construct the bit constraints out of thin air ...
        let bytes = inner.to_bytes();
        // The U128x128 type uses a big-endian encoding
        let limb_3 = u64::from_be_bytes(bytes[0..8].try_into().expect("slice is 8 bytes"));
        let limb_2 = u64::from_be_bytes(bytes[8..16].try_into().expect("slice is 8 bytes"));
        let limb_1 = u64::from_be_bytes(bytes[16..24].try_into().expect("slice is 8 bytes"));
        let limb_0 = u64::from_be_bytes(bytes[24..32].try_into().expect("slice is 8 bytes"));

        let limb_0_var = UInt64::new_variable(cs.clone(), || Ok(limb_0), AllocationMode::Witness)?;
        let limb_1_var = UInt64::new_variable(cs.clone(), || Ok(limb_1), AllocationMode::Witness)?;
        let limb_2_var = UInt64::new_variable(cs.clone(), || Ok(limb_2), AllocationMode::Witness)?;
        let limb_3_var = UInt64::new_variable(cs, || Ok(limb_3), AllocationMode::Witness)?;

        // ... and then bind them to the input variables we created above.
        let lo_128_bits = limb_0_var
            .to_bits_le()
            .into_iter()
            .chain(limb_1_var.to_bits_le())
            .collect::<Vec<_>>();
        let hi_128_bits = limb_2_var
            .to_bits_le()
            .into_iter()
            .chain(limb_3_var.to_bits_le())
            .collect::<Vec<_>>();

        hi_128_var.enforce_equal(&Boolean::<Fq>::le_bits_to_fp_var(
            &(hi_128_bits[..]).to_bits_le()?,
        )?)?;
        lo_128_var.enforce_equal(&Boolean::<Fq>::le_bits_to_fp_var(
            &(lo_128_bits[..]).to_bits_le()?,
        )?)?;

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
    pub fn checked_add(self, rhs: &Self) -> Result<U128x128Var, SynthesisError> {
        // x = [x0, x1, x2, x3]
        // x = x0 + x1 * 2^64 + x2 * 2^128 + x3 * 2^192
        // y = [y0, y1, y2, y3]
        // y = y0 + y1 * 2^64 + y2 * 2^128 + y3 * 2^192
        let x0 = Boolean::<Fq>::le_bits_to_fp_var(&self.limbs[0].to_bits_le())?;
        let x1 = Boolean::<Fq>::le_bits_to_fp_var(&self.limbs[1].to_bits_le())?;
        let x2 = Boolean::<Fq>::le_bits_to_fp_var(&self.limbs[2].to_bits_le())?;
        let x3 = Boolean::<Fq>::le_bits_to_fp_var(&self.limbs[3].to_bits_le())?;

        let y0 = Boolean::<Fq>::le_bits_to_fp_var(&rhs.limbs[0].to_bits_le())?;
        let y1 = Boolean::<Fq>::le_bits_to_fp_var(&rhs.limbs[1].to_bits_le())?;
        let y2 = Boolean::<Fq>::le_bits_to_fp_var(&rhs.limbs[2].to_bits_le())?;
        let y3 = Boolean::<Fq>::le_bits_to_fp_var(&rhs.limbs[3].to_bits_le())?;

        // z = x + y
        // z = [z0, z1, z2, z3]
        let z0_raw = &x0 + &y0;
        let z1_raw = &x1 + &y1;
        let z2_raw = &x2 + &y2;
        let z3_raw = &x3 + &y3;

        // z0 <= (2^64 - 1) + (2^64 - 1) < 2^(65) => 65 bits
        let z0_bits = bit_constrain(z0_raw, 65)?; // no carry-in
        let z0 = UInt64::from_bits_le(&z0_bits[0..64]);
        let c1 = Boolean::<Fq>::le_bits_to_fp_var(&z0_bits[64..].to_bits_le()?)?;

        // z1 <= (2^64 - 1) + (2^64 - 1) + 1 < 2^(65) => 65 bits
        let z1_bits = bit_constrain(z1_raw + c1, 65)?; // carry-in c1
        let z1 = UInt64::from_bits_le(&z1_bits[0..64]);
        let c2 = Boolean::<Fq>::le_bits_to_fp_var(&z1_bits[64..].to_bits_le()?)?;

        // z2 <= (2^64 - 1) + (2^64 - 1) + 1 < 2^(65) => 65 bits
        let z2_bits = bit_constrain(z2_raw + c2, 65)?; // carry-in c2
        let z2 = UInt64::from_bits_le(&z2_bits[0..64]);
        let c3 = Boolean::<Fq>::le_bits_to_fp_var(&z2_bits[64..].to_bits_le()?)?;

        // z3 <= (2^64 - 1) + (2^64 - 1) + 1 < 2^(65) => 65 bits
        // However, the last bit (65th) which would be used as a final carry flag, should be 0 if there is no overflow.
        // As such, we can constrain the length for this call to 64 bits.
        let z3_bits = bit_constrain(z3_raw + c3, 64)?; // carry-in c3
        let z3 = UInt64::from_bits_le(&z3_bits[0..64]);

        Ok(Self {
            limbs: [z0, z1, z2, z3],
        })
    }

    pub fn checked_sub(
        self,
        _rhs: &Self,
        _cs: ConstraintSystemRef<Fq>,
    ) -> Result<U128x128Var, SynthesisError> {
        todo!();
    }

    pub fn checked_mul(self, rhs: &Self) -> Result<U128x128Var, SynthesisError> {
        // x = [x0, x1, x2, x3]
        // x = x0 + x1 * 2^64 + x2 * 2^128 + x3 * 2^192
        // y = [y0, y1, y2, y3]
        // y = y0 + y1 * 2^64 + y2 * 2^128 + y3 * 2^192
        let x0 = Boolean::<Fq>::le_bits_to_fp_var(&self.limbs[0].to_bits_le())?;
        let x1 = Boolean::<Fq>::le_bits_to_fp_var(&self.limbs[1].to_bits_le())?;
        let x2 = Boolean::<Fq>::le_bits_to_fp_var(&self.limbs[2].to_bits_le())?;
        let x3 = Boolean::<Fq>::le_bits_to_fp_var(&self.limbs[3].to_bits_le())?;

        let y0 = Boolean::<Fq>::le_bits_to_fp_var(&rhs.limbs[0].to_bits_le())?;
        let y1 = Boolean::<Fq>::le_bits_to_fp_var(&rhs.limbs[1].to_bits_le())?;
        let y2 = Boolean::<Fq>::le_bits_to_fp_var(&rhs.limbs[2].to_bits_le())?;
        let y3 = Boolean::<Fq>::le_bits_to_fp_var(&rhs.limbs[3].to_bits_le())?;

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
        let t1 = z2 + Boolean::<Fq>::le_bits_to_fp_var(&t0_bits[128..193].to_bits_le()?)?;
        // Constrain: t1 fits in 130 bits
        let t1_bits = bit_constrain(t1, 130)?;

        // w0 = t1 & 2^64 - 1
        let w0 = UInt64::from_bits_le(&t1_bits[0..64]);

        // t2 = (t1 >> 64) + z3
        let t2 = z3 + Boolean::<Fq>::le_bits_to_fp_var(&t1_bits[64..129].to_bits_le()?)?;
        // Constrain: t2 fits in 129 bits
        let t2_bits = bit_constrain(t2, 129)?;

        // w1 = t2 & 2^64 - 1
        let w1 = UInt64::from_bits_le(&t2_bits[0..64]);

        // t3 = (t2 >> 64) + z4
        let t3 = z4 + Boolean::<Fq>::le_bits_to_fp_var(&t2_bits[64..129].to_bits_le()?)?;
        // Constrain: t3 fits in 128 bits
        let t3_bits = bit_constrain(t3, 128)?;

        // w2 = t3 & 2^64 - 1
        let w2 = UInt64::from_bits_le(&t3_bits[0..64]);

        // t4 = (t3 >> 64) + z5
        let t4 = z5 + Boolean::<Fq>::le_bits_to_fp_var(&t3_bits[64..128].to_bits_le()?)?;
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

    pub fn to_bits_le(&self) -> Vec<Boolean<Fq>> {
        let lo_128_bits = self.limbs[0]
            .to_bits_le()
            .into_iter()
            .chain(self.limbs[1].to_bits_le())
            .collect::<Vec<_>>();
        let hi_128_bits = self.limbs[2]
            .to_bits_le()
            .into_iter()
            .chain(self.limbs[3].to_bits_le())
            .collect::<Vec<_>>();
        lo_128_bits.into_iter().chain(hi_128_bits).collect()
    }

    /// This function enforces the ordering between `self` and `other`.
    pub fn enforce_cmp(
        &self,
        other: &U128x128Var,
        ordering: std::cmp::Ordering,
    ) -> Result<(), SynthesisError> {
        // Collect bits from each limb to be compared.
        let self_bits: Vec<Boolean<Fq>> = self.to_bits_le().into_iter().rev().collect();
        let other_bits: Vec<Boolean<Fq>> = other.to_bits_le().into_iter().rev().collect();

        // Now starting at the most significant side, compare bits.
        // `gt` is true if we have conclusively determined that self > other.
        // `lt` is true if we have conclusively determined that self < other.
        let mut gt: Boolean<Fq> = Boolean::constant(false);
        let mut lt: Boolean<Fq> = Boolean::constant(false);
        for (p, q) in zip(self_bits, other_bits) {
            // If we've determined that self > other, that will remain
            // true as we continue to look at other bits. Otherwise,
            // we need to make sure that we don't have self < other.
            // At this point, if we see a 1 bit for self and a 0 bit for other,
            // we know that self > other.
            gt = gt.or(&lt.not().and(&p)?.and(&q.not())?)?;
            // The exact same logic, but swapping gt <-> lt, p <-> q
            lt = lt.or(&gt.not().and(&q)?.and(&p.not())?)?;
        }

        match ordering {
            std::cmp::Ordering::Greater => {
                gt.enforce_equal(&Boolean::constant(true))?;
                lt.enforce_equal(&Boolean::constant(false))?;
            }
            std::cmp::Ordering::Less => {
                gt.enforce_equal(&Boolean::constant(false))?;
                lt.enforce_equal(&Boolean::constant(true))?;
            }
            std::cmp::Ordering::Equal => {
                unimplemented!("use EqGadget for efficiency");
            }
        }

        Ok(())
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

        // use a division oracle to compute (qbar, r) out-of-circuit (OOC)
        // Constrain: divisor is non-zero
        rhs.enforce_not_equal(&U128x128Var::zero())?;

        // OOC division
        let xbar_ooc = self.value().unwrap_or_default();
        let ybar_ooc = rhs.value().unwrap_or(U128x128::from(1u64));
        let Ok((quo_ooc, rem_ooc)) = stub_div_rem_u384_by_u256(xbar_ooc.0, ybar_ooc.0) else {
            return Err(SynthesisError::Unsatisfiable);
        };
        // Constrain: xbar * 2^128 = qbar * ybar + r
        // We already have xbar as bits, so we have xbar * 2^128 "for free" by rearranging limbs
        // Need the bits of qbar * ybar + r => need bits of qbar, ybar, r + mul constraint

        let x = self;
        let y = rhs;
        let q = U128x128Var::new_witness(cs.clone(), || Ok(U128x128(quo_ooc)))?;
        // r isn't a U128x128, but we can reuse the codepath for constraining its bits as limb values
        let r_var = U128x128Var::new_witness(cs, || Ok(U128x128(rem_ooc)))?;
        // Constrain r < ybar: this also constrains that r is non-negative, i.e. that 0 <= r
        // i.e. the remainder cannot be greater than the divisor (`y` also called `rhs`)
        r_var.enforce_cmp(rhs, core::cmp::Ordering::Less)?;

        let r = r_var.limbs;
        let qbar = &q.limbs;
        let ybar = &y.limbs;
        let xbar = &x.limbs;

        // qbar = qbar0 + qbar1 * 2^64 + qbar2 * 2^128 + qbar3 * 2^192
        // ybar = ybar0 + ybar1 * 2^64 + ybar2 * 2^128 + ybar3 * 2^192
        //    r =    r0 +    r1 * 2^64 +    r2 * 2^128 +    r3 * 2^192

        let xbar0 = Boolean::<Fq>::le_bits_to_fp_var(&xbar[0].to_bits_le())?;
        let xbar1 = Boolean::<Fq>::le_bits_to_fp_var(&xbar[1].to_bits_le())?;
        let xbar2 = Boolean::<Fq>::le_bits_to_fp_var(&xbar[2].to_bits_le())?;
        let xbar3 = Boolean::<Fq>::le_bits_to_fp_var(&xbar[3].to_bits_le())?;

        let ybar0 = Boolean::<Fq>::le_bits_to_fp_var(&ybar[0].to_bits_le())?;
        let ybar1 = Boolean::<Fq>::le_bits_to_fp_var(&ybar[1].to_bits_le())?;
        let ybar2 = Boolean::<Fq>::le_bits_to_fp_var(&ybar[2].to_bits_le())?;
        let ybar3 = Boolean::<Fq>::le_bits_to_fp_var(&ybar[3].to_bits_le())?;

        let qbar0 = Boolean::<Fq>::le_bits_to_fp_var(&qbar[0].to_bits_le())?;
        let qbar1 = Boolean::<Fq>::le_bits_to_fp_var(&qbar[1].to_bits_le())?;
        let qbar2 = Boolean::<Fq>::le_bits_to_fp_var(&qbar[2].to_bits_le())?;
        let qbar3 = Boolean::<Fq>::le_bits_to_fp_var(&qbar[3].to_bits_le())?;

        let r0 = Boolean::<Fq>::le_bits_to_fp_var(&r[0].to_bits_le())?;
        let r1 = Boolean::<Fq>::le_bits_to_fp_var(&r[1].to_bits_le())?;
        let r2 = Boolean::<Fq>::le_bits_to_fp_var(&r[2].to_bits_le())?;
        let r3 = Boolean::<Fq>::le_bits_to_fp_var(&r[3].to_bits_le())?;

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

        // z0 <= (2^64 - 1)(2^64 - 1) + (2^64 - 1) => 128 bits
        let z0_bits = bit_constrain(z0_raw, 128)?; // no carry-in
        let z0 = Boolean::<Fq>::le_bits_to_fp_var(&z0_bits[0..64].to_bits_le()?)?;
        let c1 = Boolean::<Fq>::le_bits_to_fp_var(&z0_bits[64..].to_bits_le()?)?; // 64 bits

        // z1 <= 2*(2^64 - 1)(2^64 - 1) + (2^64 - 1) + carry (2^64 - 1) => 129 bits
        let z1_bits = bit_constrain(z1_raw + c1, 129)?; // carry-in c1
        let z1 = Boolean::<Fq>::le_bits_to_fp_var(&z1_bits[0..64].to_bits_le()?)?;
        let c2 = Boolean::<Fq>::le_bits_to_fp_var(&z1_bits[64..].to_bits_le()?)?; // 65 bits

        // z2 <= 3*(2^64 - 1)(2^64 - 1) + (2^64 - 1) + carry (2^65 - 2) => 130 bits
        let z2_bits = bit_constrain(z2_raw + c2, 130)?; // carry-in c2
        let z2 = Boolean::<Fq>::le_bits_to_fp_var(&z2_bits[0..64].to_bits_le()?)?;
        let c3 = Boolean::<Fq>::le_bits_to_fp_var(&z2_bits[64..].to_bits_le()?)?; // 66 bits

        // z3 <= 4*(2^64 - 1)(2^64 - 1) + (2^64 - 1) + carry (2^66 - 1) => 130 bits
        let z3_bits = bit_constrain(z3_raw + c3, 130)?; // carry-in c3
        let z3 = Boolean::<Fq>::le_bits_to_fp_var(&z3_bits[0..64].to_bits_le()?)?;
        let c4 = Boolean::<Fq>::le_bits_to_fp_var(&z3_bits[64..].to_bits_le()?)?; // 66 bits

        // z4 <= 3*(2^64 - 1)(2^64 - 1) + carry (2^66 - 1) => 130 bits
        // But extra bits beyond 128 spill into z6, which should be zero, so we can constrain to 128 bits.
        let z4_bits = bit_constrain(z4_raw + c4, 128)?; // carry-in c4
        let z4 = Boolean::<Fq>::le_bits_to_fp_var(&z4_bits[0..64].to_bits_le()?)?;
        let c5 = Boolean::<Fq>::le_bits_to_fp_var(&z4_bits[64..].to_bits_le()?)?; // 64 bits

        // z5 <= 2*(2^64 - 1)(2^64 - 1) + (2^64 - 1)
        // But if there is no overflow, the final carry (which would be c6 constructed from z5_bits[64..])
        // should be zero. So instead of constructing that final carry, we can instead bit constrain z5 to
        // the first 64 bits to save constraints.
        let z5_bits = bit_constrain(z5_raw + c5, 64)?; // carry-in c5
        let z5 = Boolean::<Fq>::le_bits_to_fp_var(&z5_bits[0..64].to_bits_le()?)?;

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
        // z6_raw should be zero if there was no overflow.
        z6_raw.enforce_equal(&FqVar::zero())?;

        Ok(q)
    }

    pub fn round_down(self) -> U128x128Var {
        Self {
            limbs: [
                UInt64::constant(0u64),
                UInt64::constant(0u64),
                self.limbs[2].clone(),
                self.limbs[3].clone(),
            ],
        }
    }
    /// Multiply an amount by this fraction, then round down.
    pub fn apply_to_amount(self, rhs: AmountVar) -> Result<AmountVar, SynthesisError> {
        U128x128Var::from_amount_var(rhs)?
            .checked_mul(&self)?
            .round_down_to_amount()
    }

    pub fn round_down_to_amount(self) -> Result<AmountVar, SynthesisError> {
        let bits = self.limbs[2]
            .to_bits_le()
            .into_iter()
            .chain(self.limbs[3].to_bits_le().into_iter())
            .collect::<Vec<Boolean<Fq>>>();
        Ok(AmountVar {
            amount: Boolean::<Fq>::le_bits_to_fp_var(&bits)?,
        })
    }

    pub fn zero() -> U128x128Var {
        Self {
            limbs: [
                UInt64::constant(0u64),
                UInt64::constant(0u64),
                UInt64::constant(0u64),
                UInt64::constant(0u64),
            ],
        }
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
        let (hi_128, lo_128) = self.0.into_words();
        Some(vec![Fq::from(hi_128), Fq::from(lo_128)])
    }
}

impl CondSelectGadget<Fq> for U128x128Var {
    fn conditionally_select(
        cond: &Boolean<Fq>,
        true_value: &Self,
        false_value: &Self,
    ) -> Result<Self, SynthesisError> {
        let limb0 = cond.select(&true_value.limbs[0], &false_value.limbs[0])?;
        let limb1 = cond.select(&true_value.limbs[1], &false_value.limbs[1])?;
        let limb2 = cond.select(&true_value.limbs[2], &false_value.limbs[2])?;
        let limb3 = cond.select(&true_value.limbs[3], &false_value.limbs[3])?;
        Ok(Self {
            limbs: [limb0, limb1, limb2, limb3],
        })
    }
}

/// Convert Uint64 into an FqVar
pub fn convert_uint64_to_fqvar<F: PrimeField>(value: &UInt64<F>) -> FpVar<F> {
    Boolean::<F>::le_bits_to_fp_var(&value.to_bits_le()).expect("can convert to bits")
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
    let constructed_fqvar = Boolean::<Fq>::le_bits_to_fp_var(&boolean_constraints.to_bits_le()?)
        .expect("can convert to bits");
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

    use crate::Amount;

    use super::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(1))]
        #[test]
        fn multiply_and_round(
            a_int in any::<u64>(),
            a_frac in any::<u64>(),
            b_int in any::<u64>(),
            b_frac in any::<u64>(),
        ) {
            let a = U128x128(
                U256([a_frac.into(), a_int.into()])
            );
            let b = U128x128(
                U256([b_frac.into(), b_int.into()])
            );

            let result = a.checked_mul(&b);

            let expected_c = result.expect("result should not overflow");
            let rounded_down_c = expected_c.round_down();

            let circuit = TestMultiplicationCircuit {
                a,
                b,
                c: expected_c,
                rounded_down_c,
            };

            let (pk, vk) = TestMultiplicationCircuit::generate_test_parameters();
            let mut rng = OsRng;

            let proof = Groth16::<Bls12_377, LibsnarkReduction>::prove(&pk, circuit, &mut rng)
            .expect("should be able to form proof");

            let mut pi = Vec::new();
            pi.extend_from_slice(&expected_c.to_field_elements().unwrap());
            pi.extend_from_slice(&rounded_down_c.to_field_elements().unwrap());
            let proof_result = Groth16::<Bls12_377, LibsnarkReduction>::verify(
                &vk,
                &pi,
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
        pub rounded_down_c: U128x128,
    }

    impl ConstraintSynthesizer<Fq> for TestMultiplicationCircuit {
        fn generate_constraints(
            self,
            cs: ConstraintSystemRef<Fq>,
        ) -> ark_relations::r1cs::Result<()> {
            let a_var = U128x128Var::new_witness(cs.clone(), || Ok(self.a))?;
            let b_var = U128x128Var::new_witness(cs.clone(), || Ok(self.b))?;
            let c_public_var = U128x128Var::new_input(cs.clone(), || Ok(self.c))?;
            let c_public_rounded_down_var = U128x128Var::new_input(cs, || Ok(self.rounded_down_c))?;
            let c_var = a_var.clone().checked_mul(&b_var)?;
            c_var.enforce_equal(&c_public_var)?;
            let c_rounded_down = c_var.clone().round_down();
            c_rounded_down.enforce_equal(&c_public_rounded_down_var)?;

            // Also check that a < c
            a_var.enforce_cmp(&c_var, std::cmp::Ordering::Less)?;

            // Also check that c > a
            c_var.enforce_cmp(&a_var, std::cmp::Ordering::Greater)?;
            Ok(())
        }
    }

    impl TestMultiplicationCircuit {
        fn generate_test_parameters() -> (ProvingKey<Bls12_377>, VerifyingKey<Bls12_377>) {
            let num: [u8; 32] = [0u8; 32];
            let a = U128x128::from_bytes(num);
            let b = U128x128::from_bytes(num);
            let c = a.checked_mul(&b).unwrap();
            let rounded_down_c = c.round_down();
            let circuit = TestMultiplicationCircuit {
                a,
                b,
                c,
                rounded_down_c,
            };
            let (pk, vk) = Groth16::<Bls12_377, LibsnarkReduction>::circuit_specific_setup(
                circuit, &mut OsRng,
            )
            .expect("can perform circuit specific setup");
            (pk, vk)
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(5))]
        #[test]
        fn add(
            a_int in any::<u64>(),
            a_frac in any::<u128>(),
            b_int in any::<u64>(),
            b_frac in any::<u128>(),
        ) {
            let a = U128x128(
                U256([a_frac, a_int.into()])
            );
            let b = U128x128(
                U256([b_frac, b_int.into()])
            );
            let result = a.checked_add(&b);

            if result.is_err() {
                // If the result overflowed, then we can't construct a valid proof.
                return Ok(())
            }
            let expected_c = result.expect("result should not overflow");

            let circuit = TestAdditionCircuit {
                a,
                b,
                c: expected_c,
            };

            let (pk, vk) = TestAdditionCircuit::generate_test_parameters();
            let mut rng = OsRng;

            let proof = Groth16::<Bls12_377, LibsnarkReduction>::prove(&pk, circuit, &mut rng)
            .expect("should be able to form proof");

            let proof_result = Groth16::<Bls12_377, LibsnarkReduction>::verify(
                &vk,
                &expected_c.to_field_elements().unwrap(),
                &proof,
            );
            assert!(proof_result.is_ok());
        }
    }

    #[test]
    fn max_u64_addition() {
        let a = U128x128(U256([u64::MAX as u128, 0]));
        let b = U128x128(U256([u64::MAX as u128, 0]));

        let result = a.checked_add(&b);

        let expected_c = result.expect("result should not overflow");

        let circuit = TestAdditionCircuit {
            a,
            b,
            c: expected_c,
        };

        let (pk, vk) = TestAdditionCircuit::generate_test_parameters();
        let mut rng = OsRng;

        let proof = Groth16::<Bls12_377, LibsnarkReduction>::prove(&pk, circuit, &mut rng)
            .expect("should be able to form proof");

        let proof_result = Groth16::<Bls12_377, LibsnarkReduction>::verify(
            &vk,
            &expected_c.to_field_elements().unwrap(),
            &proof,
        );
        assert!(proof_result.is_ok());
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
            let c_public_var = U128x128Var::new_input(cs, || Ok(self.c))?;
            let c_var = a_var.checked_add(&b_var)?;
            c_var.enforce_equal(&c_public_var)?;
            Ok(())
        }
    }

    impl TestAdditionCircuit {
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

    #[test]
    fn max_division() {
        let b = U128x128(U256([0, 1]));
        let a = U128x128(U256([u128::MAX, u128::MAX]));

        let result = a.checked_div(&b);

        let expected_c = result.expect("result should not overflow");
        dbg!(expected_c);

        let circuit = TestDivisionCircuit {
            a,
            b,
            c: expected_c,
        };

        let (pk, vk) = TestDivisionCircuit::generate_test_parameters();
        let mut rng = OsRng;

        let proof = Groth16::<Bls12_377, LibsnarkReduction>::prove(&pk, circuit, &mut rng)
            .expect("should be able to form proof");

        let proof_result = Groth16::<Bls12_377, LibsnarkReduction>::verify(
            &vk,
            &expected_c.to_field_elements().unwrap(),
            &proof,
        );
        assert!(proof_result.is_ok());
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10))]
        #[test]
        fn division(
            a_int in any::<u64>(),
            a_frac in any::<u64>(),
            b_int in any::<u128>(),
            b_frac in any::<u128>(),
        ) {
            let a = U128x128(
                U256([a_frac.into(), a_int.into()])
            );
            let b = U128x128(
                U256([b_frac, b_int])
            );

            // We can't divide by zero
            if b_int == 0 {
                return Ok(())
            }

            let result = a.checked_div(&b);

            let expected_c = result.expect("result should not overflow");

            let circuit = TestDivisionCircuit {
                a,
                b,
                c: expected_c,
            };

            let (pk, vk) = TestDivisionCircuit::generate_test_parameters();
            let mut rng = OsRng;

            let proof = Groth16::<Bls12_377, LibsnarkReduction>::prove(&pk, circuit, &mut rng)
            .expect("should be able to form proof");

            let proof_result = Groth16::<Bls12_377, LibsnarkReduction>::verify(
                &vk,
                &expected_c.to_field_elements().unwrap(),
                &proof,
            );
            assert!(proof_result.is_ok());
        }
    }

    struct TestDivisionCircuit {
        a: U128x128,
        b: U128x128,

        // c = a / b
        pub c: U128x128,
    }

    impl ConstraintSynthesizer<Fq> for TestDivisionCircuit {
        fn generate_constraints(
            self,
            cs: ConstraintSystemRef<Fq>,
        ) -> ark_relations::r1cs::Result<()> {
            let a_var = U128x128Var::new_witness(cs.clone(), || Ok(self.a))?;
            let b_var = U128x128Var::new_witness(cs.clone(), || Ok(self.b))?;
            let c_public_var = U128x128Var::new_input(cs.clone(), || Ok(self.c))?;
            let c_var = a_var.checked_div(&b_var, cs)?;
            c_var.enforce_equal(&c_public_var)?;
            Ok(())
        }
    }

    impl TestDivisionCircuit {
        fn generate_test_parameters() -> (ProvingKey<Bls12_377>, VerifyingKey<Bls12_377>) {
            let num: [u8; 32] = [1u8; 32];
            let a = U128x128::from_bytes(num);
            let b = U128x128::from_bytes(num);
            let circuit = TestDivisionCircuit {
                a,
                b,
                c: a.checked_div(&b).unwrap(),
            };
            let (pk, vk) = Groth16::<Bls12_377, LibsnarkReduction>::circuit_specific_setup(
                circuit, &mut OsRng,
            )
            .expect("can perform circuit specific setup");
            (pk, vk)
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(5))]
        #[test]
        fn compare(
            a_int in any::<u64>(),
            c_int in any::<u64>(),
        ) {
            // a < b
            let a =
                if a_int == u64::MAX {
                    U128x128::from(a_int - 1)
                } else {
                    U128x128::from(a_int)
                };
            let b = (a + U128x128::from(1u64)).expect("should not overflow");
            // c > d
            let c =
                if c_int == 0 {
                    U128x128::from(c_int + 1)
                } else {
                    U128x128::from(c_int)
                };
            let d = (c - U128x128::from(1u64)).expect("should not underflow");

            let circuit = TestComparisonCircuit {
                a,
                b,
                c,
                d,
            };

            let (pk, vk) = TestComparisonCircuit::generate_test_parameters();
            let mut rng = OsRng;

            let proof = Groth16::<Bls12_377, LibsnarkReduction>::prove(&pk, circuit, &mut rng)
            .expect("should be able to form proof");

            let proof_result = Groth16::<Bls12_377, LibsnarkReduction>::verify(
                &vk,
                &[],
                &proof,
            );
            assert!(proof_result.is_ok());
        }
    }

    struct TestComparisonCircuit {
        a: U128x128,
        b: U128x128,
        c: U128x128,
        d: U128x128,
    }

    impl ConstraintSynthesizer<Fq> for TestComparisonCircuit {
        fn generate_constraints(
            self,
            cs: ConstraintSystemRef<Fq>,
        ) -> ark_relations::r1cs::Result<()> {
            // a < b
            let a_var = U128x128Var::new_witness(cs.clone(), || Ok(self.a))?;
            let b_var = U128x128Var::new_witness(cs.clone(), || Ok(self.b))?;
            a_var.enforce_cmp(&b_var, std::cmp::Ordering::Less)?;
            // c > d
            let c_var = U128x128Var::new_witness(cs.clone(), || Ok(self.c))?;
            let d_var = U128x128Var::new_witness(cs, || Ok(self.d))?;
            c_var.enforce_cmp(&d_var, std::cmp::Ordering::Greater)?;

            Ok(())
        }
    }

    impl TestComparisonCircuit {
        fn generate_test_parameters() -> (ProvingKey<Bls12_377>, VerifyingKey<Bls12_377>) {
            let num: [u8; 32] = [0u8; 32];
            let a = U128x128::from_bytes(num);
            let b = U128x128::from_bytes(num);
            let c = U128x128::from_bytes(num);
            let d = U128x128::from_bytes(num);
            let circuit = TestComparisonCircuit { a, b, c, d };
            let (pk, vk) = Groth16::<Bls12_377, LibsnarkReduction>::circuit_specific_setup(
                circuit, &mut OsRng,
            )
            .expect("can perform circuit specific setup");
            (pk, vk)
        }
    }

    struct TestGreaterInvalidComparisonCircuit {
        a: U128x128,
        b: U128x128,
    }

    impl ConstraintSynthesizer<Fq> for TestGreaterInvalidComparisonCircuit {
        fn generate_constraints(
            self,
            cs: ConstraintSystemRef<Fq>,
        ) -> ark_relations::r1cs::Result<()> {
            // In reality a < b, but we're asserting that a > b here (should panic)
            let a_var = U128x128Var::new_witness(cs.clone(), || Ok(self.a))?;
            let b_var = U128x128Var::new_witness(cs, || Ok(self.b))?;
            a_var.enforce_cmp(&b_var, std::cmp::Ordering::Greater)?;

            Ok(())
        }
    }

    impl TestGreaterInvalidComparisonCircuit {
        fn generate_test_parameters(
        ) -> Result<(ProvingKey<Bls12_377>, VerifyingKey<Bls12_377>), SynthesisError> {
            let num: [u8; 32] = [0u8; 32];
            let a = U128x128::from_bytes(num);
            let b = U128x128::from_bytes(num);
            let circuit = TestGreaterInvalidComparisonCircuit { a, b };
            Groth16::<Bls12_377, LibsnarkReduction>::circuit_specific_setup(circuit, &mut OsRng)
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(5))]
        #[should_panic]
        #[test]
        fn invalid_greater_compare(
            a_int in any::<u128>(),
        ) {
            // a < b
            let a =
                if a_int == u128::MAX {
                    U128x128::from(a_int - 1)
                } else {
                    U128x128::from(a_int)
                };
            let b = (a + U128x128::from(1u64)).expect("should not overflow");

            let circuit = TestGreaterInvalidComparisonCircuit {
                a,
                b,
            };

            let (pk, vk) = TestGreaterInvalidComparisonCircuit::generate_test_parameters().expect("can perform setup");
            let mut rng = OsRng;

            let proof = Groth16::<Bls12_377, LibsnarkReduction>::prove(&pk, circuit, &mut rng)
            .expect("in debug mode only, we assert that the circuit is satisfied, so we will panic here");

            let proof_result = Groth16::<Bls12_377, LibsnarkReduction>::verify(
                &vk,
                &[],
                &proof,
            ).expect("in release mode, we will be able to construct the proof, so we can unwrap the result");

            // We want the same behavior in release or debug mode, so we will panic if the proof does not verify.
            if !proof_result {
                panic!("should not be able to verify proof");
            }
        }
    }

    struct TestLessInvalidComparisonCircuit {
        c: U128x128,
        d: U128x128,
    }

    impl ConstraintSynthesizer<Fq> for TestLessInvalidComparisonCircuit {
        fn generate_constraints(
            self,
            cs: ConstraintSystemRef<Fq>,
        ) -> ark_relations::r1cs::Result<()> {
            // In reality c > d, but we're asserting that c < d here (should panic)
            let c_var = U128x128Var::new_witness(cs.clone(), || Ok(self.c))?;
            let d_var = U128x128Var::new_witness(cs, || Ok(self.d))?;
            c_var.enforce_cmp(&d_var, std::cmp::Ordering::Less)?;

            Ok(())
        }
    }

    impl TestLessInvalidComparisonCircuit {
        fn generate_test_parameters(
        ) -> Result<(ProvingKey<Bls12_377>, VerifyingKey<Bls12_377>), SynthesisError> {
            let num: [u8; 32] = [0u8; 32];
            let c = U128x128::from_bytes(num);
            let d = U128x128::from_bytes(num);
            let circuit = TestLessInvalidComparisonCircuit { c, d };
            Groth16::<Bls12_377, LibsnarkReduction>::circuit_specific_setup(circuit, &mut OsRng)
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(5))]
        #[should_panic]
        #[test]
        fn invalid_less_compare(
            c_int in any::<u128>(),
        ) {
            // c > d
            let c =
                if c_int == 0 {
                    U128x128::from(c_int + 1)
                } else {
                    U128x128::from(c_int)
                };
            let d = (c - U128x128::from(1u64)).expect("should not underflow");

            let circuit = TestLessInvalidComparisonCircuit {
                c,
                d,
            };

            let (pk, vk) = TestLessInvalidComparisonCircuit::generate_test_parameters().expect("can perform setup");
            let mut rng = OsRng;

            let proof = Groth16::<Bls12_377, LibsnarkReduction>::prove(&pk, circuit, &mut rng)
            .expect("in debug mode only, we assert that the circuit is satisfied, so we will panic here");

            let proof_result = Groth16::<Bls12_377, LibsnarkReduction>::verify(
                &vk,
                &[],
                &proof,
            ).expect("in release mode, we will be able to construct the proof, so we can unwrap the result");

            // We want the same behavior in release or debug mode, so we will panic if the proof does not verify.
            if !proof_result {
                panic!("should not be able to verify proof");
            }
        }
    }

    #[should_panic]
    #[test]
    fn regression_invalid_less_compare() {
        // c > d in reality, the circuit will attempt to prove c < d (should panic)
        let c = U128x128::from(354389783742u64);
        let d = U128x128::from(17u64);

        let circuit = TestLessInvalidComparisonCircuit { c, d };

        let (pk, vk) = TestLessInvalidComparisonCircuit::generate_test_parameters()
            .expect("can perform setup");
        let mut rng = OsRng;

        let proof = Groth16::<Bls12_377, LibsnarkReduction>::prove(&pk, circuit, &mut rng).expect(
            "in debug mode only, we assert that the circuit is satisfied, so we will panic here",
        );

        let proof_result = Groth16::<Bls12_377, LibsnarkReduction>::verify(&vk, &[], &proof)
            .expect(
            "in release mode, we will be able to construct the proof, so we can unwrap the result",
        );

        // We want the same behavior in release or debug mode, so we will panic if the proof does not verify.
        if !proof_result {
            panic!("should not be able to verify proof");
        }
    }

    proptest! {
            #![proptest_config(ProptestConfig::with_cases(5))]
        #[test]
        fn round_down_to_amount(
            a_int in any::<u128>(),
            a_frac in any::<u128>(),
            ) {
            let a = U128x128(
                U256([a_frac, a_int])
            );

            let expected_c = a.round_down().try_into().expect("should be able to round down OOC");

            let circuit = TestRoundDownCircuit {
                a,
                c: expected_c,
            };

            let (pk, vk) = TestRoundDownCircuit::generate_test_parameters();
            let mut rng = OsRng;

            let proof = Groth16::<Bls12_377, LibsnarkReduction>::prove(&pk, circuit, &mut rng)
                .expect("should be able to form proof");

            let proof_result = Groth16::<Bls12_377, LibsnarkReduction>::verify(
                &vk,
                &expected_c.to_field_elements().unwrap(),
                &proof,
            );
            assert!(proof_result.is_ok());
        }
    }

    struct TestRoundDownCircuit {
        a: U128x128,

        // `c` is expected to be `a` rounded down to an `Amount`
        pub c: Amount,
    }

    impl ConstraintSynthesizer<Fq> for TestRoundDownCircuit {
        fn generate_constraints(
            self,
            cs: ConstraintSystemRef<Fq>,
        ) -> ark_relations::r1cs::Result<()> {
            let a_var = U128x128Var::new_witness(cs.clone(), || Ok(self.a))?;
            let c_public_var = AmountVar::new_input(cs, || Ok(self.c))?;
            let c_var = a_var.round_down_to_amount()?;
            c_var.enforce_equal(&c_public_var)?;
            Ok(())
        }
    }

    impl TestRoundDownCircuit {
        fn generate_test_parameters() -> (ProvingKey<Bls12_377>, VerifyingKey<Bls12_377>) {
            let num: [u8; 32] = [0u8; 32];
            let a = U128x128::from_bytes(num);
            let c: Amount = a
                .round_down()
                .try_into()
                .expect("should be able to round down OOC");
            let circuit = TestRoundDownCircuit { a, c };
            let (pk, vk) = Groth16::<Bls12_377, LibsnarkReduction>::circuit_specific_setup(
                circuit, &mut OsRng,
            )
            .expect("can perform circuit specific setup");
            (pk, vk)
        }
    }
}
