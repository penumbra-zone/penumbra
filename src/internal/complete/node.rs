use std::cell::Cell;

use crate::{
    internal::{
        hash::OptionHash,
        height::{IsHeight, Succ},
        path::{self, AuthPath, WhichWay},
        three::{IntoElems, Three},
    },
    Complete, GetHash, Hash, Height, Insert, Witness,
};

use super::super::active;

pub mod children;
pub use children::Children;

/// A complete sparse node in a tree, storing only the witnessed subtrees.
#[derive(Clone, Eq, Derivative)]
#[derivative(Debug, PartialEq(bound = "Child: PartialEq"))]
pub struct Node<Child> {
    #[derivative(PartialEq = "ignore")]
    #[derivative(Debug(format_with = "fmt_cache"))]
    hash: Cell<OptionHash>,
    children: Children<Child>,
}

/// Concisely format `OptionHash` for debug output.
pub(crate) fn fmt_cache(
    cell: &Cell<OptionHash>,
    f: &mut std::fmt::Formatter,
) -> Result<(), std::fmt::Error> {
    if let Some(hash) = <Option<Hash>>::from(cell.get()) {
        write!(f, "{:?}", hash)
    } else {
        write!(f, "_")
    }
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
        self.hash.set(Some(hash).into());
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
                hash: Cell::new(None.into()),
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
        self.cached_hash().unwrap_or_else(|| {
            let [a, b, c, d] = self.children.children().map(|x| x.hash());
            let hash = Hash::node(<Self as Height>::Height::HEIGHT, a, b, c, d);
            self.hash.set(Some(hash).into());
            hash
        })
    }

    #[inline]
    fn cached_hash(&self) -> Option<Hash> {
        self.hash.get().into()
    }
}

impl<Child: GetHash + Witness> Witness for Node<Child> {
    type Item = Child::Item;

    fn witness(&self, index: usize) -> Option<(AuthPath<Self>, Self::Item)> {
        let [a, b, c, d] = self.children();

        // Which way to go down the tree from this node
        let which_way = WhichWay::at(Self::Height::HEIGHT, index);

        // The index to use when witnessing the child: mask off all the bits for the parent nodes of
        // the path above us
        let index = index & (0b11 << ((Self::Height::HEIGHT - 1) * 2));

        let (siblings, (child, leaf)) = match which_way {
            WhichWay::Leftmost => (
                // Siblings are the left, right, and rightmost children
                [b, c, d].map(|x| x.hash()),
                // Authentication path to the leftmost child
                a.keep()?.witness(index)?,
            ),
            WhichWay::Left => (
                // Siblings are the leftmost, right, and rightmost children
                [a, c, d].map(|x| x.hash()),
                // Authentication path to the left child
                b.keep()?.witness(index)?,
            ),
            WhichWay::Right => (
                // Siblings are the leftmost, left, and rightmost children
                [a, b, d].map(|x| x.hash()),
                // Authentication path to the right child
                c.keep()?.witness(index)?,
            ),
            WhichWay::Rightmost => (
                // Siblings are the leftmost, left, and right children
                [a, b, c].map(|x| x.hash()),
                // Authentication path to the rightmost child
                d.keep()?.witness(index)?,
            ),
        };

        Some((path::Node { siblings, child }, leaf))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn check_node_size() {
        static_assertions::assert_eq_size!(Node<()>, [u8; 48]);
    }
}
