use std::{collections::VecDeque, fmt::Debug};

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

impl<Child: Focus + GetPosition + Height + structure::Any> structure::Any for Node<Child>
where
    Child::Complete: structure::Any,
{
    fn kind(&self) -> Kind {
        Kind::Internal {
            height: <Self as Height>::Height::HEIGHT,
        }
    }

    fn global_position(&self) -> Option<Position> {
        <Self as GetPosition>::position(self).map(Into::into)
    }

    fn forgotten(&self) -> Forgotten {
        self.forgotten().iter().copied().max().unwrap_or_default()
    }

    fn children(&self) -> Vec<structure::Node> {
        self.forgotten
            .iter()
            .copied()
            .zip(
                self.siblings
                    .iter()
                    .map(|child| child.as_ref().map(|child| child as &dyn structure::Any))
                    .chain(std::iter::once(Insert::Keep(
                        &self.focus as &dyn structure::Any,
                    ))),
            )
            .map(|(forgotten, child)| structure::Node::child(forgotten, child))
            .collect()
    }
}

/// A builder for a frontier node.
pub struct Builder<Child: Built + Focus>
where
    Child::Complete: Built,
{
    global_position: u64,
    index: u64,
    // This is populated when we first get a `Node` instruction, to the size requested
    remaining: Option<Remaining<Child>>,
    // Then as we complete siblings, we push them onto this
    siblings: Three<Insert<Child::Complete>>,
}

struct Remaining<Child: Built + Focus>
where
    Child::Complete: Built,
{
    hash: Option<Hash>,
    siblings: VecDeque<<Child::Complete as Built>::Builder>,
    focus: Option<<Child as Built>::Builder>,
}

impl<Child: Focus + Built> Built for Node<Child>
where
    Child::Complete: Built,
{
    type Builder = Builder<Child>;

    fn build(global_position: u64, index: u64) -> Self::Builder {
        Builder {
            global_position,
            index,
            siblings: Three::new(),
            remaining: None,
        }
    }
}

impl<Child: Focus + Built> Build for Builder<Child>
where
    Child::Complete: Built,
{
    type Output = Node<Child>;

    fn go(mut self, instruction: Instruction) -> Result<IResult<Self>, HitBottom<Self>> {
        use {IResult::*, Instruction::*};

        // We have started construction of the children...
        if let Some(ref mut remaining) = self.remaining {
            // If there's a sibling builder remaining, we should direct the instruction to that builder
            if let Some(front_sibling) = remaining.siblings.pop_front() {
                // If the instruction is a `Leaf` instruction and we haven't started construction of
                // the front sibling, we want to skip the construction of the front sibling and just
                // abbreviate it with a hash -- but if we've already started construction, then the
                // instruction should just be forwarded as usual
                let mut skipped = false;
                if !front_sibling.is_started() {
                    if let Leaf { here } = instruction {
                        self.siblings.push_mut(Insert::Hash(Hash::new(here)));
                        skipped = true;
                    }
                };

                // If we didn't skip constructing this sibling, then we should forward the
                // instruction to the front sibling builder
                if !skipped {
                    match front_sibling.go(instruction) {
                        Err(HitBottom(incomplete)) => {
                            // We bounced off the bottom, so restore this sibling builder and error
                            remaining.siblings.push_front(incomplete);
                            return Err(HitBottom(self));
                        }
                        Ok(Incomplete(incomplete)) => {
                            // We haven't finished with this sibling builder, so push it back onto
                            // the front, so we'll pop it off again next instruction
                            remaining.siblings.push_front(incomplete);
                        }
                        Ok(Complete(complete)) => {
                            // We finished building the sibling, so push it into list of siblings
                            self.siblings.push_mut(Insert::Keep(complete));
                        }
                    }
                }

                // No matter what, we're never complete after finishing a sibling, because we still
                // have to construct the focus
                Ok(Incomplete(self))
            } else {
                // If there are no remaining siblings to construct, then construct the focus by
                // forwarding the instruction to it
                match remaining
                    .focus
                    .take()
                    .expect("focus builder is present")
                    .go(instruction)
                {
                    Err(HitBottom(incomplete)) => {
                        // We bounced off the bottom, so restore the focus builder and error
                        remaining.focus = Some(incomplete);
                        Err(HitBottom(self))
                    }
                    Ok(Incomplete(incomplete)) => {
                        // We haven't finished building the focus, so restore it so we'll pop it off
                        // and keep building it next time
                        remaining.focus = Some(incomplete);
                        Ok(Incomplete(self))
                    }
                    // The completion of the builder: map everything into a node
                    Ok(Complete(focus)) => Ok(Complete(self::Node {
                        hash: remaining.hash.map(Into::into).unwrap_or_default(),
                        forgotten: [Forgotten::default(); 4],
                        siblings: self.siblings,
                        focus,
                    })),
                }
            }
        // We have not started construction of the children (this is the first instruction)...
        } else {
            self.remaining = Some(match instruction {
                Leaf { .. } => {
                    unreachable!("leaf instruction is never given as first instruction to a node")
                }
                Node { here, size } => {
                    let hash = here.map(Hash::new);
                    let size: usize = size.into();

                    // Pre-allocate builders for each of the siblings and the focus, giving each the
                    // correct index, so that when we continue with construction, we just need to
                    // pop off the next builder to work with it:
                    let mut index = self.index * 4; // multiply by 4 because we're going one level down

                    let mut siblings = VecDeque::with_capacity(size - 1);
                    while index < self.index + size as u64 {
                        siblings.push_back(Child::Complete::build(self.global_position, index));
                        index += 1;
                    }

                    let focus = Some(Child::build(self.global_position, index));

                    Remaining {
                        hash,
                        siblings,
                        focus,
                    }
                }
            });

            // We're never complete at this point, because at very least we need to build the focus
            Ok(Incomplete(self))
        }
    }

    fn is_started(&self) -> bool {
        self.remaining.is_some()
    }

    fn index(&self) -> u64 {
        if let Some(ref remaining) = self.remaining {
            if let Some(first_sibling) = remaining.siblings.front() {
                first_sibling.index()
            } else {
                remaining
                    .focus
                    .as_ref()
                    .expect("focus builder is present")
                    .index()
            }
        } else {
            self.index
        }
    }

    fn height(&self) -> u8 {
        if let Some(ref remaining) = self.remaining {
            if let Some(first_sibling) = remaining.siblings.front() {
                first_sibling.height()
            } else {
                remaining
                    .focus
                    .as_ref()
                    .expect("focus builder is present")
                    .height()
            }
        } else {
            <Self::Output as Height>::Height::HEIGHT
        }
    }

    fn min_required(&self) -> usize {
        if let Some(ref remaining) = self.remaining {
            // In the case when we have already started construction, we can just sum up the
            // remaining required things for each in-progress builder:
            remaining
                .siblings
                .iter()
                .map(|sibling| sibling.min_required())
                .sum::<usize>()
                + remaining
                    .focus
                    .as_ref()
                    .expect("focus builder is present")
                    .min_required()
        } else {
            // Because we are a frontier node and we haven't yet started construction, the minimum
            // number of instructions we need is the amount necessary to construct the frontier,
            // assuming that nothing is witnessed: we calculate that here by iterating through the
            // global position, adding a number between 1 and 4 to our count depending on the value
            // at that height; we then add 1, because the tip of the frontier must be represented,
            // at least by a hash.
            let mut position = self.global_position;
            let mut min_required = 1; // start at 1 to include the tip
            for _ in 0..self.height() {
                min_required += (position & 0b11) + 1; // 00 => 1, 01 => 2, 10 => 3, 11 => 4
                position >>= 2;
            }
            min_required as usize
        }
    }
}
