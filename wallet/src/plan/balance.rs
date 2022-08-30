use std::{
    collections::{btree_map, BTreeMap},
    fmt::Debug,
    iter::FusedIterator,
    mem,
    num::NonZeroU64,
    ops::{Add, AddAssign, Neg, Sub, SubAssign},
};

use penumbra_crypto::{asset, Value};

mod imbalance;
mod iter;
use imbalance::Imbalance;
pub use iter::{IntoIter, Iter};

#[derive(Debug, Clone, Eq, Default)]
pub struct Balance {
    negated: bool,
    balance: BTreeMap<asset::Id, Imbalance<NonZeroU64>>,
}

impl Balance {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_zero(&self) -> bool {
        self.balance.is_empty()
    }

    pub fn dimension(&self) -> usize {
        self.balance.len()
    }

    pub fn require(&mut self, value: Value) {
        *self -= Balance::from(value);
    }

    pub fn provide(&mut self, value: Value) {
        *self += Balance::from(value);
    }

    pub fn required(
        &self,
    ) -> impl Iterator<Item = Value> + DoubleEndedIterator + FusedIterator + '_ {
        self.iter().filter_map(Imbalance::required)
    }

    pub fn provided(
        &self,
    ) -> impl Iterator<Item = Value> + DoubleEndedIterator + FusedIterator + '_ {
        self.iter().filter_map(Imbalance::provided)
    }
}

impl PartialEq for Balance {
    // Eq is implemented this way because there are two different representations for a `Balance`,
    // to allow fast negation, so we check elements of the iterator against each other, because the
    // iterator returns the values in canonical imbalance representation, in order
    fn eq(&self, other: &Self) -> bool {
        if self.dimension() != other.dimension() {
            return false;
        }

        for (i, j) in self.iter().zip(other.iter()) {
            if i != j {
                return false;
            }
        }

        true
    }
}

impl Neg for Balance {
    type Output = Self;

    fn neg(self) -> Self {
        Self {
            negated: !self.negated,
            balance: self.balance,
        }
    }
}

impl Add for Balance {
    type Output = Self;

    fn add(mut self, mut other: Self) -> Self {
        // Always iterate through the smaller of the two
        if other.dimension() > self.dimension() {
            mem::swap(&mut self, &mut other);
        }

        for imbalance in other.into_iter() {
            // Convert back into an asset id key and imbalance value
            let (sign, Value { asset_id, amount }) = imbalance.into_inner();
            let (asset_id, imbalance) = if let Some(amount) = NonZeroU64::new(amount) {
                (asset_id, sign.imbalance(amount))
            } else {
                unreachable!("values stored in balance are always nonzero")
            };

            match self.balance.entry(asset_id) {
                btree_map::Entry::Vacant(entry) => {
                    entry.insert(imbalance);
                }
                btree_map::Entry::Occupied(mut entry) => {
                    if let Some(new_imbalance) = *entry.get() + imbalance {
                        // If there's still an imbalance, update the map entry
                        entry.insert(new_imbalance);
                    } else {
                        // If adding this imbalance zeroed out the balance for this asset, remove
                        // the entry
                        entry.remove();
                    }
                }
            }
        }

        self
    }
}

impl AddAssign for Balance {
    fn add_assign(&mut self, other: Self) {
        *self = mem::take(self) + other;
    }
}

impl Sub for Balance {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        self + -other
    }
}

impl SubAssign for Balance {
    fn sub_assign(&mut self, other: Self) {
        *self = mem::take(self) - other;
    }
}

impl From<Value> for Balance {
    fn from(Value { amount, asset_id }: Value) -> Self {
        let mut balance = BTreeMap::new();
        if let Some(amount) = NonZeroU64::new(amount) {
            balance.insert(asset_id, Imbalance::Provided(amount));
        }
        Balance {
            negated: false,
            balance,
        }
    }
}
