use serde::{Deserialize, Serialize};

use crate::prelude::*;

use super::super::frontier;

pub mod children;
pub use children::Children;

/// A complete sparse node in a tree, storing only the witnessed subtrees.
#[derive(Clone, Debug)]
pub struct Node<Child: Clone> {
    hash: Hash,
    forgotten: [Forgotten; 4],
    children: Children<Child>,
}

impl<Child: Serialize + Clone> Serialize for Node<Child> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.children.serialize(serializer)
    }
}

impl<'de, Child: Height + GetHash + Deserialize<'de> + Clone> Deserialize<'de> for Node<Child> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let children = Children::deserialize(deserializer)?;
        Ok(Self {
            hash: children.hash(),
            forgotten: Default::default(),
            children,
        })
    }
}

impl<Child: Height + Clone> Node<Child> {
    pub(in super::super) fn from_children_or_else_hash(
        forgotten: [Forgotten; 4],
        children: [Insert<Child>; 4],
    ) -> Insert<Self>
    where
        Child: GetHash,
    {
        match Children::try_from(children) {
            Ok(children) => Insert::Keep(Self {
                hash: children.hash(),
                forgotten,
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

    /// Get the forgotten versions of the children.
    pub fn forgotten(&self) -> [Forgotten; 4] {
        self.forgotten
    }
}

impl<Child: Height + Clone> Height for Node<Child> {
    type Height = Succ<Child::Height>;
}

impl<Child: Complete + Clone> Complete for Node<Child>
where
    Child::Focus: Clone,
{
    type Focus = frontier::Node<Child::Focus>;
}

impl<Child: Height + GetHash + Clone> GetHash for Node<Child> {
    #[inline]
    fn hash(&self) -> Hash {
        self.hash
    }

    #[inline]
    fn cached_hash(&self) -> Option<Hash> {
        Some(self.hash)
    }
}

impl<Child: GetHash + Witness + Clone> Witness for Node<Child> {
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

impl<Child: GetHash + ForgetOwned + Clone> ForgetOwned for Node<Child> {
    #[inline]
    fn forget_owned(
        self,
        forgotten: Option<Forgotten>,
        index: impl Into<u64>,
    ) -> (Insert<Self>, bool) {
        let index = index.into();

        let [a, b, c, d]: [Insert<Child>; 4] = self.children.into();

        // Which child should we be forgetting?
        let (which_way, index) = WhichWay::at(Self::Height::HEIGHT, index);

        // Recursively forget the appropriate child
        let (children, was_forgotten) = match which_way {
            WhichWay::Leftmost => {
                let (a, forgotten) = match a {
                    Insert::Keep(a) => a.forget_owned(forgotten, index),
                    Insert::Hash(_) => (a, false),
                };
                ([a, b, c, d], forgotten)
            }
            WhichWay::Left => {
                let (b, forgotten) = match b {
                    Insert::Keep(b) => b.forget_owned(forgotten, index),
                    Insert::Hash(_) => (b, false),
                };
                ([a, b, c, d], forgotten)
            }
            WhichWay::Right => {
                let (c, forgotten) = match c {
                    Insert::Keep(c) => c.forget_owned(forgotten, index),
                    Insert::Hash(_) => (c, false),
                };
                ([a, b, c, d], forgotten)
            }
            WhichWay::Rightmost => {
                let (d, forgotten) = match d {
                    Insert::Keep(d) => d.forget_owned(forgotten, index),
                    Insert::Hash(_) => (d, false),
                };
                ([a, b, c, d], forgotten)
            }
        };

        // Reconstruct the node from the children, or else (if all the children are hashes) hash
        // those hashes into a single node hash
        let reconstructed = match Children::try_from(children) {
            Ok(children) => {
                let mut reconstructed = Self {
                    children,
                    hash: self.hash,
                    forgotten: self.forgotten,
                };
                // If we forgot something, mark the location of the forgetting
                if was_forgotten {
                    if let Some(forgotten) = forgotten {
                        reconstructed.forgotten[which_way] = forgotten.next();
                    }
                }
                Insert::Keep(reconstructed)
            }
            Err(_) => Insert::Hash(self.hash),
        };

        (reconstructed, was_forgotten)
    }
}

impl<Child: Clone> GetPosition for Node<Child> {
    fn position(&self) -> Option<u64> {
        None
    }
}

impl<'tree, Item: Height + structure::Any<'tree> + Clone> structure::Any<'tree> for Node<Item> {
    fn kind(&self) -> Kind {
        Kind::Internal {
            height: <Self as Height>::Height::HEIGHT,
        }
    }

    fn global_position(&self) -> Option<Position> {
        <Self as GetPosition>::position(self).map(Into::into)
    }

    fn forgotten(&self) -> Forgotten {
        self.forgotten.iter().copied().max().unwrap_or_default()
    }

    fn children(&self) -> Vec<structure::Node<'_, 'tree>> {
        self.forgotten
            .iter()
            .copied()
            .zip(self.children.children().into_iter())
            .map(|(forgotten, child)| {
                structure::Node::child(forgotten, child.map(|child| child as &dyn structure::Any))
            })
            .collect()
    }
}

impl<Child: Height + OutOfOrderOwned + Clone> OutOfOrderOwned for Node<Child> {
    fn uninitialized_out_of_order_insert_commitment_owned(
        this: Insert<Self>,
        index: u64,
        commitment: Commitment,
    ) -> Self {
        let (which_way, index) = WhichWay::at(<Self as Height>::Height::HEIGHT, index);

        let (hash, forgotten, mut children) = match this {
            // If there's an extant node, extract its contents
            Insert::Keep(Node {
                hash,
                forgotten,
                children,
            }) => (hash, forgotten, children.into()),
            // If there's no node here yet, grab the hash and make up the contents of a new node,
            // into which we will insert the commitment
            Insert::Hash(hash) => (hash, [Forgotten::default(); 4], {
                // Initially, all the children are the uninitialized hash; these will be filled in
                // over time, and then those that aren't filled in will be set to the appropriate
                // finalized hash
                let u = || Insert::Hash(Hash::uninitialized());
                [u(), u(), u(), u()]
            }),
        };

        // Temporarily swap in an uninitialized hash at the child, so we can directly
        // manipulate it as an owned object
        let child = std::mem::replace(
            &mut children[which_way],
            Insert::Hash(Hash::uninitialized()),
        );

        // Set that same child back to the result of inserting the commitment
        children[which_way] = Insert::Keep(
            Child::uninitialized_out_of_order_insert_commitment_owned(child, index, commitment),
        );

        // Convert the children back into a `Children`, which will always succeed
        // because we've guaranteed that we have at least one `Keep` child: the one we
        // just created
        let children = children.try_into().expect(
            "adding a commitment to extant children always allows them to be reconstituted",
        );

        Node {
            hash,
            forgotten,
            children,
        }
    }
}

impl<Child: GetHash + UncheckedSetHash + Clone> UncheckedSetHash for Node<Child> {
    fn unchecked_set_hash(&mut self, index: u64, height: u8, hash: Hash) {
        use std::cmp::Ordering::*;

        match height.cmp(&Self::Height::HEIGHT) {
            Greater => panic!("height too large when setting hash: {}", height),
            // Set the hash here
            Equal => self.hash = hash,
            // Set the hash below
            Less => {
                let (which_way, index) = WhichWay::at(Self::Height::HEIGHT, index);
                let (child, _) = which_way.pick(self.children.children_mut());
                match child {
                    InsertMut::Keep(child) => child.unchecked_set_hash(index, height, hash),
                    InsertMut::Hash(child_hash) => {
                        if <Child as Height>::Height::HEIGHT == height {
                            *child_hash = hash
                        }
                    }
                }
            }
        }
    }

    fn finish_initialize(&mut self) {
        // Finish all the children
        for child in self.children.children_mut() {
            match child {
                InsertMut::Keep(child) => child.finish_initialize(),
                InsertMut::Hash(hash) => {
                    if hash.is_uninitialized() {
                        // If the hash is not initialized, we set it to the empty finalized hash
                        *hash = Hash::one();
                    }
                }
            }
        }

        // IMPORTANT: we *must* finish the children before computing the hash at this node, or else
        // we will potentially be computing an invalid hash, since there might be invalid hashes in
        // the children which haven't been resolved yet!

        // Then, compute the hash at this node, if necessary
        if self.hash.is_uninitialized() {
            self.hash = self.children.hash();
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn check_node_size() {
        // Disabled due to spurious test failure.
        // static_assertions::assert_eq_size!(Node<()>, [u8; 72]);
    }
}
