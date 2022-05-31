use serde::{Deserialize, Serialize};

use crate::prelude::*;

use super::super::frontier;

pub mod children;
pub use children::Children;

/// A complete sparse node in a tree, storing only the witnessed subtrees.
#[derive(Clone, Debug)]
pub struct Node<Child> {
    hash: Hash,
    children: Children<Child>,
}

impl<Child: Height + GetHash> From<Children<Child>> for Node<Child> {
    fn from(children: Children<Child>) -> Self {
        Self {
            hash: children.hash(),
            children,
        }
    }
}

impl<Child: Serialize> Serialize for Node<Child> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.children.serialize(serializer)
    }
}

impl<'de, Child: Height + GetHash + Deserialize<'de>> Deserialize<'de> for Node<Child> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let children = Children::deserialize(deserializer)?;
        Ok(children.into())
    }
}

impl<Child: GetHash + Height> Node<Child> {
    pub(in super::super) fn from_siblings_and_focus_or_else_hash(
        siblings: Three<Insert<Child>>,
        focus: Insert<Child>,
    ) -> Insert<Self> {
        let one = || Insert::Hash(Hash::one());

        // Push the focus into the siblings, and fill any empty children with the *ONE* hash, which
        // causes the hash of a complete node to deliberately differ from that of a frontier node,
        // which uses *ZERO* padding
        Self::from_children_or_else_hash(match siblings.push(focus) {
            Err([a, b, c, d]) => [a, b, c, d],
            Ok(siblings) => match siblings.into_elems() {
                IntoElems::_3([a, b, c]) => [a, b, c, one()],
                IntoElems::_2([a, b]) => [a, b, one(), one()],
                IntoElems::_1([a]) => [a, one(), one(), one()],
                IntoElems::_0([]) => [one(), one(), one(), one()],
            },
        })
    }

    pub(in super::super) fn from_children_or_else_hash(
        children: [Insert<Child>; 4],
    ) -> Insert<Self> {
        match Children::try_from(children) {
            Ok(children) => Insert::Keep(children.into()),
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
    type Focus = frontier::Node<Child::Focus>;
}

impl<Child: Height + GetHash> GetHash for Node<Child> {
    #[inline]
    fn hash(&self) -> Hash {
        self.hash
    }

    #[inline]
    fn cached_hash(&self) -> Option<Hash> {
        Some(self.hash)
    }
}

impl<Child: GetHash + Witness> Witness for Node<Child> {
    #[inline]
    fn witness(&self, index: impl Into<u64>) -> Option<(AuthPath<Self>, Hash)> {
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

impl<Child: GetHash + ForgetOwned> ForgetOwned for Node<Child> {
    #[inline]
    fn forget_owned(self, index: impl Into<u64>) -> (Insert<Self>, bool) {
        let index = index.into();

        let [a, b, c, d]: [Insert<Child>; 4] = self.children.into();

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

        (reconstructed, forgotten)
    }
}

impl<Item: Height + Any> Any for Node<Item> {
    fn place(&self) -> Place {
        Place::Complete
    }

    fn kind(&self) -> Kind {
        Kind::Node
    }

    fn height(&self) -> u8 {
        <Self as Height>::Height::HEIGHT
    }

    fn children(&self) -> Vec<Insert<Child>> {
        self.children
            .children()
            .into_iter()
            .map(|child| child.map(|child| Child::new(child)))
            .collect()
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
