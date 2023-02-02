use ark_r1cs_std::prelude::*;
use ark_relations::r1cs::SynthesisError;
use penumbra_proto::{core::crypto::v1alpha1 as pb, DomainType};
use serde::{Deserialize, Serialize};
use std::{fmt::Display, iter::Sum, num::NonZeroU128, ops};

use crate::{fixpoint::U128x128, Fq, Fr};
use decaf377::r1cs::FqVar;

#[derive(Serialize, Deserialize, PartialEq, PartialOrd, Eq, Clone, Debug, Copy)]
#[serde(try_from = "pb::Amount", into = "pb::Amount")]
pub struct Amount {
    inner: u128,
}

impl Amount {
    pub fn value(&self) -> u64 {
        self.inner as u64
    }

    pub fn zero() -> Self {
        Self { inner: 0 }
    }

    // We need fixed length encoding to produce encrypted `Note`s.
    pub fn to_le_bytes(&self) -> [u8; 16] {
        self.inner.to_le_bytes()
    }

    pub fn from_le_bytes(bytes: [u8; 16]) -> Amount {
        Amount {
            inner: u128::from_le_bytes(bytes),
        }
    }
}

#[derive(Clone)]
pub struct AmountVar {
    pub amount: FqVar,
}

impl AmountVar {
    pub fn negate(&self) -> Result<Self, SynthesisError> {
        Ok(Self {
            amount: self.amount.negate()?,
        })
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
        match mode {
            AllocationMode::Constant => unimplemented!(),
            AllocationMode::Input => unimplemented!(),
            AllocationMode::Witness => {
                let inner_amount_var = FqVar::new_witness(cs, || Ok(Fq::from(amount)))?;
                Ok(Self {
                    amount: inner_amount_var,
                })
            }
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

impl From<Amount> for u64 {
    fn from(amount: Amount) -> u64 {
        amount.inner as u64
    }
}

impl TryFrom<Amount> for i64 {
    type Error = anyhow::Error;
    fn try_from(value: Amount) -> Result<Self, Self::Error> {
        value
            .inner
            .try_into()
            .map_err(|_| anyhow::anyhow!("failed conversion!"))
    }
}

impl From<u32> for Amount {
    fn from(amount: u32) -> Amount {
        Amount {
            inner: amount as u128,
        }
    }
}

impl From<Amount> for u32 {
    fn from(amount: Amount) -> u32 {
        amount.inner as u32
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

impl ops::Sub<Amount> for Amount {
    type Output = Amount;

    fn sub(self, rhs: Amount) -> Amount {
        Amount {
            inner: self.inner - rhs.inner,
        }
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

impl TryFrom<U128x128> for Amount {
    type Error = <u128 as TryFrom<U128x128>>::Error;
    fn try_from(value: U128x128) -> Result<Self, Self::Error> {
        Ok(Amount {
            inner: value.try_into()?,
        })
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
    use penumbra_proto::core::crypto::v1alpha1 as pb;
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
