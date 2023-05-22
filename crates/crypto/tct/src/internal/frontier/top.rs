use std::fmt::Debug;

use serde::{Deserialize, Serialize};

use crate::prelude::*;

use frontier::tier::Nested;

/// The frontier of the top level of some part of the commitment tree, which may be empty, but may
/// not be finalized or hashed.
#[derive(Derivative, Serialize, Deserialize)]
#[derivative(
    Debug(bound = "Item: Debug, Item::Complete: Debug"),
    Clone(bound = "Item: Clone, Item::Complete: Clone")
)]
#[serde(bound(
    serialize = "Item: Serialize, Item::Complete: Serialize",
    deserialize = "Item: Deserialize<'de>, Item::Complete: Deserialize<'de>"
))]
pub struct Top<Item: Focus + Clone>
where
    Item::Complete: Clone,
{
    track_forgotten: TrackForgotten,
    inner: Option<Nested<Item>>,
}

/// Whether or not to track forgotten elements of the tree.
///
/// This is set to `Yes` for trees, but to `No` for epoch and block builders, because when they are
/// inserted all at once, there would be no meaning to their internal forgotten versions, and the
/// tree wouldn't have known about any elements that were forgotten before the builder was inserted,
/// so it doesn't need to track them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrackForgotten {
    /// Do keep track of what things are forgotten.
    Yes,
    /// Do not keep track of what things are forgotten.
    No,
}

impl<Item: Focus + Clone> Top<Item>
where
    Item::Complete: Clone,
{
    /// Create a new top-level frontier tier.
    #[allow(unused)]
    pub fn new(track_forgotten: TrackForgotten) -> Self {
        Self {
            track_forgotten,
            inner: None,
        }
    }

    /// Insert an item or its hash into this frontier tier.
    ///
    /// If the tier is full, return the input item without inserting it.
    #[inline]
    pub fn insert(&mut self, item: Item) -> Result<(), Item> {
        // Temporarily replace the inside with `None` (it will get put back right away, this is just
        // to satisfy the borrow checker)
        let inner = std::mem::take(&mut self.inner);

        let (result, inner) = if let Some(inner) = inner {
            if inner.is_full() {
                // Don't even try inserting when we know it will fail: this means that there is *no
                // implicit finalization* of the frontier, even when it is full
                (Err(item), inner)
            } else {
                // If it's not full, then insert the item into it (which we know will succeed)
                let inner = inner
                    .insert_owned(item)
                    .unwrap_or_else(|_| panic!("frontier is not full, so insert must succeed"));
                (Ok(()), inner)
            }
        } else {
            // If the tier was empty, create a new frontier containing only the inserted item
            let inner = Nested::new(item);
            (Ok(()), inner)
        };

        // Put the inner back
        self.inner = Some(inner);

        result
    }

    /// Forgot the commitment at the given index.
    ///
    /// This isn't an implementation of [`Forget`] because unlike [`Forget`], this doesn't require
    /// an input forgotten version, since it calculates it based on the forgotten versions at this
    /// top level.
    #[inline]
    pub fn forget(&mut self, index: impl Into<u64>) -> bool
    where
        Item: Forget,
        Item::Complete: ForgetOwned,
    {
        let forgotten = self.forgotten();

        if let Some(ref mut inner) = self.inner {
            inner.forget(forgotten, index)
        } else {
            false
        }
    }

    /// Count the number of times something has been forgotten from this tree.
    #[inline]
    pub fn forgotten(&self) -> Option<Forgotten> {
        if let TrackForgotten::Yes = self.track_forgotten {
            Some(
                self.inner
                    .iter()
                    .flat_map(|inner| inner.forgotten().iter().copied())
                    .max()
                    .unwrap_or_default(),
            )
        } else {
            None
        }
    }

    /// Update the currently focused `Item` (i.e. the most-recently-[`insert`](Self::insert)ed one),
    /// returning the result of the function.
    ///
    /// If this top-level tier is empty or the most recently inserted item is a hash, returns
    /// `None`.
    #[inline]
    pub fn update<T>(&mut self, f: impl FnOnce(&mut Item) -> T) -> Option<T> {
        self.inner.as_mut().and_then(|inner| inner.update(f))
    }

    /// Get a reference to the focused `Insert<Item>`, if there is one.
    ///
    /// If this top-level tier is empty or the focus is a hash, returns `None`.
    #[inline]
    pub fn focus(&self) -> Option<&Item> {
        if let Some(ref inner) = self.inner {
            inner.focus()
        } else {
            None
        }
    }

    /// Finalize the top tier into either a summary root hash or a complete tier.
    #[inline]
    pub fn finalize(self) -> Insert<complete::Top<Item::Complete>> {
        if let Some(inner) = self.inner {
            inner.finalize_owned().map(|inner| complete::Top { inner })
        } else {
            // The hash of an empty top-level tier is 1
            Insert::Hash(Hash::one())
        }
    }

    /// Check whether this top-level tier is full.
    #[inline]
    pub fn is_full(&self) -> bool {
        if let Some(ref inner) = self.inner {
            inner.is_full()
        } else {
            false
        }
    }

    /// Check whether this top-level tier is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.inner.is_none()
    }
}

impl<Item: Focus + Clone> Height for Top<Item>
where
    Item::Complete: Clone,
{
    type Height = <Nested<Item> as Height>::Height;
}

impl<Item: Focus + GetPosition + Clone> GetPosition for Top<Item>
where
    Item::Complete: Clone,
{
    #[inline]
    fn position(&self) -> Option<u64> {
        if let Some(ref frontier) = self.inner {
            frontier.position()
        } else {
            Some(0)
        }
    }
}

impl<Item: Focus + Clone> GetHash for Top<Item>
where
    Item::Complete: Clone,
{
    #[inline]
    fn hash(&self) -> Hash {
        if let Some(ref inner) = self.inner {
            inner.hash()
        } else {
            Hash::zero()
        }
    }

    #[inline]
    fn cached_hash(&self) -> Option<Hash> {
        if let Some(ref inner) = self.inner {
            inner.cached_hash()
        } else {
            Some(Hash::zero())
        }
    }
}

impl<Item: Focus + Witness + Clone> Witness for Top<Item>
where
    Item::Complete: Witness + Clone,
{
    fn witness(&self, index: impl Into<u64>) -> Option<(AuthPath<Self>, Hash)> {
        if let Some(ref inner) = self.inner {
            inner.witness(index)
        } else {
            None
        }
    }
}

impl<'tree, Item: Focus + GetPosition + Height + structure::Any<'tree> + Clone>
    structure::Any<'tree> for Top<Item>
where
    Item::Complete: structure::Any<'tree> + Clone,
{
    fn kind(&self) -> Kind {
        Kind::Internal {
            height: <Self as Height>::Height::HEIGHT,
        }
    }

    fn forgotten(&self) -> Forgotten {
        self.forgotten().unwrap_or_default()
    }

    fn children(&'tree self) -> Vec<HashOrNode<'tree>> {
        self.inner
            .as_ref()
            .map(structure::Any::children)
            .unwrap_or_default()
    }
}

impl<Item: Focus + Height + OutOfOrder + Clone> OutOfOrder for Top<Item>
where
    Item::Complete: OutOfOrderOwned + Clone,
{
    fn uninitialized(position: Option<u64>, forgotten: Forgotten) -> Self {
        let inner = if position == Some(0) {
            // If the position is zero, there's no frontier to manifest
            None
        } else {
            // Otherwise, create a frontier
            Some(Nested::uninitialized(position, forgotten))
        };

        Self {
            inner,
            // Track forgotten things by default (we only deserialize entire full trees, which
            // always have this flipped on)
            track_forgotten: TrackForgotten::Yes,
        }
    }

    fn uninitialized_out_of_order_insert_commitment(&mut self, index: u64, commitment: Commitment) {
        if let Some(ref mut inner) = self.inner {
            inner.uninitialized_out_of_order_insert_commitment(index, commitment);
        }
    }
}

impl<Item: Focus + Height + UncheckedSetHash + Clone> UncheckedSetHash for Top<Item>
where
    Item::Complete: UncheckedSetHash + Clone,
{
    fn unchecked_set_hash(&mut self, index: u64, height: u8, hash: Hash) {
        if let Some(ref mut inner) = self.inner {
            inner.unchecked_set_hash(index, height, hash);
        }
    }

    fn finish_initialize(&mut self) {
        if let Some(ref mut inner) = self.inner {
            inner.finish_initialize();
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn position_advances_by_one() {
        let mut top: Top<Item> = Top::new(TrackForgotten::No);
        for expected_position in 0..=(u16::MAX as u64) {
            assert_eq!(top.position(), Some(expected_position));
            top.insert(Hash::zero().into()).unwrap();
        }
        assert_eq!(top.position(), None);
    }
}
