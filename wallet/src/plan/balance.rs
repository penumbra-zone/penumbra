use std::{
    collections::{btree_map, BTreeMap},
    fmt::{self, Debug, Formatter},
    iter::FusedIterator,
    mem,
    num::NonZeroU64,
    ops::{Add, AddAssign, Neg, Sub, SubAssign},
};

use penumbra_crypto::{asset, Value};

mod imbalance;
mod iter;
use imbalance::Imbalance;

/// A `Balance` is a "vector of [`Value`]s", where some values may be required, while others may be
/// provided. For a transaction to be valid, its balance must be zero.
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
    /// Make a new, zero balance.
    pub fn zero() -> Self {
        Self::default()
    }

    /// Check if this balance is zero.
    pub fn is_zero(&self) -> bool {
        self.balance.is_empty()
    }

    /// Find out how many distinct assets are represented in this balance.
    pub fn dimension(&self) -> usize {
        self.balance.len()
    }

    /// Iterate over all the requirements of the balance, as [`Value`]s.
    pub fn required(
        &self,
    ) -> impl Iterator<Item = Value> + DoubleEndedIterator + FusedIterator + '_ {
        self.iter().filter_map(Imbalance::required)
    }

    // Iterate over all the provisions of the balance, as [`Value`]s.
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
            let (asset_id, mut imbalance) = if let Some(amount) = NonZeroU64::new(amount.into()) {
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

impl Add<Value> for Balance {
    type Output = Balance;

    fn add(self, value: Value) -> Self::Output {
        self + Balance::from(value)
    }
}

impl AddAssign for Balance {
    fn add_assign(&mut self, other: Self) {
        *self = mem::take(self) + other;
    }
}

impl AddAssign<Value> for Balance {
    fn add_assign(&mut self, other: Value) {
        *self += Balance::from(other);
    }
}

impl Sub for Balance {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        self + -other
    }
}

impl Sub<Value> for Balance {
    type Output = Balance;

    fn sub(self, value: Value) -> Self::Output {
        self - Balance::from(value)
    }
}

impl SubAssign for Balance {
    fn sub_assign(&mut self, other: Self) {
        *self = mem::take(self) - other;
    }
}

impl SubAssign<Value> for Balance {
    fn sub_assign(&mut self, other: Value) {
        *self -= Balance::from(other);
    }
}

impl From<Value> for Balance {
    fn from(Value { amount, asset_id }: Value) -> Self {
        let mut balance = BTreeMap::new();
        if let Some(amount) = NonZeroU64::new(amount.into()) {
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
    use once_cell::sync::Lazy;
    use penumbra_crypto::{value, Fr, Zero, STAKING_TOKEN_ASSET_ID};
    use proptest::prelude::*;

    use super::*;

    #[test]
    fn provide_then_require() {
        let mut balance = Balance::zero();
        balance += Value {
            amount: 1u64.into(),
            asset_id: *STAKING_TOKEN_ASSET_ID,
        };
        balance -= Value {
            amount: 1u64.into(),
            asset_id: *STAKING_TOKEN_ASSET_ID,
        };
        assert!(balance.is_zero());
    }

    #[test]
    fn require_then_provide() {
        let mut balance = Balance::zero();
        balance -= Value {
            amount: 1u64.into(),
            asset_id: *STAKING_TOKEN_ASSET_ID,
        };
        balance += Value {
            amount: 1u64.into(),
            asset_id: *STAKING_TOKEN_ASSET_ID,
        };
        assert!(balance.is_zero());
    }

    #[test]
    fn provide_then_require_negative_zero() {
        let mut balance = -Balance::zero();
        balance += Value {
            amount: 1u64.into(),
            asset_id: *STAKING_TOKEN_ASSET_ID,
        };
        balance -= Value {
            amount: 1u64.into(),
            asset_id: *STAKING_TOKEN_ASSET_ID,
        };
        assert!(balance.is_zero());
    }

    #[test]
    fn require_then_provide_negative_zero() {
        let mut balance = -Balance::zero();
        balance -= Value {
            amount: 1u64.into(),
            asset_id: *STAKING_TOKEN_ASSET_ID,
        };
        balance += Value {
            amount: 1u64.into(),
            asset_id: *STAKING_TOKEN_ASSET_ID,
        };
        assert!(balance.is_zero());
    }

    #[derive(Debug, Clone)]
    enum Expression {
        Value(Value),
        Neg(Box<Expression>),
        Add(Box<Expression>, Box<Expression>),
        Sub(Box<Expression>, Box<Expression>),
    }

    impl Expression {
        fn transparent_value_commitment(&self) -> value::Commitment {
            match self {
                Expression::Value(value) => value.commit(Fr::zero()),
                Expression::Neg(expr) => -expr.transparent_value_commitment(),
                Expression::Add(lhs, rhs) => {
                    lhs.transparent_value_commitment() + rhs.transparent_value_commitment()
                }
                Expression::Sub(lhs, rhs) => {
                    lhs.transparent_value_commitment() - rhs.transparent_value_commitment()
                }
            }
        }

        fn balance(&self) -> Balance {
            match self {
                Expression::Value(value) => Balance::from(*value),
                Expression::Neg(expr) => -expr.balance(),
                Expression::Add(lhs, rhs) => lhs.balance() + rhs.balance(),
                Expression::Sub(lhs, rhs) => lhs.balance() - rhs.balance(),
            }
        }
    }

    // Two sample denom/asset id pairs, for testing
    static DENOM_1: Lazy<asset::Denom> = Lazy::new(|| asset::REGISTRY.parse_denom("a").unwrap());
    static ASSET_ID_1: Lazy<asset::Id> = Lazy::new(|| DENOM_1.id());

    static DENOM_2: Lazy<asset::Denom> = Lazy::new(|| asset::REGISTRY.parse_denom("b").unwrap());
    static ASSET_ID_2: Lazy<asset::Id> = Lazy::new(|| DENOM_2.id());

    fn gen_expression() -> impl proptest::strategy::Strategy<Value = Expression> {
        (
            (0u64..u32::MAX as u64), // limit amounts so that there is no overflow
            prop_oneof![Just(*ASSET_ID_1), Just(*ASSET_ID_2)],
        )
            .prop_map(|(amount, asset_id)| {
                Expression::Value(Value {
                    amount: amount.into(),
                    asset_id,
                })
            })
            .prop_recursive(8, 256, 2, |inner| {
                prop_oneof![
                    inner
                        .clone()
                        .prop_map(|beneath| Expression::Neg(Box::new(beneath))),
                    (inner.clone(), inner.clone()).prop_map(|(left, right)| {
                        Expression::Add(Box::new(left), Box::new(right))
                    }),
                    (inner.clone(), inner).prop_map(|(left, right)| {
                        Expression::Sub(Box::new(left), Box::new(right))
                    }),
                ]
            })
    }

    proptest! {
        /// Checks to make sure that any possible expression made of negation, addition, and
        /// subtraction is a homomorphism with regard to the resultant value commitment, which
        /// should provide assurance that these operations are implemented correctly on the balance
        /// type itself.
        #[test]
        fn all_expressions_correct_commitment(
            expr in gen_expression()
        ) {
            // Compute the balance for the expression
            let balance = expr.balance();

            // Compute the transparent commitment for the expression
            let commitment = expr.transparent_value_commitment();

            // Compute the transparent commitment for the balance
            let mut balance_commitment = value::Commitment::default();
            for required in balance.required() {
                balance_commitment = balance_commitment - required.commit(Fr::zero());
            }
            for provided in balance.provided() {
                balance_commitment = balance_commitment + provided.commit(Fr::zero());
            }

            assert_eq!(commitment, balance_commitment);
        }
    }
}
