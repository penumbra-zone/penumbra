use std::{
    cmp::Ordering,
    fmt::Debug,
    num::NonZeroU64,
    ops::{Add, AddAssign, Neg, Sub, SubAssign},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Imbalance<T> {
    Required(T),
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

impl AddAssign for Imbalance<NonZeroU64> {
    fn add_assign(&mut self, other: Self) {
        if let Some(new) = *self + other {
            *self = new;
        }
    }
}

impl Sub for Imbalance<NonZeroU64> {
    type Output = <Self as Add>::Output;

    fn sub(self, other: Self) -> Self::Output {
        self + -other
    }
}

impl SubAssign for Imbalance<NonZeroU64> {
    fn sub_assign(&mut self, other: Self) {
        if let Some(new) = *self - other {
            *self = new;
        }
    }
}

impl<T> Imbalance<T> {
    pub fn into_inner(self) -> (Sign, T) {
        match self {
            Imbalance::Required(t) => (Sign::Required, t),
            Imbalance::Provided(t) => (Sign::Provided, t),
        }
    }

    pub fn map<S>(self, f: impl FnOnce(T) -> S) -> Imbalance<S> {
        match self {
            Imbalance::Required(t) => Imbalance::Required(f(t)),
            Imbalance::Provided(t) => Imbalance::Provided(f(t)),
        }
    }

    pub fn required(self) -> Option<T> {
        match self {
            Imbalance::Required(t) => Some(t),
            Imbalance::Provided(_) => None,
        }
    }

    pub fn provided(self) -> Option<T> {
        match self {
            Imbalance::Required(_) => None,
            Imbalance::Provided(t) => Some(t),
        }
    }

    pub fn is_required(&self) -> bool {
        matches!(self, Imbalance::Required(_))
    }

    pub fn is_provided(&self) -> bool {
        !self.is_required()
    }

    pub fn sign(&self) -> Sign {
        match self {
            Imbalance::Required(_) => Sign::Required,
            Imbalance::Provided(_) => Sign::Provided,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Sign {
    Required,
    Provided,
}

impl Sign {
    pub fn is_required(&self) -> bool {
        matches!(self, Sign::Required)
    }

    pub fn is_provided(&self) -> bool {
        !self.is_required()
    }

    pub fn imbalance<T>(&self, t: T) -> Imbalance<T> {
        match self {
            Sign::Required => Imbalance::Required(t),
            Sign::Provided => Imbalance::Provided(t),
        }
    }
}
