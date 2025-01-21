use ark_ff::{BigInteger, PrimeField, ToConstraintField};
use ark_r1cs_std::{prelude::*, uint64::UInt64};
use ark_relations::r1cs::{ConstraintSystemRef, SynthesisError};
use decaf377::{Fq, Fr};
use penumbra_sdk_proto::{penumbra::core::num::v1 as pb, DomainType};
use serde::{Deserialize, Serialize};
use std::{fmt::Display, iter::Sum, num::NonZeroU128, ops};

use crate::fixpoint::{bit_constrain, U128x128, U128x128Var};
use decaf377::r1cs::FqVar;

#[derive(Serialize, Default, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
#[serde(try_from = "pb::Amount", into = "pb::Amount")]
pub struct Amount {
    inner: u128,
}

impl std::fmt::Debug for Amount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl Amount {
    pub fn value(&self) -> u128 {
        self.inner
    }

    pub fn zero() -> Self {
        Self { inner: 0 }
    }

    // We need fixed length encoding to produce encrypted `Note`s.
    pub fn to_le_bytes(&self) -> [u8; 16] {
        self.inner.to_le_bytes()
    }

    pub fn to_be_bytes(&self) -> [u8; 16] {
        self.inner.to_be_bytes()
    }

    pub fn from_le_bytes(bytes: [u8; 16]) -> Amount {
        Amount {
            inner: u128::from_le_bytes(bytes),
        }
    }

    pub fn from_be_bytes(bytes: [u8; 16]) -> Amount {
        Amount {
            inner: u128::from_be_bytes(bytes),
        }
    }

    pub fn checked_sub(&self, rhs: &Self) -> Option<Self> {
        self.inner
            .checked_sub(rhs.inner)
            .map(|inner| Self { inner })
    }

    pub fn checked_add(&self, rhs: &Self) -> Option<Self> {
        self.inner
            .checked_add(rhs.inner)
            .map(|inner| Self { inner })
    }

    pub fn checked_mul(&self, rhs: &Self) -> Option<Self> {
        self.inner
            .checked_mul(rhs.inner)
            .map(|inner| Self { inner })
    }

    pub fn saturating_add(&self, rhs: &Self) -> Self {
        Self {
            inner: self.inner.saturating_add(rhs.inner),
        }
    }

    pub fn saturating_sub(&self, rhs: &Self) -> Self {
        Self {
            inner: self.inner.saturating_sub(rhs.inner),
        }
    }
}

impl ops::Not for Amount {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self { inner: !self.inner }
    }
}

#[derive(Clone)]
pub struct AmountVar {
    pub amount: FqVar,
}

impl ToConstraintField<Fq> for Amount {
    fn to_field_elements(&self) -> Option<Vec<Fq>> {
        let mut elements = Vec::new();
        elements.extend_from_slice(&[Fq::from(self.inner)]);
        Some(elements)
    }
}

/// Return a boolean constraint indicating if the FqVar can be represented using n bits
pub fn is_bit_constrained(
    cs: ConstraintSystemRef<Fq>,
    value: FqVar,
    n: usize,
) -> Result<Boolean<Fq>, SynthesisError> {
    let inner = value.value().unwrap_or(Fq::from(1u64));

    // Get only first n bits based on that value (OOC)
    let inner_bigint = inner.into_bigint();
    let bits = &inner_bigint.to_bits_le()[0..n];

    // Allocate Boolean vars for first n bits
    let mut boolean_constraints = Vec::new();
    for bit in bits {
        let boolean = Boolean::new_witness(cs.clone(), || Ok(bit))?;
        boolean_constraints.push(boolean);
    }

    // Construct an FqVar from those n Boolean constraints
    let constructed_fqvar = Boolean::<Fq>::le_bits_to_fp_var(&boolean_constraints.to_bits_le()?)
        .expect("can convert to bits");
    constructed_fqvar.is_eq(&value)
}

impl AmountVar {
    pub fn negate(&self) -> Result<Self, SynthesisError> {
        Ok(Self {
            amount: self.amount.negate()?,
        })
    }

    pub fn quo_rem(
        &self,
        divisor_var: &AmountVar,
    ) -> Result<(AmountVar, AmountVar), SynthesisError> {
        let current_amount_bytes: [u8; 16] = self.amount.value().unwrap_or_default().to_bytes()
            [0..16]
            .try_into()
            .expect("amounts should fit in 16 bytes");
        let current_amount = u128::from_le_bytes(current_amount_bytes);
        let divisor_bytes: [u8; 16] = divisor_var.amount.value().unwrap_or_default().to_bytes()
            [0..16]
            .try_into()
            .expect("amounts should fit in 16 bytes");
        let divisor = u128::from_le_bytes(divisor_bytes);

        // Out of circuit
        let quo = current_amount.checked_div(divisor).unwrap_or(0);
        let rem = current_amount.checked_rem(divisor).unwrap_or(0);

        // Add corresponding in-circuit variables
        let quo_var = AmountVar::new_witness(self.cs(), || Ok(Fq::from(quo)))?;
        let rem_var = AmountVar::new_witness(self.cs(), || Ok(Fq::from(rem)))?;

        // Constrain either quo_var or divisor_var to be 64 bits to guard against overflow
        let q_is_64_bits = is_bit_constrained(self.cs(), quo_var.amount.clone(), 64)?;
        let d_is_64_bits = is_bit_constrained(self.cs(), divisor_var.amount.clone(), 64)?;
        let q_or_d_is_64_bits = q_is_64_bits.or(&d_is_64_bits)?;
        q_or_d_is_64_bits.enforce_equal(&Boolean::constant(true))?;

        // Constrain: numerator = quo * divisor + rem
        let numerator_var = quo_var.clone() * divisor_var.clone() + rem_var.clone();
        self.enforce_equal(&numerator_var)?;

        // In this stanza we constrain: 0 <= rem < divisor.
        //
        // We do not need to explicitly constrain 0 <= rem, as that is done
        // inside the `FqVar::enforce_cmp` function, which verifies the inputs are
        // of size <(p-1)/2.
        //
        // See: https://docs.rs/ark-r1cs-std/latest/ark_r1cs_std/fields/fp/enum.FpVar.html#method.enforce_cmp
        //
        // Constrain: 0 <= rem < divisor
        rem_var
            .amount
            .enforce_cmp(&divisor_var.amount, core::cmp::Ordering::Less, false)?;
        // As above, `FpVar::enforce_cmp` requires that the amounts have size <(p-1)/2 which is
        // true for amounts as they are 128 bits at most.

        // We do not need to check the divisor is non-zero, as that is already
        // enforced by 0 <= r < d above.

        Ok((quo_var, rem_var))
    }
}

impl AllocVar<Amount, Fq> for AmountVar {
    fn new_variable<T: std::borrow::Borrow<Amount>>(
        cs: impl Into<ark_relations::r1cs::Namespace<Fq>>,
        f: impl FnOnce() -> Result<T, SynthesisError>,
        mode: ark_r1cs_std::prelude::AllocationMode,
    ) -> Result<Self, SynthesisError> {
        let ns = cs.into();
        let cs = ns.cs();
        let amount: Amount = *f()?.borrow();
        let inner_amount_var = FqVar::new_variable(cs, || Ok(Fq::from(amount)), mode)?;
        // Check the amounts are 128 bits maximum
        let _ = bit_constrain(inner_amount_var.clone(), 128);
        Ok(Self {
            amount: inner_amount_var,
        })
    }
}

impl AllocVar<Fq, Fq> for AmountVar {
    fn new_variable<T: std::borrow::Borrow<Fq>>(
        cs: impl Into<ark_relations::r1cs::Namespace<Fq>>,
        f: impl FnOnce() -> Result<T, SynthesisError>,
        mode: ark_r1cs_std::prelude::AllocationMode,
    ) -> Result<Self, SynthesisError> {
        let ns = cs.into();
        let cs = ns.cs();
        let amount: Fq = *f()?.borrow();
        let inner_amount_var = FqVar::new_variable(cs, || Ok(amount), mode)?;
        // Check the amounts are 128 bits maximum
        let _ = bit_constrain(inner_amount_var.clone(), 128);
        Ok(Self {
            amount: inner_amount_var,
        })
    }
}

impl R1CSVar<Fq> for AmountVar {
    type Value = Amount;

    fn cs(&self) -> ark_relations::r1cs::ConstraintSystemRef<Fq> {
        self.amount.cs()
    }

    fn value(&self) -> Result<Self::Value, SynthesisError> {
        let amount_fq = self.amount.value()?;
        let amount_bytes = &amount_fq.to_bytes()[0..16];
        Ok(Amount::from_le_bytes(
            amount_bytes
                .try_into()
                .expect("should be able to fit in 16 bytes"),
        ))
    }
}

impl EqGadget<Fq> for AmountVar {
    fn is_eq(&self, other: &Self) -> Result<Boolean<Fq>, SynthesisError> {
        self.amount.is_eq(&other.amount)
    }
}

impl CondSelectGadget<Fq> for AmountVar {
    fn conditionally_select(
        cond: &Boolean<Fq>,
        true_value: &Self,
        false_value: &Self,
    ) -> Result<Self, SynthesisError> {
        Ok(Self {
            amount: FqVar::conditionally_select(cond, &true_value.amount, &false_value.amount)?,
        })
    }
}

impl std::ops::Add for AmountVar {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            amount: self.amount + rhs.amount,
        }
    }
}

impl std::ops::Sub for AmountVar {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            amount: self.amount - rhs.amount,
        }
    }
}

impl std::ops::Mul for AmountVar {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            amount: self.amount * rhs.amount,
        }
    }
}

impl From<Amount> for pb::Amount {
    fn from(a: Amount) -> Self {
        let lo = a.inner as u64;
        let hi = (a.inner >> 64) as u64;
        pb::Amount { lo, hi }
    }
}

impl TryFrom<pb::Amount> for Amount {
    type Error = anyhow::Error;

    fn try_from(amount: pb::Amount) -> Result<Self, Self::Error> {
        let lo = amount.lo as u128;
        let hi = amount.hi as u128;
        // `hi` and `lo` represent the high/low order bytes respectively.
        //
        // We want to decode `hi` and `lo` into a single `u128` of the form:
        //
        //            hi: u64                          lo: u64
        // ┌───┬───┬───┬───┬───┬───┬───┬───┐ ┌───┬───┬───┬───┬───┬───┬───┬───┐
        // │   │   │   │   │   │   │   │   │ │   │   │   │   │   │   │   │   │
        // └───┴───┴───┴───┴───┴───┴───┴───┘ └───┴───┴───┴───┴───┴───┴───┴───┘
        //   15  14  13  12  11  10  9   8     7   6   5   4   3   2   1   0
        //
        // To achieve this, we shift `hi` 8 bytes to the left:
        let shifted = hi << 64;
        // and then add the lower order bytes:
        let inner = shifted + lo;

        Ok(Amount { inner })
    }
}

impl TryFrom<std::string::String> for Amount {
    type Error = anyhow::Error;

    fn try_from(s: std::string::String) -> Result<Self, Self::Error> {
        let inner = s.parse::<u128>()?;
        Ok(Amount { inner })
    }
}

impl DomainType for Amount {
    type Proto = pb::Amount;
}

impl From<u64> for Amount {
    fn from(amount: u64) -> Amount {
        Amount {
            inner: amount as u128,
        }
    }
}

impl From<u32> for Amount {
    fn from(amount: u32) -> Amount {
        Amount {
            inner: amount as u128,
        }
    }
}

impl From<u16> for Amount {
    fn from(amount: u16) -> Amount {
        Amount {
            inner: amount as u128,
        }
    }
}

impl From<u8> for Amount {
    fn from(amount: u8) -> Amount {
        Amount {
            inner: amount as u128,
        }
    }
}

impl From<Amount> for f64 {
    fn from(amount: Amount) -> f64 {
        amount.inner as f64
    }
}

impl Display for Amount {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl ops::Add<Amount> for Amount {
    type Output = Amount;

    fn add(self, rhs: Amount) -> Amount {
        Amount {
            inner: self.inner + rhs.inner,
        }
    }
}

impl ops::AddAssign<Amount> for Amount {
    fn add_assign(&mut self, rhs: Amount) {
        self.inner += rhs.inner;
    }
}

impl ops::Sub<Amount> for Amount {
    type Output = Amount;

    fn sub(self, rhs: Amount) -> Amount {
        Amount {
            inner: self.inner - rhs.inner,
        }
    }
}

impl ops::SubAssign<Amount> for Amount {
    fn sub_assign(&mut self, rhs: Amount) {
        self.inner -= rhs.inner;
    }
}

impl ops::Rem<Amount> for Amount {
    type Output = Amount;

    fn rem(self, rhs: Amount) -> Amount {
        Amount {
            inner: self.inner % rhs.inner,
        }
    }
}

impl ops::Mul<Amount> for Amount {
    type Output = Amount;

    fn mul(self, rhs: Amount) -> Amount {
        Amount {
            inner: self.inner * rhs.inner,
        }
    }
}

impl ops::Div<Amount> for Amount {
    type Output = Amount;

    fn div(self, rhs: Amount) -> Amount {
        Amount {
            inner: self.inner / rhs.inner,
        }
    }
}

impl From<NonZeroU128> for Amount {
    fn from(n: NonZeroU128) -> Self {
        Self { inner: n.get() }
    }
}

impl From<Amount> for Fq {
    fn from(amount: Amount) -> Fq {
        Fq::from(amount.inner)
    }
}

impl From<Amount> for Fr {
    fn from(amount: Amount) -> Fr {
        Fr::from(amount.inner)
    }
}

impl From<u128> for Amount {
    fn from(amount: u128) -> Amount {
        Amount { inner: amount }
    }
}

impl From<Amount> for u128 {
    fn from(amount: Amount) -> u128 {
        amount.inner
    }
}

impl From<i128> for Amount {
    fn from(amount: i128) -> Amount {
        Amount {
            inner: amount as u128,
        }
    }
}

impl From<Amount> for i128 {
    fn from(amount: Amount) -> i128 {
        amount.inner as i128
    }
}

impl From<Amount> for U128x128 {
    fn from(amount: Amount) -> U128x128 {
        U128x128::from(amount.inner)
    }
}

impl From<&Amount> for U128x128 {
    fn from(value: &Amount) -> Self {
        (*value).into()
    }
}

impl TryFrom<U128x128> for Amount {
    type Error = <u128 as TryFrom<U128x128>>::Error;
    fn try_from(value: U128x128) -> Result<Self, Self::Error> {
        Ok(Amount {
            inner: value.try_into()?,
        })
    }
}

impl U128x128Var {
    pub fn from_amount_var(amount: AmountVar) -> Result<U128x128Var, SynthesisError> {
        let bits = amount.amount.to_bits_le()?;
        let limb_2 = UInt64::from_bits_le(&bits[0..64]);
        let limb_3 = UInt64::from_bits_le(&bits[64..128]);
        Ok(Self {
            limbs: [
                UInt64::constant(0u64),
                UInt64::constant(0u64),
                limb_2,
                limb_3,
            ],
        })
    }
}

impl From<U128x128Var> for AmountVar {
    fn from(value: U128x128Var) -> Self {
        let mut le_bits = Vec::new();
        le_bits.extend_from_slice(&value.limbs[2].to_bits_le()[..]);
        le_bits.extend_from_slice(&value.limbs[3].to_bits_le()[..]);
        Self {
            amount: Boolean::<Fq>::le_bits_to_fp_var(&le_bits[..]).expect("can convert to bits"),
        }
    }
}

impl Sum for Amount {
    fn sum<I: Iterator<Item = Amount>>(iter: I) -> Amount {
        iter.fold(Amount::zero(), |acc, x| acc + x)
    }
}

#[cfg(test)]
mod test {
    use crate::Amount;
    use penumbra_sdk_proto::penumbra::core::num::v1 as pb;
    use rand::RngCore;
    use rand_core::OsRng;

    fn encode_decode(value: u128) -> u128 {
        let amount = Amount { inner: value };
        let proto: pb::Amount = amount.into();
        Amount::try_from(proto).unwrap().inner
    }

    #[test]
    fn encode_decode_max() {
        let value = u128::MAX;
        assert_eq!(value, encode_decode(value))
    }

    #[test]
    fn encode_decode_zero() {
        let value = u128::MIN;
        assert_eq!(value, encode_decode(value))
    }

    #[test]
    fn encode_decode_right_border_bit() {
        let value: u128 = 1 << 64;
        assert_eq!(value, encode_decode(value))
    }

    #[test]
    fn encode_decode_left_border_bit() {
        let value: u128 = 1 << 63;
        assert_eq!(value, encode_decode(value))
    }

    #[test]
    fn encode_decode_random() {
        let mut rng = OsRng;
        let mut dest: [u8; 16] = [0; 16];
        rng.fill_bytes(&mut dest);
        let value: u128 = u128::from_le_bytes(dest);
        assert_eq!(value, encode_decode(value))
    }

    #[test]
    fn encode_decode_u64_max() {
        let value = u64::MAX as u128;
        assert_eq!(value, encode_decode(value))
    }

    #[test]
    fn encode_decode_random_lower_order_bytes() {
        let mut rng = OsRng;
        let lo = rng.next_u64() as u128;
        assert_eq!(lo, encode_decode(lo))
    }

    #[test]
    fn encode_decode_random_higher_order_bytes() {
        let mut rng = OsRng;
        let value = rng.next_u64();
        let hi = (value as u128) << 64;
        assert_eq!(hi, encode_decode(hi))
    }
}
