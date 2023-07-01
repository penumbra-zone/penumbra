use crate::fixpoint::{Error, U128x128};
use std::ops::{Add, Div, Mul, Sub};

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
    type Output = Result<U128x128, Error>;
    fn add(self, rhs: U128x128) -> Self::Output {
        self.checked_add(&rhs)
    }
}

impl Add<&U128x128> for U128x128 {
    type Output = Result<U128x128, Error>;
    fn add(self, rhs: &U128x128) -> Self::Output {
        self.checked_add(rhs)
    }
}

impl Add<U128x128> for &U128x128 {
    type Output = Result<U128x128, Error>;
    fn add(self, rhs: U128x128) -> Self::Output {
        self.checked_add(&rhs)
    }
}

impl Add<&U128x128> for &U128x128 {
    type Output = Result<U128x128, Error>;
    fn add(self, rhs: &U128x128) -> Self::Output {
        self.checked_add(rhs)
    }
}

impl Add<U128x128> for Result<U128x128, Error> {
    type Output = Result<U128x128, Error>;
    fn add(self, rhs: U128x128) -> Self::Output {
        self.and_then(|lhs| lhs.checked_add(&rhs))
    }
}

impl Add<Result<U128x128, Error>> for U128x128 {
    type Output = Result<U128x128, Error>;
    fn add(self, rhs: Result<U128x128, Error>) -> Self::Output {
        rhs.and_then(|rhs| self.checked_add(&rhs))
    }
}

impl Add<&U128x128> for Result<U128x128, Error> {
    type Output = Result<U128x128, Error>;
    fn add(self, rhs: &U128x128) -> Self::Output {
        self.and_then(|lhs| lhs.checked_add(rhs))
    }
}

impl Add<Result<U128x128, Error>> for &U128x128 {
    type Output = Result<U128x128, Error>;
    fn add(self, rhs: Result<U128x128, Error>) -> Self::Output {
        rhs.and_then(|rhs| self.checked_add(&rhs))
    }
}

impl Sub<U128x128> for U128x128 {
    type Output = Result<U128x128, Error>;
    fn sub(self, rhs: U128x128) -> Self::Output {
        self.checked_sub(&rhs)
    }
}

impl Sub<&U128x128> for U128x128 {
    type Output = Result<U128x128, Error>;
    fn sub(self, rhs: &U128x128) -> Self::Output {
        self.checked_sub(rhs)
    }
}

impl Sub<U128x128> for &U128x128 {
    type Output = Result<U128x128, Error>;
    fn sub(self, rhs: U128x128) -> Self::Output {
        self.checked_sub(&rhs)
    }
}

impl Sub<&U128x128> for &U128x128 {
    type Output = Result<U128x128, Error>;
    fn sub(self, rhs: &U128x128) -> Self::Output {
        self.checked_sub(rhs)
    }
}

impl Sub<U128x128> for Result<U128x128, Error> {
    type Output = Result<U128x128, Error>;
    fn sub(self, rhs: U128x128) -> Self::Output {
        self.and_then(|lhs| lhs.checked_sub(&rhs))
    }
}

impl Sub<Result<U128x128, Error>> for U128x128 {
    type Output = Result<U128x128, Error>;
    fn sub(self, rhs: Result<U128x128, Error>) -> Self::Output {
        rhs.and_then(|rhs| self.checked_sub(&rhs))
    }
}

impl Sub<&U128x128> for Result<U128x128, Error> {
    type Output = Result<U128x128, Error>;
    fn sub(self, rhs: &U128x128) -> Self::Output {
        self.and_then(|lhs| lhs.checked_sub(rhs))
    }
}

impl Sub<Result<U128x128, Error>> for &U128x128 {
    type Output = Result<U128x128, Error>;
    fn sub(self, rhs: Result<U128x128, Error>) -> Self::Output {
        rhs.and_then(|rhs| self.checked_sub(&rhs))
    }
}

impl Mul<U128x128> for U128x128 {
    type Output = Result<U128x128, Error>;
    fn mul(self, rhs: U128x128) -> Self::Output {
        self.checked_mul(&rhs)
    }
}

impl Mul<&U128x128> for U128x128 {
    type Output = Result<U128x128, Error>;
    fn mul(self, rhs: &U128x128) -> Self::Output {
        self.checked_mul(rhs)
    }
}

impl Mul<U128x128> for &U128x128 {
    type Output = Result<U128x128, Error>;
    fn mul(self, rhs: U128x128) -> Self::Output {
        self.checked_mul(&rhs)
    }
}

impl Mul<&U128x128> for &U128x128 {
    type Output = Result<U128x128, Error>;
    fn mul(self, rhs: &U128x128) -> Self::Output {
        self.checked_mul(rhs)
    }
}

impl Mul<U128x128> for Result<U128x128, Error> {
    type Output = Result<U128x128, Error>;
    fn mul(self, rhs: U128x128) -> Self::Output {
        self.and_then(|lhs| lhs.checked_mul(&rhs))
    }
}

impl Mul<Result<U128x128, Error>> for U128x128 {
    type Output = Result<U128x128, Error>;
    fn mul(self, rhs: Result<U128x128, Error>) -> Self::Output {
        rhs.and_then(|rhs| self.checked_mul(&rhs))
    }
}

impl Mul<&U128x128> for Result<U128x128, Error> {
    type Output = Result<U128x128, Error>;
    fn mul(self, rhs: &U128x128) -> Self::Output {
        self.and_then(|lhs| lhs.checked_mul(rhs))
    }
}

impl Mul<Result<U128x128, Error>> for &U128x128 {
    type Output = Result<U128x128, Error>;
    fn mul(self, rhs: Result<U128x128, Error>) -> Self::Output {
        rhs.and_then(|rhs| self.checked_mul(&rhs))
    }
}

impl Div<U128x128> for U128x128 {
    type Output = Result<U128x128, Error>;
    fn div(self, rhs: U128x128) -> Self::Output {
        self.checked_div(&rhs)
    }
}

impl Div<&U128x128> for U128x128 {
    type Output = Result<U128x128, Error>;
    fn div(self, rhs: &U128x128) -> Self::Output {
        self.checked_div(rhs)
    }
}

impl Div<U128x128> for &U128x128 {
    type Output = Result<U128x128, Error>;
    fn div(self, rhs: U128x128) -> Self::Output {
        self.checked_div(&rhs)
    }
}

impl Div<&U128x128> for &U128x128 {
    type Output = Result<U128x128, Error>;
    fn div(self, rhs: &U128x128) -> Self::Output {
        self.checked_div(rhs)
    }
}

impl Div<U128x128> for Result<U128x128, Error> {
    type Output = Result<U128x128, Error>;
    fn div(self, rhs: U128x128) -> Self::Output {
        self.and_then(|lhs| lhs.checked_div(&rhs))
    }
}

impl Div<Result<U128x128, Error>> for U128x128 {
    type Output = Result<U128x128, Error>;
    fn div(self, rhs: Result<U128x128, Error>) -> Self::Output {
        rhs.and_then(|rhs| self.checked_div(&rhs))
    }
}

impl Div<&U128x128> for Result<U128x128, Error> {
    type Output = Result<U128x128, Error>;
    fn div(self, rhs: &U128x128) -> Self::Output {
        self.and_then(|lhs| lhs.checked_div(rhs))
    }
}

impl Div<Result<U128x128, Error>> for &U128x128 {
    type Output = Result<U128x128, Error>;
    fn div(self, rhs: Result<U128x128, Error>) -> Self::Output {
        rhs.and_then(|rhs| self.checked_div(&rhs))
    }
}
