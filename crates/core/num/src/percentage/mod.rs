use crate::fixpoint::U128x128;

/// Represents a percentage value.
///
/// Useful for more robust typesafety, versus just passing around a `u64` which
/// is merely *understood* to only contain values in [0, 100].
///
/// Defaults to 0%.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Percentage(u64);

impl Percentage {
    /// 0%
    pub const fn zero() -> Self {
        Self(0)
    }

    /// Convert this value into a `u64` in [0, 100];
    pub const fn to_percent(self) -> u64 {
        self.0
    }

    /// Given an arbitrary `u64`, produce a percentage, *saturating* at 100.
    pub fn from_percent(p: u64) -> Self {
        Self(u64::min(p.into(), 100))
    }

    /// Given p%, return (1 - p)%.
    pub fn complement(self) -> Self {
        Self(100 - self.0)
    }
}

impl From<Percentage> for U128x128 {
    fn from(value: Percentage) -> Self {
        Self::ratio(value.to_percent(), 100).expect("dividing by 100 should succeed")
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_percentage_operations() {
        assert_eq!(Percentage::from_percent(101), Percentage::from_percent(100));
        assert_eq!(Percentage::from_percent(48).to_percent(), 48);
    }
}
