use ark_r1cs_std::prelude::*;
use ark_r1cs_std::uint8::UInt8;
use ark_relations::r1cs::SynthesisError;
use std::{
    collections::{btree_map, BTreeMap},
    fmt::{self, Debug, Formatter},
    iter::FusedIterator,
    mem,
    num::NonZeroU128,
    ops::{Add, AddAssign, Deref, Neg, Sub, SubAssign},
};

use crate::{
    asset::{self, AmountVar, AssetIdVar},
    value::ValueVar,
    Amount, Value,
};

pub mod commitment;
pub use commitment::Commitment;

mod imbalance;
mod iter;
use commitment::VALUE_BLINDING_GENERATOR;
use decaf377::{r1cs::ElementVar, Fq, Fr};
use imbalance::Imbalance;

use self::commitment::BalanceCommitmentVar;

/// A `Balance` is a "vector of [`Value`]s", where some values may be required, while others may be
/// provided. For a transaction to be valid, its balance must be zero.
#[derive(Clone, Eq, Default)]
pub struct Balance {
    negated: bool,
    balance: BTreeMap<asset::Id, Imbalance<NonZeroU128>>,
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

    /// Commit to a [`Balance`] using a provided blinding factor.
    ///
    /// This is like a vectorized [`Value::commit`].
    #[allow(non_snake_case)]
    pub fn commit(&self, blinding_factor: Fr) -> Commitment {
        // Accumulate all the elements for the values
        let mut commitment = decaf377::Element::default();
        for imbalance in self.iter() {
            let (sign, value) = imbalance.into_inner();
            let G_v = value.asset_id.value_generator();

            // Depending on the sign, either subtract or add
            match sign {
                imbalance::Sign::Required => {
                    commitment -= G_v * Fr::from(value.amount);
                }
                imbalance::Sign::Provided => {
                    commitment += G_v * Fr::from(value.amount);
                }
            }
        }

        // Add the blinding factor only once, after the accumulation
        commitment += blinding_factor * VALUE_BLINDING_GENERATOR.deref();
        Commitment(commitment)
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
            let (asset_id, mut imbalance) = if let Some(amount) = NonZeroU128::new(amount.into()) {
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
        if let Some(amount) = NonZeroU128::new(amount.into()) {
            balance.insert(asset_id, Imbalance::Provided(amount));
        }
        Balance {
            negated: false,
            balance,
        }
    }
}

/// Represents a balance in a rank 1 constraint system.
///
/// A balance consists of a number of assets (represented
/// by their asset ID), the amount of each asset, as
/// well as a boolean var that represents their contribution to the
/// transaction's balance.
///
/// True values represent assets that are being provided (positive sign).
/// False values represent assets that are required (negative sign).
#[derive(Clone)]
pub struct BalanceVar {
    pub inner: Vec<(AssetIdVar, (Boolean<Fq>, AmountVar))>,
}

impl AllocVar<Balance, Fq> for BalanceVar {
    fn new_variable<T: std::borrow::Borrow<Balance>>(
        cs: impl Into<ark_relations::r1cs::Namespace<Fq>>,
        f: impl FnOnce() -> Result<T, SynthesisError>,
        mode: ark_r1cs_std::prelude::AllocationMode,
    ) -> Result<Self, SynthesisError> {
        let ns = cs.into();
        let cs = ns.cs();
        let inner1 = f()?;
        let inner = inner1.borrow();
        match mode {
            AllocationMode::Constant => unimplemented!(),
            AllocationMode::Input => unimplemented!(),
            AllocationMode::Witness => {
                if !inner.negated {
                    unimplemented!();
                }

                let mut inner_balance_vars = Vec::new();
                for (asset_id, imbalance) in inner.balance.iter() {
                    let (sign, amount) = imbalance.into_inner();

                    let asset_id_var = AssetIdVar::new_witness(cs.clone(), || Ok(asset_id))?;
                    let amount_var = AmountVar::new_witness(cs.clone(), || {
                        Ok(Amount::from(u128::from(amount)))
                    })?;

                    let boolean_var = match sign {
                        imbalance::Sign::Required => Boolean::constant(false),
                        imbalance::Sign::Provided => Boolean::constant(true),
                    };

                    inner_balance_vars.push((asset_id_var, (boolean_var, amount_var)));
                }

                Ok(BalanceVar {
                    inner: inner_balance_vars,
                })
            }
        }
    }
}

impl From<ValueVar> for BalanceVar {
    fn from(ValueVar { amount, asset_id }: ValueVar) -> Self {
        let mut balance_vec = Vec::new();
        let sign = Boolean::constant(true);
        balance_vec.push((asset_id, (sign, amount)));

        BalanceVar { inner: balance_vec }
    }
}

impl BalanceVar {
    /// Commit to a [`BalanceVar`] using a provided blinding factor.
    ///
    /// This is like a vectorized [`ValueVar::commit`].
    #[allow(non_snake_case)]
    pub fn commit(
        &self,
        blinding_factor: Vec<UInt8<Fq>>,
    ) -> Result<BalanceCommitmentVar, SynthesisError> {
        // Access constraint system ref from one of the balance contributions
        let cs = self
            .inner
            .get(0)
            .expect("at least one contribution to balance")
            .0
            .asset_id
            .cs();

        // Begin by adding the blinding factor only once
        let value_blinding_generator = ElementVar::new_constant(cs, *VALUE_BLINDING_GENERATOR)?;
        let mut commitment =
            value_blinding_generator.scalar_mul_le(blinding_factor.to_bits_le()?.iter())?;

        // Accumulate all the elements for the values
        for (asset_id, (sign, amount)) in self.inner.iter() {
            let G_v = asset_id.value_generator()?;
            // Access the inner `FqVar` on `AmountVar` for scalar mul
            let value_amount = amount.amount.clone();

            // We scalar mul first with value (small), _then_ negate [v]G_v if needed
            let commitment_plus_contribution =
                commitment.clone() + G_v.scalar_mul_le(value_amount.to_bits_le()?.iter())?;
            let commitment_minus_contribution =
                commitment - G_v.scalar_mul_le(value_amount.to_bits_le()?.iter())?;
            commitment = ElementVar::conditionally_select(
                sign,
                &commitment_plus_contribution,
                &commitment_minus_contribution,
            )?;
        }
        Ok(BalanceCommitmentVar { inner: commitment })
    }

    /// Create a balance from a positive [`ValueVar`].
    pub fn from_positive_value_var(value: ValueVar) -> Self {
        value.into()
    }

    /// Create a balance from a negated [`ValueVar`].
    pub fn from_negative_value_var(value: ValueVar) -> Self {
        let mut balance_vec = Vec::new();
        let sign = Boolean::constant(false);
        balance_vec.push((value.asset_id, (sign, value.amount)));

        BalanceVar { inner: balance_vec }
    }
}

impl std::ops::Add for BalanceVar {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        let mut balance_vec = self.inner.clone();
        for (asset_id, (sign, amount)) in other.inner {
            balance_vec.push((asset_id, (sign, amount)));
        }
        BalanceVar { inner: balance_vec }
    }
}

#[cfg(test)]
mod test {
    use crate::{Fr, Zero, STAKING_TOKEN_ASSET_ID};
    use once_cell::sync::Lazy;
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
        fn transparent_balance_commitment(&self) -> Commitment {
            match self {
                Expression::Value(value) => value.commit(Fr::zero()),
                Expression::Neg(expr) => -expr.transparent_balance_commitment(),
                Expression::Add(lhs, rhs) => {
                    lhs.transparent_balance_commitment() + rhs.transparent_balance_commitment()
                }
                Expression::Sub(lhs, rhs) => {
                    lhs.transparent_balance_commitment() - rhs.transparent_balance_commitment()
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
            let commitment = expr.transparent_balance_commitment();

            // Compute the transparent commitment for the balance
            let mut balance_commitment = Commitment::default();
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
