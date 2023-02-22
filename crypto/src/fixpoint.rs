use std::{
    fmt::Display,
    ops::{Add, Div, Mul, Sub},
};

mod div;

use ethnum::U256;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct U128x128(U256);

impl From<u128> for U128x128 {
    fn from(value: u128) -> Self {
        Self(U256([0, value]))
    }
}

impl U128x128 {
    pub fn checked_mul(self, rhs: Self) -> Option<Self> {
        let [x0, x1] = self.0 .0;
        let [y0, y1] = rhs.0 .0;
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

        x1y1.checked_shl(128)
            .and_then(|acc| acc.checked_add(x0y1))
            .and_then(|acc| acc.checked_add(x1y0))
            .and_then(|acc| acc.checked_add(x0y0 >> 128))
            .map(U128x128)
    }

    pub fn checked_div(self, rhs: Self) -> Option<Self> {
        if rhs.0 == U256::ZERO {
            return None;
        }

        // TEMP HACK: need to implement this properly
        let self_big = ibig::UBig::from_le_bytes(&self.0.to_le_bytes());
        let rhs_big = ibig::UBig::from_le_bytes(&rhs.0.to_le_bytes());
        // this is what we actually want to compute: 384-bit / 256-bit division.
        let q_big = (self_big << 128) / rhs_big;
        let q_big_bytes = q_big.to_le_bytes();
        let mut q_bytes = [0; 32];
        if q_big_bytes.len() > 32 {
            return None;
        } else {
            q_bytes[..q_big_bytes.len()].copy_from_slice(&q_big_bytes);
        }
        let q = U256::from_le_bytes(q_bytes);

        Some(U128x128(q))
    }
}

impl Add<U128x128> for U128x128 {
    type Output = Option<U128x128>;
    fn add(self, rhs: U128x128) -> Self::Output {
        let checked_add = self.0.checked_add(rhs.0);
        let Some(result) = checked_add else {
            return None
        };
        Some(Self(result))
    }
}

impl Sub<U128x128> for U128x128 {
    type Output = Option<U128x128>;
    fn sub(self, rhs: U128x128) -> Self::Output {
        let checked_sub = self.0.checked_sub(rhs.0);
        let Some(result) = checked_sub else {
            return None
        };
        Some(Self(result))
    }
}

impl Mul<U128x128> for U128x128 {
    type Output = Option<U128x128>;
    fn mul(self, rhs: U128x128) -> Self::Output {
        let checked_mul = self.0.checked_mul(rhs.0);
        let Some(result) = checked_mul else {
            return None
        };
        Some(Self(result))
    }
}

impl Div<U128x128> for U128x128 {
    type Output = Option<U128x128>;
    fn div(self, rhs: U128x128) -> Self::Output {
        self.checked_div(rhs)
    }
}

impl From<U128x128> for f64 {
    fn from(value: U128x128) -> Self {
        let (hi, lo) = value.0.into_words();
        // binary repr of 2^128
        const BASE: u64 = 0x47f0000000000000;
        (hi as f64) + (lo as f64) / f64::from_bits(BASE)
    }
}

impl Display for U128x128 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", f64::from(self.clone()))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn basic() {
        use super::*;
        let a = U128x128::from(1);
        let x = U128x128::from(2);
        let y = U128x128::from(3);
        println!("a = {}, {:?}", a, a);
        println!("x = {}, {:?}", x, x);
        println!("y = {}, {:?}", y, y);
        let z = (x / y).unwrap();
        println!("z = {}, {:?}", z, z);
        let w = (y / x).unwrap();
        println!("w = {}, {:?}", w, w);
        let w2 = (w + w).unwrap();
        println!("w2 = {}, {:?}", w2, w2);
        let s = (w2 / y).unwrap();
        println!("s = {}, {:?}", s, s);
        let t = (z * w).unwrap();
        println!("t = {}, {:?}", t, t);
        let u = (U128x128::from(1) / y).unwrap();
        println!("u = {}, {:?}", u, u);
        let v = (U128x128::from(1) / u).unwrap();
        println!("v = {}, {:?}", v, v);
    }
}
