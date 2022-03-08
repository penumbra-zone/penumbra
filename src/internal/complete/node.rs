use std::cell::Cell;

use crate::{
    internal::height::Succ, internal::three::Three, Complete, GetHash, Hash, Height, Insert,
};

use super::super::active;

pub mod children;
pub use children::Children;

/// A complete sparse node in a tree, storing only the witnessed subtrees.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Node<Child> {
    hash: Cell<Option<Hash>>,
    children: Children<Child>,
}

impl<Child> Node<Child> {
    /// Set the hash of this node without checking to see whether the hash is correct.
    ///
    /// # Correctness
    ///
    /// This should only be called when the hash is already known (i.e. after construction from
    /// children with a known node hash).
    pub(in super::super) fn set_hash_unchecked(&self, hash: Hash) {
        self.hash.set(Some(hash));
    }

    pub(in super::super) fn from_siblings_and_focus_or_else_hash(
        siblings: Three<Insert<Child>>,
        focus: Insert<Child>,
    ) -> Insert<Self>
    where
        Child: Complete,
    {
        todo!("construct `Complete` from siblings and focus")
    }

    pub(in super::super) fn from_children_or_else_hash(children: [Insert<Child>; 4]) -> Insert<Self>
    where
        Child: Complete + GetHash + Height,
    {
        todo!("construct `Complete` from all four children")
    }
}

impl<Child: Height> Height for Node<Child> {
    type Height = Succ<Child::Height>;
}

impl<Child: Complete> Complete for Node<Child> {
    type Focus = active::Node<Child::Focus>;
}

impl<Child> GetHash for Node<Child> {
    #[inline]
    fn hash(&self) -> Hash {
        self.hash.get().unwrap_or_else(|| {
            let hash = todo!("hash children");
            self.hash.set(Some(hash));
            hash
        })
    }

    #[inline]
    fn cached_hash(&self) -> Option<Hash> {
        self.hash.get()
    }
}
