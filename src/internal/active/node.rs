use std::{cell::Cell, fmt::Debug};

use crate::{
    internal::{
        active::{Forget, Full},
        hash::OptionHash,
        height::{IsHeight, Succ},
        path::{self, WhichWay, Witness},
        three::{Elems, ElemsMut, Three},
    },
    Active, AuthPath, Focus, ForgetOwned, GetHash, Hash, Height, Insert,
};

use super::super::complete;

/// An active node in a tree, into which items can be inserted.
#[derive(Clone, Eq, Derivative)]
#[derivative(
    Debug(bound = "Child: Debug, Child::Complete: Debug"),
    PartialEq(bound = "Child: PartialEq, Child::Complete: PartialEq")
)]
pub struct Node<Child: Focus> {
    #[derivative(
        PartialEq = "ignore",
        Debug(format_with = "super::super::complete::node::fmt_cache")
    )]
    hash: Cell<OptionHash>,
    siblings: Three<Insert<Child::Complete>>,
    focus: Child,
}

impl<Child: Focus> PartialEq<complete::Node<Child::Complete>> for Node<Child>
where
    Child::Complete: PartialEq<Child> + PartialEq,
{
    fn eq(&self, other: &complete::Node<Child::Complete>) -> bool {
        let zero = || -> Insert<&Child> { Insert::Hash(Hash::default()) };

        let children = other.children();

        match (self.siblings.elems(), &self.focus) {
            (Elems::_0([]), a) => {
                children[0] == Insert::Keep(a)
                    && children[1] == zero()
                    && children[2] == zero()
                    && children[3] == zero()
            }
            (Elems::_1([a]), b) => {
                children[0] == a.as_ref()
                    && children[1] == Insert::Keep(b)
                    && children[2] == zero()
                    && children[3] == zero()
            }
            (Elems::_2([a, b]), c) => {
                children[0] == a.as_ref()
                    && children[1] == b.as_ref()
                    && children[2] == Insert::Keep(c)
                    && children[3] == zero()
            }
            (Elems::_3([a, b, c]), d) => {
                children[0] == a.as_ref()
                    && children[1] == b.as_ref()
                    && children[2] == c.as_ref()
                    && children[3] == Insert::Keep(d)
            }
        }
    }
}

impl<Child: Focus> Node<Child> {
    pub(crate) fn from_parts(siblings: Three<Insert<Child::Complete>>, focus: Child) -> Self
    where
        Child: Active + GetHash,
    {
        Self {
            hash: Cell::new(None.into()),
            siblings,
            focus,
        }
    }
}

impl<Child: Focus> Height for Node<Child> {
    type Height = Succ<Child::Height>;
}

impl<Child: Focus> GetHash for Node<Child> {
    fn hash(&self) -> Hash {
        // Extract the hashes of an array of `Insert<T>`s.
        fn hashes_of_all<T: GetHash, const N: usize>(full: [&Insert<T>; N]) -> [Hash; N] {
            full.map(|hash_or_t| match hash_or_t {
                Insert::Hash(hash) => *hash,
                Insert::Keep(t) => t.hash(),
            })
        }

        match self.hash.get().into() {
            // If the hash was already cached, return that without doing any more work
            Some(hash) => hash,

            // If the hash was not already cached, compute and cache it
            None => {
                // Get the four hashes of the node's siblings + focus, *in that order*, adding
                // zero-padding when there are less than four elements
                let zero = Hash::default();
                let focus = self.focus.hash();

                let (a, b, c, d) = match self.siblings.elems() {
                    Elems::_0([]) => (focus, zero, zero, zero),
                    Elems::_1(full) => {
                        let [a] = hashes_of_all(full);
                        (a, focus, zero, zero)
                    }
                    Elems::_2(full) => {
                        let [a, b] = hashes_of_all(full);
                        (a, b, focus, zero)
                    }
                    Elems::_3(full) => {
                        let [a, b, c] = hashes_of_all(full);
                        (a, b, c, focus)
                    }
                };

                // Compute the hash of the node based on its height and the height of its children,
                // and cache it in the node
                let hash = Hash::node(Child::Height::HEIGHT + 1, a, b, c, d);
                self.hash.set(Some(hash).into());
                hash
            }
        }
    }

    #[inline]
    fn cached_hash(&self) -> Option<Hash> {
        self.hash.get().into()
    }
}

impl<Child: Focus> Focus for Node<Child> {
    type Complete = complete::Node<Child::Complete>;

    #[inline]
    fn finalize(self) -> Insert<Self::Complete> {
        complete::Node::from_siblings_and_focus_or_else_hash(self.siblings, self.focus.finalize())
    }
}

impl<Focus> Active for Node<Focus>
where
    Focus: Active + GetHash,
{
    type Item = Focus::Item;

    #[inline]
    fn singleton(item: Insert<Self::Item>) -> Self {
        let focus = Focus::singleton(item);
        let siblings = Three::new();
        Self::from_parts(siblings, focus)
    }

    #[inline]
    fn update<T>(&mut self, f: impl FnOnce(&mut Insert<Self::Item>) -> T) -> T {
        let before_hash = self.focus.cached_hash();
        let output = self.focus.update(f);
        let after_hash = self.focus.cached_hash();

        // If the cached hash of the focus changed, clear the cached hash here, because it is now
        // invalid and needs to be recalculated
        if before_hash != after_hash {
            self.hash.set(None.into());
        }

        output
    }

    #[inline]
    fn focus(&self) -> &Insert<Self::Item> {
        self.focus.focus()
    }

    #[inline]
    fn insert(self, item: Insert<Self::Item>) -> Result<Self, Full<Self>> {
        match self.focus.insert(item) {
            // We successfully inserted at the focus, so siblings don't need to be changed
            Ok(focus) => Ok(Self::from_parts(self.siblings, focus)),

            // We couldn't insert at the focus because it was full, so we need to move our path
            // rightwards and insert into a newly created focus
            Err(Full {
                item,
                complete: sibling,
            }) => match self.siblings.push(sibling) {
                // We had enough room to add another sibling, so we set our focus to a new focus
                // containing only the item we couldn't previously insert
                Ok(siblings) => Ok(Self::from_parts(siblings, Focus::singleton(item))),

                // We didn't have enough room to add another sibling, so we return a complete node
                // as a carry, to be propagated up above us and added to some ancestor segment's
                // siblings, along with the item we couldn't insert
                Err(children) => {
                    Err(Full {
                        item,
                        // Implicitly, we only hash the children together when we're pruning them
                        // (because otherwise we would lose that informtion); if at least one child
                        // and its sibling hashes/subtrees is preserved in a `Complete` node, then
                        // we defer calculating the node hash until looking up an authentication path
                        complete: match complete::Node::from_children_or_else_hash(children) {
                            Insert::Hash(hash) => Insert::Hash(hash),
                            Insert::Keep(node) => {
                                if let Some(hash) = self.hash.get().into() {
                                    // This is okay because `complete` is guaranteed to have the same elements in
                                    // the same order as `siblings + [focus]`:
                                    node.set_hash_unchecked(hash);
                                }
                                Insert::Keep(node)
                            }
                        },
                    })
                }
            },
        }
    }
}

impl<Child: Focus + Witness> Witness for Node<Child>
where
    Child::Complete: Witness<Item = Child::Item>,
{
    type Item = Child::Item;

    fn witness(&self, index: impl Into<u64>) -> Option<(AuthPath<Self>, Self::Item)> {
        use Elems::*;
        use WhichWay::*;

        let index = index.into();

        // Which direction should we go from this node?
        let (which_way, index) = WhichWay::at(Self::Height::HEIGHT, index);

        let (siblings, (child, leaf)) = match (self.siblings.elems(), &self.focus) {
            // Zero siblings to the left
            (_0([]), a) => match which_way {
                Leftmost => (
                    // All sibling hashes are default for the left, right, and rightmost
                    [Hash::default(); 3],
                    // Authentication path is to the leftmost child
                    a.witness(index)?,
                ),
                Left | Right | Rightmost => return None,
            },

            // One sibling to the left
            (_1([a]), b) => match which_way {
                Leftmost => (
                    // Sibling hashes are the left child and default for right and rightmost
                    [b.hash(), Hash::default(), Hash::default()],
                    // Authentication path is to the leftmost child
                    a.as_ref().keep()?.witness(index)?,
                ),
                Left => (
                    // Sibling hashes are the leftmost child and default for right and rightmost
                    [a.hash(), Hash::default(), Hash::default()],
                    // Authentication path is to the left child
                    b.witness(index)?,
                ),
                Right | Rightmost => return None,
            },

            // Two siblings to the left
            (_2([a, b]), c) => match which_way {
                Leftmost => (
                    // Sibling hashes are the left child and right child and default for rightmost
                    [b.hash(), c.hash(), Hash::default()],
                    // Authentication path is to the leftmost child
                    a.as_ref().keep()?.witness(index)?,
                ),
                Left => (
                    // Sibling hashes are the leftmost child and right child and default for rightmost
                    [a.hash(), c.hash(), Hash::default()],
                    // Authentication path is to the left child
                    b.as_ref().keep()?.witness(index)?,
                ),
                Right => (
                    // Sibling hashes are the leftmost child and left child and default for rightmost
                    [a.hash(), b.hash(), Hash::default()],
                    // Authentication path is to the right child
                    c.witness(index)?,
                ),
                Rightmost => return None,
            },

            // Three siblings to the left
            (_3([a, b, c]), d) => match which_way {
                Leftmost => (
                    // Sibling hashes are the left child, right child, and rightmost child
                    [b.hash(), c.hash(), d.hash()],
                    // Authentication path is to the leftmost child
                    a.as_ref().keep()?.witness(index)?,
                ),
                Left => (
                    // Sibling hashes are the leftmost child, right child, and rightmost child
                    [a.hash(), c.hash(), d.hash()],
                    // Authentication path is to the left child
                    b.as_ref().keep()?.witness(index)?,
                ),
                Right => (
                    // Sibling hashes are the leftmost child, left child, and rightmost child
                    [a.hash(), b.hash(), d.hash()],
                    // Authentication path is to the right child
                    c.as_ref().keep()?.witness(index)?,
                ),
                Rightmost => (
                    // Sibling hashes are the leftmost child, left child, and right child
                    [a.hash(), b.hash(), c.hash()],
                    // Authentication path is to the rightmost child
                    d.witness(index)?,
                ),
            },
        };

        Some((path::Node { siblings, child }, leaf))
    }
}

impl<Child: Focus + Forget> Forget for Node<Child>
where
    Child::Complete: ForgetOwned,
{
    fn forget(&mut self, index: impl Into<u64>) -> bool {
        use ElemsMut::*;
        use WhichWay::*;

        let index = index.into();

        // Which direction should we forget from this node?
        let (which_way, index) = WhichWay::at(Self::Height::HEIGHT, index);

        match (self.siblings.elems_mut(), &mut self.focus) {
            (_0([]), a) => match which_way {
                Leftmost => a.forget(index),
                Left | Right | Rightmost => false,
            },
            (_1([a]), b) => match which_way {
                Leftmost => a.forget(index),
                Left => b.forget(index),
                Right | Rightmost => false,
            },
            (_2([a, b]), c) => match which_way {
                Leftmost => a.forget(index),
                Left => b.forget(index),
                Right => c.forget(index),
                Rightmost => false,
            },
            (_3([a, b, c]), d) => match which_way {
                Leftmost => a.forget(index),
                Left => b.forget(index),
                Right => c.forget(index),
                Rightmost => d.forget(index),
            },
        }
    }
}
