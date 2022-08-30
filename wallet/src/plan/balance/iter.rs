use std::{
    collections::btree_map,
    fmt::{self, Debug, Formatter},
    iter::FusedIterator,
    num::NonZeroU64,
};

use penumbra_crypto::{asset, Value};

use super::{Balance, Imbalance};

impl Balance {
    pub(super) fn iter(&self) -> Iter<'_> {
        Iter {
            negated: self.negated,
            iter: self.balance.iter(),
        }
    }
}

impl IntoIterator for Balance {
    type Item = Imbalance<Value>;
    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            negated: self.negated,
            iter: self.balance.into_iter(),
        }
    }
}

#[derive(Clone)]
pub struct Iter<'a> {
    negated: bool,
    iter: btree_map::Iter<'a, asset::Id, Imbalance<NonZeroU64>>,
}

impl Debug for Iter<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}

impl Iterator for Iter<'_> {
    type Item = Imbalance<Value>;

    fn next(&mut self) -> Option<Self::Item> {
        let (&asset_id, &imbalance) = self.iter.next()?;
        let mut value_imbalance = imbalance.map(move |amount| Value {
            asset_id,
            amount: amount.into(),
        });
        if self.negated {
            value_imbalance = -value_imbalance;
        }
        Some(value_imbalance)
    }
}

impl DoubleEndedIterator for Iter<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let (&asset_id, &imbalance) = self.iter.next_back()?;
        let mut value_imbalance = imbalance.map(move |amount| Value {
            asset_id,
            amount: amount.into(),
        });
        if self.negated {
            value_imbalance = -value_imbalance;
        }
        Some(value_imbalance)
    }
}

impl ExactSizeIterator for Iter<'_> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl FusedIterator for Iter<'_> {}

pub struct IntoIter {
    negated: bool,
    iter: btree_map::IntoIter<asset::Id, Imbalance<NonZeroU64>>,
}

impl Iterator for IntoIter {
    type Item = Imbalance<Value>;

    fn next(&mut self) -> Option<Self::Item> {
        let (asset_id, imbalance) = self.iter.next()?;
        let mut value_imbalance = imbalance.map(move |amount| Value {
            asset_id,
            amount: amount.into(),
        });
        if self.negated {
            value_imbalance = -value_imbalance;
        }
        Some(value_imbalance)
    }
}

impl DoubleEndedIterator for IntoIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        let (asset_id, imbalance) = self.iter.next_back()?;
        let mut value_imbalance = imbalance.map(move |amount| Value {
            asset_id,
            amount: amount.into(),
        });
        if self.negated {
            value_imbalance = -value_imbalance;
        }
        Some(value_imbalance)
    }
}

impl ExactSizeIterator for IntoIter {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl FusedIterator for IntoIter {}
