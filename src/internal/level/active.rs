use std::{fmt::Debug, mem};

use crate::{internal::Full, Active as _, Focus, GetHash, Hash, Height, Insert};

#[derive(Derivative)]
#[derivative(Debug(bound = "Item: Debug, <Item as Focus>::Complete: Debug"))]
#[derivative(Clone(bound = "Item: Clone, <Item as Focus>::Complete: Clone"))]
#[derivative(PartialEq(bound = "Item: PartialEq, <Item as Focus>::Complete: PartialEq"))]
#[derivative(Eq(bound = "Item: Eq, <Item as Focus>::Complete: Eq"))]
pub struct Active<Item: Focus> {
    inner: Inner<Item>,
}

type N<Focus> = super::super::node::Active<Focus>;
type L<Item> = super::super::leaf::Active<Item>;

/// An eight-deep active tree with the given item stored in each leaf.
pub(super) type Nested<Item> = N<N<N<N<N<N<N<N<L<Item>>>>>>>>>;
// You can count the levels:   1 2 3 4 5 6 7 8

/// The inside of an active level.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Inner<Item: Focus> {
    /// The starting state: an empty tree.
    Empty,
    /// A tree with at least one element in it, which could potentially allow more to be inserted.
    Active(Nested<Item>),
    /// A tree which has been filled, so no more elements can be inserted.
    ///
    /// This is one of two final states; the other is [`Inner::Hash`].
    Complete(super::complete::Nested<Item::Complete>),
    /// A tree which has been filled, but which witnessed no elements, so it is represented by a
    /// single hash.
    ///
    /// This is one of two final states: the other is [`Inner::Complete`].
    Hash(Hash),
}

impl<Item: Focus> Default for Inner<Item> {
    fn default() -> Self {
        Inner::Empty
    }
}

impl<Item: Focus> Active<Item> {
    pub fn new() -> Self {
        Self {
            inner: Inner::default(),
        }
    }

    pub fn insert(&mut self, item: Insert<Item>) -> Result<(), Insert<Item>> {
        match mem::take(&mut self.inner) {
            Inner::Empty => {
                self.inner = Inner::Active(Nested::singleton(item));
                Ok(())
            }
            Inner::Active(active) => match active.insert(item) {
                Ok(active) => {
                    self.inner = Inner::Active(active);
                    Ok(())
                }
                Err(Full { item, complete }) => {
                    self.inner = match complete {
                        Insert::Hash(hash) => Inner::Hash(hash),
                        Insert::Keep(complete) => Inner::Complete(complete),
                    };
                    Err(item)
                }
            },
            Inner::Complete(complete) => {
                self.inner = Inner::Complete(complete);
                Err(item)
            }
            Inner::Hash(hash) => {
                self.inner = Inner::Hash(hash);
                Err(item)
            }
        }
    }
}

impl<Item: Focus> Default for Active<Item> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Item: Focus> Height for Active<Item> {
    type Height = <Nested<Item> as Height>::Height;
}

impl<Item: Focus> GetHash for Active<Item> {
    fn hash(&self) -> Hash {
        match &self.inner {
            Inner::Empty => Hash::empty_tree(),
            Inner::Active(active) => active.hash(),
            Inner::Complete(complete) => complete.hash(),
            Inner::Hash(hash) => *hash,
        }
    }
}

impl<Item: Focus> Focus for Active<Item> {
    type Complete = super::Complete<Item::Complete>;

    #[inline]
    fn finalize(self) -> Insert<Self::Complete> {
        match self.inner {
            Inner::Empty => Insert::Hash(Hash::empty_tree()),
            Inner::Active(active) => match active.finalize() {
                Insert::Hash(hash) => Insert::Hash(hash),
                Insert::Keep(inner) => Insert::Keep(super::Complete { inner }),
            },
            Inner::Complete(inner) => Insert::Keep(super::Complete { inner }),
            Inner::Hash(hash) => Insert::Hash(hash),
        }
    }
}
