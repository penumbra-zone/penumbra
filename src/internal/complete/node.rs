use std::cell::Cell;

use crate::{
    internal::height::{IsHeight, Succ},
    internal::three::{IntoElems, Three},
    Complete, GetHash, Hash, Height, Insert,
};

use super::super::active;

pub mod children;
pub use children::Children;

/// A complete sparse node in a tree, storing only the witnessed subtrees.
#[derive(Debug, Clone, Eq, Derivative)]
#[derivative(PartialEq(bound = "Child: PartialEq"))]
pub struct Node<Child> {
    #[derivative(PartialEq = "ignore")]
    hash: Cell<Option<Hash>>,
    children: Children<Child>,
}

impl<Child: Complete> PartialEq<active::Node<Child::Focus>> for Node<Child>
where
    Child: PartialEq + PartialEq<Child::Focus>,
{
    fn eq(&self, other: &active::Node<Child::Focus>) -> bool {
        other == self
    }
}

impl<Child: Height> Node<Child> {
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
    ) -> Insert<Self> {
        fn zero<T>() -> Insert<T> {
            Insert::Hash(Hash::default())
        }

        // Push the focus into the siblings, and fill any empty children with the zero hash
        Self::from_children_or_else_hash(match siblings.push(focus) {
            Err([a, b, c, d]) => [a, b, c, d],
            Ok(siblings) => match siblings.into_elems() {
                IntoElems::_3([a, b, c]) => [a, b, c, zero()],
                IntoElems::_2([a, b]) => [a, b, zero(), zero()],
                IntoElems::_1([a]) => [a, zero(), zero(), zero()],
                IntoElems::_0([]) => [zero(), zero(), zero(), zero()],
            },
        })
    }

    pub(in super::super) fn from_children_or_else_hash(
        children: [Insert<Child>; 4],
    ) -> Insert<Self> {
        match Children::try_from(children) {
            Ok(children) => Insert::Keep(Self {
                hash: Cell::new(None),
                children,
            }),
            Err([a, b, c, d]) => {
                // If there were no witnessed children, compute a hash for this node based on the
                // node's height and the hashes of its children.
                Insert::Hash(Hash::node(<Self as Height>::Height::HEIGHT, a, b, c, d))
            }
        }
    }

    /// Get the children of this node as an array of either children or hashes.
    pub fn children(&self) -> [Insert<&Child>; 4] {
        self.children.children()
    }
}

impl<Child: Height> Height for Node<Child> {
    type Height = Succ<Child::Height>;
}

impl<Child: Complete> Complete for Node<Child> {
    type Focus = active::Node<Child::Focus>;
}

impl<Child: Height + GetHash> GetHash for Node<Child> {
    #[inline]
    fn hash(&self) -> Hash {
        self.hash.get().unwrap_or_else(|| {
            let [a, b, c, d] = self.children.children().map(|x| x.hash());
            let hash = Hash::node(<Self as Height>::Height::HEIGHT, a, b, c, d);
            self.hash.set(Some(hash));
            hash
        })
    }

    #[inline]
    fn cached_hash(&self) -> Option<Hash> {
        self.hash.get()
    }
}
