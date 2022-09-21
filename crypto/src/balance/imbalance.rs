use std::{
    cmp::Ordering,
    fmt::Debug,
    num::NonZeroU64,
    ops::{Add, Neg, Sub},
};

/// An imbalance is either a required amount or a provided amount.
///
/// This is used exclusively when the type contained is non-zero.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Imbalance<T> {
    /// Something is required, i.e. it must be cancelled out by a provided thing.
    Required(T),
    /// Something is provided, i.e. it must be cancelled out by a required thing.
    Provided(T),
}

impl<T> Neg for Imbalance<T> {
    type Output = Imbalance<T>;

    fn neg(self) -> Self::Output {
        match self {
            Imbalance::Required(t) => Imbalance::Provided(t),
            Imbalance::Provided(t) => Imbalance::Required(t),
        }
    }
}

impl Add for Imbalance<NonZeroU64> {
    type Output = Option<Self>;

    fn add(self, other: Self) -> Self::Output {
        // We define the case where the two are the same, and where the two are different, and in
        // the symmetric cases, use double-negation to avoid repeating the logic
        match (self, other) {
            (Imbalance::Required(r), Imbalance::Required(s)) => {
                if let Some(t) = r.get().checked_add(s.get()) {
                    Some(Imbalance::Required(
                        NonZeroU64::new(t).expect("checked addition of nonzero u64 never is zero"),
                    ))
                } else {
                    panic!("overflow when adding imbalances")
                }
            }
            (Imbalance::Required(r), Imbalance::Provided(p)) => match p.cmp(&r) {
                Ordering::Less => Some(Imbalance::Required(
                    NonZeroU64::new(r.get() - p.get())
                        .expect("subtraction of lesser from greater is never zero"),
                )),
                Ordering::Equal => None,
                Ordering::Greater => Some(Imbalance::Provided(
                    NonZeroU64::new(p.get() - r.get())
                        .expect("subtraction of lesser from greater is never zero"),
                )),
            },
            (x, y) => Some(-((-x + -y)?)),
        }
    }
}

impl Sub for Imbalance<NonZeroU64> {
    type Output = <Self as Add>::Output;

    fn sub(self, other: Self) -> Self::Output {
        self + -other
    }
}

impl<T> Imbalance<T> {
    /// Split an imbalance into its sign and the thing contained in it.
    pub fn into_inner(self) -> (Sign, T) {
        match self {
            Imbalance::Required(t) => (Sign::Required, t),
            Imbalance::Provided(t) => (Sign::Provided, t),
        }
    }

    /// Map a function over both required or provided possibilities.
    pub fn map<S>(self, f: impl FnOnce(T) -> S) -> Imbalance<S> {
        match self {
            Imbalance::Required(t) => Imbalance::Required(f(t)),
            Imbalance::Provided(t) => Imbalance::Provided(f(t)),
        }
    }

    /// Filter an imbalance to get only the `T` out if it is a required thing.
    pub fn required(self) -> Option<T> {
        match self {
            Imbalance::Required(t) => Some(t),
            Imbalance::Provided(_) => None,
        }
    }

    /// Filter an imbalance to get only the `T` out if it is a provided thing.
    pub fn provided(self) -> Option<T> {
        match self {
            Imbalance::Required(_) => None,
            Imbalance::Provided(t) => Some(t),
        }
    }

    /// Check if an imbalance is required.
    pub fn is_required(&self) -> bool {
        matches!(self, Imbalance::Required(_))
    }

    /// Check if an imbalance is provided.
    pub fn is_provided(&self) -> bool {
        !self.is_required()
    }

    /// Get the sign of an imbalance.
    pub fn sign(&self) -> Sign {
        match self {
            Imbalance::Required(_) => Sign::Required,
            Imbalance::Provided(_) => Sign::Provided,
        }
    }
}

/// The sign of an imbalance is whether it is required or provided.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Sign {
    Required,
    Provided,
}

impl Sign {
    /// Check if the sign is required.
    pub fn is_required(&self) -> bool {
        matches!(self, Sign::Required)
    }

    /// Check if the sign if provided.
    pub fn is_provided(&self) -> bool {
        !self.is_required()
    }

    /// Form a new [`Imbalance`] by using the sign as a constructor for some value.
    pub fn imbalance<T>(&self, t: T) -> Imbalance<T> {
        match self {
            Sign::Required => Imbalance::Required(t),
            Sign::Provided => Imbalance::Provided(t),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn add_provided_provided() {
        let a = Imbalance::Provided(NonZeroU64::new(1).unwrap());
        let b = Imbalance::Provided(NonZeroU64::new(2).unwrap());
        let c = a + b;
        assert_eq!(c, Some(Imbalance::Provided(NonZeroU64::new(3).unwrap())));
    }

    #[test]
    fn add_provided_required_greater() {
        let a = Imbalance::Provided(NonZeroU64::new(2).unwrap());
        let b = Imbalance::Required(NonZeroU64::new(1).unwrap());
        let c = a + b;
        assert_eq!(c, Some(Imbalance::Provided(NonZeroU64::new(1).unwrap())));
    }

    #[test]
    fn add_provided_required_equal() {
        let a = Imbalance::Provided(NonZeroU64::new(1).unwrap());
        let b = Imbalance::Required(NonZeroU64::new(1).unwrap());
        let c = a + b;
        assert_eq!(c, None);
    }

    #[test]
    fn add_provided_required_less() {
        let a = Imbalance::Provided(NonZeroU64::new(1).unwrap());
        let b = Imbalance::Required(NonZeroU64::new(2).unwrap());
        let c = a + b;
        assert_eq!(c, Some(Imbalance::Required(NonZeroU64::new(1).unwrap())));
    }

    #[test]
    fn add_required_required() {
        let a = Imbalance::Required(NonZeroU64::new(1).unwrap());
        let b = Imbalance::Required(NonZeroU64::new(2).unwrap());
        let c = a + b;
        assert_eq!(c, Some(Imbalance::Required(NonZeroU64::new(3).unwrap())));
    }

    #[test]
    fn add_required_provided_greater() {
        let a = Imbalance::Required(NonZeroU64::new(2).unwrap());
        let b = Imbalance::Provided(NonZeroU64::new(1).unwrap());
        let c = a + b;
        assert_eq!(c, Some(Imbalance::Required(NonZeroU64::new(1).unwrap())));
    }

    #[test]
    fn add_required_provided_equal() {
        let a = Imbalance::Required(NonZeroU64::new(1).unwrap());
        let b = Imbalance::Provided(NonZeroU64::new(1).unwrap());
        let c = a + b;
        assert_eq!(c, None);
    }

    #[test]
    fn add_required_provided_less() {
        let a = Imbalance::Required(NonZeroU64::new(1).unwrap());
        let b = Imbalance::Provided(NonZeroU64::new(2).unwrap());
        let c = a + b;
        assert_eq!(c, Some(Imbalance::Provided(NonZeroU64::new(1).unwrap())));
    }

    #[test]
    fn sub_provided_provided_greater() {
        let a = Imbalance::Provided(NonZeroU64::new(2).unwrap());
        let b = Imbalance::Provided(NonZeroU64::new(1).unwrap());
        let c = a - b;
        assert_eq!(c, Some(Imbalance::Provided(NonZeroU64::new(1).unwrap())));
    }

    #[test]
    fn sub_provided_provided_equal() {
        let a = Imbalance::Provided(NonZeroU64::new(1).unwrap());
        let b = Imbalance::Provided(NonZeroU64::new(1).unwrap());
        let c = a - b;
        assert_eq!(c, None);
    }

    #[test]
    fn sub_provided_provided_less() {
        let a = Imbalance::Provided(NonZeroU64::new(1).unwrap());
        let b = Imbalance::Provided(NonZeroU64::new(2).unwrap());
        let c = a - b;
        assert_eq!(c, Some(Imbalance::Required(NonZeroU64::new(1).unwrap())));
    }

    #[test]
    fn sub_provided_required_greater() {
        let a = Imbalance::Provided(NonZeroU64::new(2).unwrap());
        let b = Imbalance::Required(NonZeroU64::new(1).unwrap());
        let c = a - b;
        assert_eq!(c, Some(Imbalance::Provided(NonZeroU64::new(3).unwrap())));
    }

    #[test]
    fn sub_provided_required_equal() {
        let a = Imbalance::Provided(NonZeroU64::new(1).unwrap());
        let b = Imbalance::Required(NonZeroU64::new(1).unwrap());
        let c = a - b;
        assert_eq!(c, Some(Imbalance::Provided(NonZeroU64::new(2).unwrap())));
    }

    #[test]
    fn sub_provided_required_less() {
        let a = Imbalance::Provided(NonZeroU64::new(1).unwrap());
        let b = Imbalance::Required(NonZeroU64::new(2).unwrap());
        let c = a - b;
        assert_eq!(c, Some(Imbalance::Provided(NonZeroU64::new(3).unwrap())));
    }

    #[test]
    fn sub_required_provided_greater() {
        let a = Imbalance::Required(NonZeroU64::new(2).unwrap());
        let b = Imbalance::Provided(NonZeroU64::new(1).unwrap());
        let c = a - b;
        assert_eq!(c, Some(Imbalance::Required(NonZeroU64::new(3).unwrap())));
    }

    #[test]
    fn sub_required_provided_equal() {
        let a = Imbalance::Required(NonZeroU64::new(1).unwrap());
        let b = Imbalance::Provided(NonZeroU64::new(1).unwrap());
        let c = a - b;
        assert_eq!(c, Some(Imbalance::Required(NonZeroU64::new(2).unwrap())));
    }

    #[test]
    fn sub_required_provided_less() {
        let a = Imbalance::Required(NonZeroU64::new(1).unwrap());
        let b = Imbalance::Provided(NonZeroU64::new(2).unwrap());
        let c = a - b;
        assert_eq!(c, Some(Imbalance::Required(NonZeroU64::new(3).unwrap())));
    }

    #[test]
    fn sub_required_required_greater() {
        let a = Imbalance::Required(NonZeroU64::new(2).unwrap());
        let b = Imbalance::Required(NonZeroU64::new(1).unwrap());
        let c = a - b;
        assert_eq!(c, Some(Imbalance::Required(NonZeroU64::new(1).unwrap())));
    }

    #[test]
    fn sub_required_required_equal() {
        let a = Imbalance::Required(NonZeroU64::new(1).unwrap());
        let b = Imbalance::Required(NonZeroU64::new(1).unwrap());
        let c = a - b;
        assert_eq!(c, None);
    }

    #[test]
    fn sub_required_required_less() {
        let a = Imbalance::Required(NonZeroU64::new(1).unwrap());
        let b = Imbalance::Required(NonZeroU64::new(2).unwrap());
        let c = a - b;
        assert_eq!(c, Some(Imbalance::Provided(NonZeroU64::new(1).unwrap())));
    }
}
