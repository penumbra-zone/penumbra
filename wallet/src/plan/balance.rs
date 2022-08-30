use std::{
    collections::{btree_map, BTreeMap},
    fmt::{self, Debug, Formatter},
    iter::FusedIterator,
    mem,
    num::NonZeroU64,
    ops::{Add, AddAssign, Neg, Sub, SubAssign},
};

use tracing::instrument;

use penumbra_crypto::{asset, Value};

mod imbalance;
mod iter;
use imbalance::Imbalance;

#[derive(Clone, Eq, Default)]
pub struct Balance {
    negated: bool,
    balance: BTreeMap<asset::Id, Imbalance<NonZeroU64>>,
}

impl Debug for Balance {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Balance")
            .field("required", &self.required().collect::<Vec<_>>())
            .field("provided", &self.provided().collect::<Vec<_>>())
            .finish()
    }
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

    #[instrument(skip(self))]
    pub fn require(&mut self, value: Value) {
        tracing::trace!("requiring balance");
        *self -= Balance::from(value);
    }

    #[instrument(skip(self))]
    pub fn provide(&mut self, value: Value) {
        tracing::trace!("providing balance");
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

    // This is a tricky function, because the representation of a `Balance` has a `negated` flag
    // which inverts the meaning of the stored entry (this is so that you can negate balances in
    // constant time, which makes subtraction fast to implement). As a consequence, however, we have
    // to take care that when we access the raw storage, we negate the imbalance we retrieve if and
    // only if we are in negated mode, and when we write back a value, we negate it again on writing
    // it back if we are in negated mode.
    fn add(mut self, mut other: Self) -> Self {
        // Always iterate through the smaller of the two
        if other.dimension() > self.dimension() {
            mem::swap(&mut self, &mut other);
        }

        for imbalance in other.into_iter() {
            // Convert back into an asset id key and imbalance value
            let (sign, Value { asset_id, amount }) = imbalance.into_inner();
            let (asset_id, mut imbalance) = if let Some(amount) = NonZeroU64::new(amount) {
                (asset_id, sign.imbalance(amount))
            } else {
                unreachable!("values stored in balance are always nonzero")
            };

            match self.balance.entry(asset_id) {
                btree_map::Entry::Vacant(entry) => {
                    // Important: if we are currently negated, we have to negate the imbalance
                    // before we store it!
                    if self.negated {
                        imbalance = -imbalance;
                    }
                    entry.insert(imbalance);
                }
                btree_map::Entry::Occupied(mut entry) => {
                    // Important: if we are currently negated, we have to negate the entry we just
                    // pulled out!
                    let mut existing_imbalance = *entry.get();
                    if self.negated {
                        existing_imbalance = -existing_imbalance;
                    }

                    if let Some(mut new_imbalance) = existing_imbalance + imbalance {
                        // If there's still an imbalance, update the map entry, making sure to
                        // negate the new imbalance if we are negated
                        if self.negated {
                            new_imbalance = -new_imbalance;
                        }
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

#[cfg(test)]
mod test {
    use penumbra_crypto::STAKING_TOKEN_ASSET_ID;

    use super::*;

    #[test]
    fn provide_then_require() {
        let mut balance = Balance::new();
        balance.provide(Value {
            amount: 1,
            asset_id: *STAKING_TOKEN_ASSET_ID,
        });
        balance.require(Value {
            amount: 1,
            asset_id: *STAKING_TOKEN_ASSET_ID,
        });
        assert!(balance.is_zero());
    }

    #[test]
    fn require_then_provide() {
        let mut balance = Balance::new();
        balance.require(Value {
            amount: 1,
            asset_id: *STAKING_TOKEN_ASSET_ID,
        });
        balance.provide(Value {
            amount: 1,
            asset_id: *STAKING_TOKEN_ASSET_ID,
        });
        assert!(balance.is_zero());
    }

    #[test]
    fn provide_then_require_negative_zero() {
        let mut balance = -Balance::new();
        balance.provide(Value {
            amount: 1,
            asset_id: *STAKING_TOKEN_ASSET_ID,
        });
        balance.require(Value {
            amount: 1,
            asset_id: *STAKING_TOKEN_ASSET_ID,
        });
        assert!(balance.is_zero());
    }

    #[test]
    fn require_then_provide_negative_zero() {
        let mut balance = -Balance::new();
        balance.require(Value {
            amount: 1,
            asset_id: *STAKING_TOKEN_ASSET_ID,
        });
        balance.provide(Value {
            amount: 1,
            asset_id: *STAKING_TOKEN_ASSET_ID,
        });
        assert!(balance.is_zero());
    }
}
