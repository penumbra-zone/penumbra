use ethnum::U256;

use super::U128x128;

impl From<u8> for U128x128 {
    fn from(value: u8) -> Self {
        Self(U256::from_words(value.into(), 0))
    }
}

impl From<u16> for U128x128 {
    fn from(value: u16) -> Self {
        Self(U256::from_words(value.into(), 0))
    }
}

impl From<u32> for U128x128 {
    fn from(value: u32) -> Self {
        Self(U256::from_words(value.into(), 0))
    }
}

impl From<u64> for U128x128 {
    fn from(value: u64) -> Self {
        Self(U256::from_words(value.into(), 0))
    }
}

impl From<u128> for U128x128 {
    fn from(value: u128) -> Self {
        Self(U256::from_words(value, 0))
    }
}

impl TryFrom<U128x128> for u128 {
    type Error = super::Error;
    fn try_from(value: U128x128) -> Result<Self, Self::Error> {
        match value.is_integral() {
            true => Ok(value.0.into_words().0),
            false => Err(super::Error::NonIntegral { value }),
        }
    }
}

impl From<[u8; 32]> for U128x128 {
    fn from(value: [u8; 32]) -> Self {
        Self::from_bytes(value)
    }
}

impl From<U128x128> for [u8; 32] {
    fn from(value: U128x128) -> Self {
        value.to_bytes()
    }
}

impl TryFrom<&[u8]> for U128x128 {
    type Error = super::Error;
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Ok(<[u8; 32]>::try_from(value)
            .map_err(|_| super::Error::SliceLength(value.len()))?
            .into())
    }
}

impl From<U128x128> for Vec<u8> {
    fn from(value: U128x128) -> Self {
        value.to_bytes().to_vec()
    }
}
