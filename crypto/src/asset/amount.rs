use crate::{Fq, Fr};
use anyhow::anyhow;
use penumbra_proto::{core::crypto::v1alpha1 as pb, Protobuf};
use serde::{Deserialize, Serialize};
use std::{fmt::Display, num::NonZeroU128, ops};

#[derive(Serialize, Deserialize, PartialEq, PartialOrd, Eq, Clone, Debug, Copy)]
#[serde(try_from = "pb::Amount", into = "pb::Amount")]
pub struct Amount {
    inner: u128,
}

impl Amount {
    pub fn value(&self) -> u64 {
        self.inner as u64
    }
}

impl From<Amount> for pb::Amount {
    fn from(a: Amount) -> Self {
        let value = a.inner;
        if value <= u64::MAX as u128 {
            pb::Amount {
                lo: value.try_into().unwrap(),
                hi: 0,
            }
        } else {
            let bytes = value.to_le_bytes();
            let mut lo_bytes: [u8; 8] = [0; 8];
            let mut hi_bytes: [u8; 8] = [0; 8];

            for i in 0..16 {
                if i < 8 {
                    lo_bytes[i] = bytes[i];
                } else {
                    hi_bytes[i % 8] = bytes[i];
                }
            }

            let lo = u64::from_le_bytes(lo_bytes);
            let hi = u64::from_be_bytes(hi_bytes);
            pb::Amount { lo, hi }
        }
    }
}

impl TryFrom<pb::Amount> for Amount {
    type Error = anyhow::Error;

    fn try_from(amount: pb::Amount) -> Result<Self, Self::Error> {
        let lo = amount.lo as u128;
        let hi = amount.hi as u128;
        // We want to encode the hi/lo bytes as follow:
        //            hi: u64                          lo: u64
        // ┌───┬───┬───┬───┬───┬───┬───┬───┐ ┌───┬───┬───┬───┬───┬───┬───┬───┐
        // │   │   │   │   │   │   │   │   │ │   │   │   │   │   │   │   │   │
        // └───┴───┴───┴───┴───┴───┴───┴───┘ └───┴───┴───┴───┴───┴───┴───┴───┘
        //   15  14  13  12  11  10  9   8     7   6   5   4   3   2   1   0
        //
        // To achieve this, we shift hi 8 bytes to the right, and then add the
        // lower order bytes (lo).

        let inner = u128::from_be(hi) + lo;
        Ok(Amount { inner })
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
    fn encode_decode_u64() {
        let value = u64::MAX as u128;
        assert_eq!(value, encode_decode(value))
    }
}

impl Protobuf<pb::Amount> for Amount {}

impl Amount {
    pub fn zero() -> Self {
        Self { inner: 0 }
    }
    pub fn to_le_bytes(&self) -> [u8; 16] {
        self.inner.to_le_bytes()
    }

    pub fn from_le_bytes(bytes: [u8; 16]) -> Self {
        Self {
            inner: u128::from_le_bytes(bytes),
        }
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
        amount.inner as u128
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

impl From<[u8; 16]> for Amount {
    fn from(bytes: [u8; 16]) -> Amount {
        Amount {
            inner: u128::from_le_bytes(bytes),
        }
    }
}

impl TryFrom<Vec<u8>> for Amount {
    type Error = anyhow::Error;

    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
        if bytes.len() != 16 {
            return Err(anyhow::anyhow!(
                "could not deserialize Amount: input vec is not 16 bytes"
            ));
        }
        let mut array = [0u8; 16];
        array.copy_from_slice(&bytes);
        Ok(Amount {
            inner: u128::from_le_bytes(array),
        })
    }
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
            .map_err(|_| anyhow!("failed conversion!"))
    }
}

impl From<[u8; 8]> for Amount {
    fn from(bytes: [u8; 8]) -> Amount {
        Amount {
            inner: u64::from_le_bytes(bytes) as u128,
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

impl From<Amount> for u32 {
    fn from(amount: Amount) -> u32 {
        amount.inner as u32
    }
}
