use std::cell::Cell;

use serde::{Deserialize, Serialize};

use crate::{
    internal::{
        hash::{self, OptionHash},
        height::{IsHeight, Succ},
        path::{self, AuthPath, WhichWay, Witness},
        three::{IntoElems, Three},
    },
    Complete, ForgetOwned, GetHash, Hash, Height, Insert,
};

use super::super::active;

pub mod children;
pub use children::Children;

/// A complete sparse node in a tree, storing only the witnessed subtrees.
#[derive(Clone, Eq, Derivative, Serialize, Deserialize)]
#[derivative(Debug, PartialEq(bound = "Child: PartialEq"))]
pub struct Node<Child, Hasher> {
    #[derivative(PartialEq = "ignore")]
    #[derivative(Debug(format_with = "fmt_cache"))]
    #[serde(skip)]
    hash: Cell<OptionHash<Hasher>>,
    children: Children<Child, Hasher>,
}

/// Concisely format `OptionHash` for debug output.
pub(crate) fn fmt_cache<Hasher>(
    cell: &Cell<OptionHash<Hasher>>,
    f: &mut std::fmt::Formatter,
) -> Result<(), std::fmt::Error> {
    if let Some(hash) = <Option<Hash<Hasher>>>::from(cell.get()) {
        write!(f, "{:?}", hash)
    } else {
        write!(f, "_")
    }
}

impl<Child: Complete<Hasher>, Hasher> PartialEq<active::Node<Child::Focus, Hasher>>
    for Node<Child, Hasher>
where
    Child: PartialEq + PartialEq<Child::Focus>,
{
    fn eq(&self, other: &active::Node<Child::Focus, Hasher>) -> bool {
        other == self
    }
}

impl<Child: Height + GetHash<Hasher>, Hasher> Node<Child, Hasher> {
    /// Set the hash of this node without checking to see whether the hash is correct.
    ///
    /// # Correctness
    ///
    /// This should only be called when the hash is already known (i.e. after construction from
    /// children with a known node hash).
    pub(in super::super) fn set_hash_unchecked(&self, hash: Hash<Hasher>) {
        self.hash.set(Some(hash).into());
    }

    pub(in super::super) fn from_siblings_and_focus_or_else_hash(
        siblings: Three<Insert<Child, Hasher>>,
        focus: Insert<Child, Hasher>,
    ) -> Insert<Self, Hasher>
    where
        Hasher: hash::Hasher,
    {
        fn zero<T: GetHash<Hasher>, Hasher>() -> Insert<T, Hasher> {
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
        children: [Insert<Child, Hasher>; 4],
    ) -> Insert<Self, Hasher>
    where
        Hasher: hash::Hasher,
    {
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
    pub fn children(&self) -> [Insert<&Child, Hasher>; 4] {
        self.children.children()
    }
}

impl<Child: Height, Hasher> Height for Node<Child, Hasher> {
    type Height = Succ<Child::Height>;
}

impl<Child: Complete<Hasher>, Hasher: hash::Hasher> Complete<Hasher> for Node<Child, Hasher> {
    type Focus = active::Node<Child::Focus, Hasher>;
}

impl<Child: Height + GetHash<Hasher>, Hasher: hash::Hasher> GetHash<Hasher>
    for Node<Child, Hasher>
{
    #[inline]
    fn hash(&self) -> Hash<Hasher> {
        self.cached_hash().unwrap_or_else(|| {
            let [a, b, c, d] = self.children.children().map(|x| x.hash());
            let hash = Hash::node(<Self as Height>::Height::HEIGHT, a, b, c, d);
            self.hash.set(Some(hash).into());
            hash
        })
    }

    #[inline]
    fn cached_hash(&self) -> Option<Hash<Hasher>> {
        self.hash.get().into()
    }
}

impl<Child: GetHash<Hasher> + Witness<Hasher>, Hasher: hash::Hasher> Witness<Hasher>
    for Node<Child, Hasher>
where
    Child::Height: path::Path<Hasher>,
{
    type Item = Child::Item;

    #[inline]
    fn witness(&self, index: impl Into<u64>) -> Option<(AuthPath<Self, Hasher>, Self::Item)> {
        let index = index.into();

        // Which way to go down the tree from this node
        let (which_way, index) = WhichWay::at(Self::Height::HEIGHT, index);

        // Select the child we should be witnessing
        let (child, siblings) = which_way.pick(self.children());

        // Hash all the other siblings
        let siblings = siblings.map(|sibling| sibling.hash());

        // Witness the selected child
        let (child, leaf) = child.keep()?.witness(index)?;

        Some((path::Node { siblings, child }, leaf))
    }
}

impl<Child: ForgetOwned<Hasher>, Hasher: hash::Hasher> ForgetOwned<Hasher> for Node<Child, Hasher> {
    #[inline]
    fn forget_owned(self, index: impl Into<u64>) -> (Insert<Self, Hasher>, bool) {
        let index = index.into();

        let [a, b, c, d]: [Insert<Child, Hasher>; 4] = self.children.into();

        // Which child should we be forgetting?
        let (which_way, index) = WhichWay::at(Self::Height::HEIGHT, index);

        // Recursively forget the appropriate child
        let (children, forgotten) = match which_way {
            WhichWay::Leftmost => {
                let (a, forgotten) = match a {
                    Insert::Keep(a) => a.forget_owned(index),
                    Insert::Hash(_) => (a, false),
                };
                ([a, b, c, d], forgotten)
            }
            WhichWay::Left => {
                let (b, forgotten) = match b {
                    Insert::Keep(b) => b.forget_owned(index),
                    Insert::Hash(_) => (b, false),
                };
                ([a, b, c, d], forgotten)
            }
            WhichWay::Right => {
                let (c, forgotten) = match c {
                    Insert::Keep(c) => c.forget_owned(index),
                    Insert::Hash(_) => (c, false),
                };
                ([a, b, c, d], forgotten)
            }
            WhichWay::Rightmost => {
                let (d, forgotten) = match d {
                    Insert::Keep(d) => d.forget_owned(index),
                    Insert::Hash(_) => (d, false),
                };
                ([a, b, c, d], forgotten)
            }
        };

        // Reconstruct the node from the children, or else (if all the children are hashes) hash
        // those hashes into a single node hash
        let reconstructed = Self::from_children_or_else_hash(children);

        // If the node was reconstructed, we know that its hash should not have changed, so carry
        // over the old cached hash, if any existed, to prevent recomputation
        let reconstructed = reconstructed.map(|node| {
            if let Some(hash) = self.hash.get().into() {
                node.set_hash_unchecked(hash);
            }
            node
        });

        (reconstructed, forgotten)
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
