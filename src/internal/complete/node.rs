use crate::{
    internal::height::Succ, internal::three::Three, Complete, GetHash, Hash, Height, Insert,
};

use super::super::active;

/// A complete sparse node in a tree, storing only the witnessed subtrees.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Node<Child> {
    hash: Hash,
    children: [Option<Box<Child>>; 4],
}

impl<Child> Node<Child> {
    /// Only call this when you know what the hash should be!
    pub(crate) fn set_hash_unchecked(&self, hash: Hash) {
        todo!("set the hash");
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
        self.hash
    }

    #[inline]
    fn cached_hash(&self) -> Option<Hash> {
        Some(self.hash)
    }
}
