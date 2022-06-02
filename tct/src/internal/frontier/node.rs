use std::fmt::Debug;

use serde::{Deserialize, Serialize};

use crate::prelude::*;

/// A frontier of a node in a tree, into which items can be inserted.
#[derive(Clone, Derivative, Serialize, Deserialize)]
#[serde(bound(serialize = "Child: Serialize, Child::Complete: Serialize"))]
#[serde(bound(deserialize = "Child: Deserialize<'de>, Child::Complete: Deserialize<'de>"))]
#[derivative(Debug(bound = "Child: Debug, Child::Complete: Debug"))]
pub struct Node<Child: Focus> {
    #[derivative(PartialEq = "ignore", Debug)]
    #[serde(skip)]
    hash: CachedHash,
    #[serde(skip)]
    forgotten: [Forgotten; 4],
    siblings: Three<Insert<Child::Complete>>,
    focus: Child,
}

impl<Child: Focus> Node<Child> {
    /// Construct a new node from parts.
    pub(crate) fn from_parts(
        forgotten: [Forgotten; 4],
        siblings: Three<Insert<Child::Complete>>,
        focus: Child,
    ) -> Self
    where
        Child: Frontier + GetHash,
    {
        Self {
            hash: Default::default(),
            forgotten,
            siblings,
            focus,
        }
    }

    /// Get the list of forgotten counts for the children of this node.
    #[inline]
    pub(crate) fn forgotten(&self) -> &[Forgotten; 4] {
        &self.forgotten
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

        self.hash.set_if_empty(|| {
            // Get the four hashes of the node's siblings + focus, *in that order*, adding
            // zero-padding when there are less than four elements
            let zero = Hash::zero();
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
            Hash::node(<Self as Height>::Height::HEIGHT, a, b, c, d)
        })
    }

    #[inline]
    fn cached_hash(&self) -> Option<Hash> {
        self.hash.get()
    }

    #[inline]
    fn clear_cached_hash(&self) {
        self.hash.clear();
    }
}

impl<Child: Focus> Focus for Node<Child> {
    type Complete = complete::Node<Child::Complete>;

    #[inline]
    fn finalize_owned(self) -> Insert<Self::Complete> {
        let one = || Insert::Hash(Hash::one());

        // Push the focus into the siblings, and fill any empty children with the *ONE* hash, which
        // causes the hash of a complete node to deliberately differ from that of a frontier node,
        // which uses *ZERO* padding
        complete::Node::from_children_or_else_hash(
            self.forgotten,
            match self.siblings.push(self.focus.finalize_owned()) {
                Err([a, b, c, d]) => [a, b, c, d],
                Ok(siblings) => match siblings.into_elems() {
                    IntoElems::_3([a, b, c]) => [a, b, c, one()],
                    IntoElems::_2([a, b]) => [a, b, one(), one()],
                    IntoElems::_1([a]) => [a, one(), one(), one()],
                    IntoElems::_0([]) => [one(), one(), one(), one()],
                },
            },
        )
    }
}

impl<Child> Frontier for Node<Child>
where
    Child: Focus + Frontier + GetHash,
{
    type Item = Child::Item;

    #[inline]
    fn new(item: Self::Item) -> Self {
        let focus = Child::new(item);
        let siblings = Three::new();
        Self::from_parts(Default::default(), siblings, focus)
    }

    #[inline]
    fn update<T>(&mut self, f: impl FnOnce(&mut Self::Item) -> T) -> Option<T> {
        let before_hash = self.focus.cached_hash();
        let output = self.focus.update(f);
        let after_hash = self.focus.cached_hash();

        // If the cached hash of the focus changed, clear the cached hash here, because it is now
        // invalid and needs to be recalculated
        if before_hash != after_hash {
            self.hash = CachedHash::default();
        }

        output
    }

    #[inline]
    fn focus(&self) -> Option<&Self::Item> {
        self.focus.focus()
    }

    #[inline]
    fn insert_owned(self, item: Self::Item) -> Result<Self, Full<Self>> {
        match self.focus.insert_owned(item) {
            // We successfully inserted at the focus, so siblings don't need to be changed
            Ok(focus) => Ok(Self::from_parts(self.forgotten, self.siblings, focus)),

            // We couldn't insert at the focus because it was full, so we need to move our path
            // rightwards and insert into a newly created focus
            Err(Full {
                item,
                complete: sibling,
            }) => match self.siblings.push(sibling) {
                // We had enough room to add another sibling, so we set our focus to a new focus
                // containing only the item we couldn't previously insert
                Ok(siblings) => Ok(Self::from_parts(self.forgotten, siblings, Child::new(item))),

                // We didn't have enough room to add another sibling, so we return a complete node
                // as a carry, to be propagated up above us and added to some ancestor segment's
                // siblings, along with the item we couldn't insert
                Err(children) => Err(Full {
                    item,
                    complete: complete::Node::from_children_or_else_hash(self.forgotten, children),
                }),
            },
        }
    }

    #[inline]
    fn is_full(&self) -> bool {
        self.siblings.is_full() && self.focus.is_full()
    }
}

impl<Child: Focus + GetPosition> GetPosition for Node<Child> {
    #[inline]
    fn position(&self) -> Option<u64> {
        let child_capacity: u64 = 4u64.pow(Child::Height::HEIGHT.into());
        let siblings = self.siblings.len() as u64;

        if let Some(focus_position) = self.focus.position() {
            // next insertion would be at: siblings * 4^height + focus_position
            // because we don't need to add a new child
            Some(siblings * child_capacity + focus_position)
        } else if siblings + 1 < 4
        /* this means adding a new child is possible */
        {
            // next insertion would be at: (siblings + 1) * 4^height
            // because we have to add a new child, and we can
            Some((siblings + 1) * child_capacity)
        } else {
            None
        }
    }
}

impl<Child: Focus + Witness> Witness for Node<Child>
where
    Child::Complete: Witness,
{
    fn witness(&self, index: impl Into<u64>) -> Option<(AuthPath<Self>, Hash)> {
        use Elems::*;
        use WhichWay::*;

        let index = index.into();

        // The zero padding hash for frontier nodes
        let zero = Hash::zero();

        // Which direction should we go from this node?
        let (which_way, index) = WhichWay::at(Self::Height::HEIGHT, index);

        let (siblings, (child, leaf)) = match (self.siblings.elems(), &self.focus) {
            // Zero siblings to the left
            (_0([]), a) => match which_way {
                Leftmost => (
                    // All sibling hashes are default for the left, right, and rightmost
                    [zero; 3],
                    // Authentication path is to the leftmost child
                    a.witness(index)?,
                ),
                Left | Right | Rightmost => return None,
            },

            // One sibling to the left
            (_1([a]), b) => match which_way {
                Leftmost => (
                    // Sibling hashes are the left child and default for right and rightmost
                    [b.hash(), zero, zero],
                    // Authentication path is to the leftmost child
                    a.as_ref().keep()?.witness(index)?,
                ),
                Left => (
                    // Sibling hashes are the leftmost child and default for right and rightmost
                    [a.hash(), zero, zero],
                    // Authentication path is to the left child
                    b.witness(index)?,
                ),
                Right | Rightmost => return None,
            },

            // Two siblings to the left
            (_2([a, b]), c) => match which_way {
                Leftmost => (
                    // Sibling hashes are the left child and right child and default for rightmost
                    [b.hash(), c.hash(), zero],
                    // Authentication path is to the leftmost child
                    a.as_ref().keep()?.witness(index)?,
                ),
                Left => (
                    // Sibling hashes are the leftmost child and right child and default for rightmost
                    [a.hash(), c.hash(), zero],
                    // Authentication path is to the left child
                    b.as_ref().keep()?.witness(index)?,
                ),
                Right => (
                    // Sibling hashes are the leftmost child and left child and default for rightmost
                    [a.hash(), b.hash(), zero],
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
    fn forget(&mut self, forgotten: Option<Forgotten>, index: impl Into<u64>) -> bool {
        use ElemsMut::*;
        use WhichWay::*;

        let index = index.into();

        // Which direction should we forget from this node?
        let (which_way, index) = WhichWay::at(Self::Height::HEIGHT, index);

        let was_forgotten = match (self.siblings.elems_mut(), &mut self.focus) {
            (_0([]), a) => match which_way {
                Leftmost => a.forget(forgotten, index),
                Left | Right | Rightmost => false,
            },
            (_1([a]), b) => match which_way {
                Leftmost => a.forget(forgotten, index),
                Left => b.forget(forgotten, index),
                Right | Rightmost => false,
            },
            (_2([a, b]), c) => match which_way {
                Leftmost => a.forget(forgotten, index),
                Left => b.forget(forgotten, index),
                Right => c.forget(forgotten, index),
                Rightmost => false,
            },
            (_3([a, b, c]), d) => match which_way {
                Leftmost => a.forget(forgotten, index),
                Left => b.forget(forgotten, index),
                Right => c.forget(forgotten, index),
                Rightmost => d.forget(forgotten, index),
            },
        };

        // If we forgot something, mark the location at which we forgot it
        if was_forgotten {
            if let Some(forgotten) = forgotten {
                self.forgotten[which_way] = forgotten.next();
            }
        }

        was_forgotten
    }
}

impl<Item: Focus + GetPosition + Height + structure::Node> structure::Node for Node<Item>
where
    Item::Complete: structure::Node,
{
    fn kind(&self) -> Kind {
        Kind::Internal(<Self as Height>::Height::HEIGHT)
    }

    fn global_position(&self) -> Option<u64> {
        <Self as GetPosition>::position(self)
    }

    fn forgotten(&self) -> Forgotten {
        self.forgotten().iter().copied().max().unwrap_or_default()
    }

    fn children(&self) -> Vec<Child> {
        self.forgotten
            .iter()
            .copied()
            .zip(
                self.siblings
                    .iter()
                    .map(|child| child.as_ref().map(|child| child as &dyn structure::Node))
                    .chain(std::iter::once(Insert::Keep(
                        &self.focus as &dyn structure::Node,
                    ))),
            )
            .map(|(forgotten, child)| Child::new(self, forgotten, child))
            .collect()
    }
}
