use crate::fixpoint;
use crate::fixpoint::U128x128;
use ethnum::U256;

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

impl From<U128x128> for f64 {
    fn from(value: U128x128) -> Self {
        // This is a hack but it seems to work mostly?
        // doesn't seem critical for it to be exact, there's no reverse conversion.
        let (hi, lo) = value.0.into_words();
        // binary repr of 2^128
        const BASE: u64 = 0x47f0000000000000;
        (hi as f64) + (lo as f64) / f64::from_bits(BASE)
    }
}

impl TryFrom<f64> for U128x128 {
    type Error = fixpoint::Error;

    fn try_from(value: f64) -> Result<U128x128, Self::Error> {
        if value < 0.0 {
            Err(fixpoint::Error::InvalidFloat64 { value })
        } else if value.is_infinite() {
            Err(fixpoint::Error::InvalidFloat64 { value })
        } else if value.is_nan() {
            Err(fixpoint::Error::InvalidFloat64 { value })
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

impl From<U128x128> for Vec<u8> {
    fn from(value: U128x128) -> Self {
        value.to_bytes().to_vec()
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
