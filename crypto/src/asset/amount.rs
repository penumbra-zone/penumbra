use crate::{Fq, Fr};
use penumbra_proto::{core::crypto::v1alpha1 as pb, Protobuf};
use serde::{Deserialize, Serialize};
use std::{fmt::Display, num::NonZeroU64, ops};

#[derive(Serialize, Deserialize, PartialEq, PartialOrd, Eq, Clone, Debug, Copy)]
#[serde(try_from = "pb::Amount", into = "pb::Amount")]
pub struct Amount {
    // TODO(ava): update to u128
    pub inner: u64,
}

impl From<Amount> for pb::Amount {
    fn from(a: Amount) -> Self {
        pb::Amount {
            inner: a.inner.to_le_bytes().to_vec(),
        }
    }
}

impl TryFrom<std::string::String> for Amount {
    type Error = anyhow::Error;

    fn try_from(s: std::string::String) -> Result<Self, Self::Error> {
        let inner = s.parse::<u64>()?;
        Ok(Amount { inner })
    }
}

impl TryFrom<pb::Amount> for Amount {
    type Error = anyhow::Error;

    fn try_from(amount: pb::Amount) -> Result<Self, Self::Error> {
        Ok(Amount {
            inner: u64::from_le_bytes(amount.inner.try_into().map_err(|_| {
                anyhow::anyhow!("could not deserialize Amount: input vec is not 8 bytes")
            })?),
        })
    }
}

impl Protobuf<pb::Amount> for Amount {}

impl Amount {
    pub fn zero() -> Self {
        Self { inner: 0 }
    }
    pub fn to_le_bytes(&self) -> [u8; 8] {
        self.inner.to_le_bytes()
    }
    pub fn from_le_bytes(bytes: [u8; 8]) -> Self {
        Self {
            inner: u64::from_le_bytes(bytes),
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

impl From<NonZeroU64> for Amount {
    fn from(n: NonZeroU64) -> Self {
        Self { inner: n.get() }
    }
}

impl From<Amount> for Fq {
    fn from(amount: Amount) -> Fq {
        Fq::from(amount.inner)
    }
}

impl From<u64> for Amount {
    fn from(amount: u64) -> Amount {
        Amount { inner: amount }
    }
}

impl From<u32> for Amount {
    fn from(amount: u32) -> Amount {
        Amount {
            inner: amount as u64,
        }
    }
}

impl From<Amount> for u32 {
    fn from(amount: Amount) -> u32 {
        amount.inner as u32
    }
}

impl From<i64> for Amount {
    fn from(amount: i64) -> Amount {
        Amount {
            inner: amount as u64,
        }
    }
}

impl From<Amount> for u128 {
    fn from(amount: Amount) -> u128 {
        amount.inner as u128
    }
}

impl From<Amount> for i64 {
    fn from(amount: Amount) -> i64 {
        amount.inner as i64
    }
}

impl From<Amount> for u64 {
    fn from(amount: Amount) -> u64 {
        amount.inner
    }
}

impl From<Amount> for Fr {
    fn from(amount: Amount) -> Fr {
        Fr::from(amount.inner)
    }
}

impl From<[u8; 8]> for Amount {
    fn from(bytes: [u8; 8]) -> Amount {
        Amount {
            inner: u64::from_le_bytes(bytes),
        }
    }
}

impl TryFrom<Vec<u8>> for Amount {
    type Error = anyhow::Error;

    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
        if bytes.len() != 8 {
            return Err(anyhow::anyhow!(
                "could not deserialize Amount: input vec is not 8 bytes"
            ));
        }
        let mut array = [0u8; 8];
        array.copy_from_slice(&bytes);
        Ok(Amount {
            inner: u64::from_le_bytes(array),
        })
    }
}
