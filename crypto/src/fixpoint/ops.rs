use std::ops::{Add, Div, Mul, Sub};

use super::U128x128;

// There are 8 impl variants per operation:
//
// (       T ,        T )
// (       T ,       &T )
// (      &T ,        T )
// (      &T ,       &T )
// (Option<T>,        T )
// (Option<T>,       &T )
// (       T , Option<T>)
// (      &T , Option<T>)
//
// We can't do (Option, Option) because of orphan rules.
// We don't do (Option<&>, _) because the reason to do Option
// is to do operations on outputs, which are owned.

impl Add<U128x128> for U128x128 {
    type Output = Option<U128x128>;
    fn add(self, rhs: U128x128) -> Self::Output {
        self.0.checked_add(rhs.0).map(Self)
    }
}

impl Add<&U128x128> for U128x128 {
    type Output = Option<U128x128>;
    fn add(self, rhs: &U128x128) -> Self::Output {
        self.0.checked_add(rhs.0).map(Self)
    }
}

impl Add<U128x128> for &U128x128 {
    type Output = Option<U128x128>;
    fn add(self, rhs: U128x128) -> Self::Output {
        self.0.checked_add(rhs.0).map(U128x128)
    }
}

impl Add<&U128x128> for &U128x128 {
    type Output = Option<U128x128>;
    fn add(self, rhs: &U128x128) -> Self::Output {
        self.0.checked_add(rhs.0).map(U128x128)
    }
}

impl Add<U128x128> for Option<U128x128> {
    type Output = Option<U128x128>;
    fn add(self, rhs: U128x128) -> Self::Output {
        self.and_then(|lhs| lhs.checked_add(&rhs))
    }
}

impl Add<Option<U128x128>> for U128x128 {
    type Output = Option<U128x128>;
    fn add(self, rhs: Option<U128x128>) -> Self::Output {
        rhs.and_then(|rhs| self.checked_add(&rhs))
    }
}

impl Add<&U128x128> for Option<U128x128> {
    type Output = Option<U128x128>;
    fn add(self, rhs: &U128x128) -> Self::Output {
        self.and_then(|lhs| lhs.checked_add(rhs))
    }
}

impl Add<Option<U128x128>> for &U128x128 {
    type Output = Option<U128x128>;
    fn add(self, rhs: Option<U128x128>) -> Self::Output {
        rhs.and_then(|rhs| self.checked_add(&rhs))
    }
}

impl Sub<U128x128> for U128x128 {
    type Output = Option<U128x128>;
    fn sub(self, rhs: U128x128) -> Self::Output {
        self.checked_sub(&rhs)
    }
}

impl Sub<&U128x128> for U128x128 {
    type Output = Option<U128x128>;
    fn sub(self, rhs: &U128x128) -> Self::Output {
        self.checked_sub(rhs)
    }
}

impl Sub<U128x128> for &U128x128 {
    type Output = Option<U128x128>;
    fn sub(self, rhs: U128x128) -> Self::Output {
        self.checked_sub(&rhs)
    }
}

impl Sub<&U128x128> for &U128x128 {
    type Output = Option<U128x128>;
    fn sub(self, rhs: &U128x128) -> Self::Output {
        self.checked_sub(rhs)
    }
}

impl Sub<U128x128> for Option<U128x128> {
    type Output = Option<U128x128>;
    fn sub(self, rhs: U128x128) -> Self::Output {
        self.and_then(|lhs| lhs.checked_sub(&rhs))
    }
}

impl Sub<Option<U128x128>> for U128x128 {
    type Output = Option<U128x128>;
    fn sub(self, rhs: Option<U128x128>) -> Self::Output {
        rhs.and_then(|rhs| self.checked_sub(&rhs))
    }
}

impl Sub<&U128x128> for Option<U128x128> {
    type Output = Option<U128x128>;
    fn sub(self, rhs: &U128x128) -> Self::Output {
        self.and_then(|lhs| lhs.checked_sub(rhs))
    }
}

impl Sub<Option<U128x128>> for &U128x128 {
    type Output = Option<U128x128>;
    fn sub(self, rhs: Option<U128x128>) -> Self::Output {
        rhs.and_then(|rhs| self.checked_sub(&rhs))
    }
}

impl Mul<U128x128> for U128x128 {
    type Output = Option<U128x128>;
    fn mul(self, rhs: U128x128) -> Self::Output {
        self.checked_mul(&rhs)
    }
}

impl Mul<&U128x128> for U128x128 {
    type Output = Option<U128x128>;
    fn mul(self, rhs: &U128x128) -> Self::Output {
        self.checked_mul(rhs)
    }
}

impl Mul<U128x128> for &U128x128 {
    type Output = Option<U128x128>;
    fn mul(self, rhs: U128x128) -> Self::Output {
        self.checked_mul(&rhs)
    }
}

impl Mul<&U128x128> for &U128x128 {
    type Output = Option<U128x128>;
    fn mul(self, rhs: &U128x128) -> Self::Output {
        self.checked_mul(rhs)
    }
}

impl Mul<U128x128> for Option<U128x128> {
    type Output = Option<U128x128>;
    fn mul(self, rhs: U128x128) -> Self::Output {
        self.and_then(|lhs| lhs.checked_mul(&rhs))
    }
}

impl Mul<Option<U128x128>> for U128x128 {
    type Output = Option<U128x128>;
    fn mul(self, rhs: Option<U128x128>) -> Self::Output {
        rhs.and_then(|rhs| self.checked_mul(&rhs))
    }
}

impl Mul<&U128x128> for Option<U128x128> {
    type Output = Option<U128x128>;
    fn mul(self, rhs: &U128x128) -> Self::Output {
        self.and_then(|lhs| lhs.checked_mul(rhs))
    }
}

impl Mul<Option<U128x128>> for &U128x128 {
    type Output = Option<U128x128>;
    fn mul(self, rhs: Option<U128x128>) -> Self::Output {
        rhs.and_then(|rhs| self.checked_mul(&rhs))
    }
}

impl Div<U128x128> for U128x128 {
    type Output = Option<U128x128>;
    fn div(self, rhs: U128x128) -> Self::Output {
        self.checked_div(&rhs)
    }
}

impl Div<&U128x128> for U128x128 {
    type Output = Option<U128x128>;
    fn div(self, rhs: &U128x128) -> Self::Output {
        self.checked_div(rhs)
    }
}

impl Div<U128x128> for &U128x128 {
    type Output = Option<U128x128>;
    fn div(self, rhs: U128x128) -> Self::Output {
        self.checked_div(&rhs)
    }
}

impl Div<&U128x128> for &U128x128 {
    type Output = Option<U128x128>;
    fn div(self, rhs: &U128x128) -> Self::Output {
        self.checked_div(rhs)
    }
}

impl Div<U128x128> for Option<U128x128> {
    type Output = Option<U128x128>;
    fn div(self, rhs: U128x128) -> Self::Output {
        self.and_then(|lhs| lhs.checked_div(&rhs))
    }
}

impl Div<Option<U128x128>> for U128x128 {
    type Output = Option<U128x128>;
    fn div(self, rhs: Option<U128x128>) -> Self::Output {
        rhs.and_then(|rhs| self.checked_div(&rhs))
    }
}

impl Div<&U128x128> for Option<U128x128> {
    type Output = Option<U128x128>;
    fn div(self, rhs: &U128x128) -> Self::Output {
        self.and_then(|lhs| lhs.checked_div(rhs))
    }
}

impl Div<Option<U128x128>> for &U128x128 {
    type Output = Option<U128x128>;
    fn div(self, rhs: Option<U128x128>) -> Self::Output {
        rhs.and_then(|rhs| self.checked_div(&rhs))
    }
}
