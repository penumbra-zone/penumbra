use std::{fmt::Debug, collections::VecDeque};

use super::*;

use crate::storage::{
    deserialize::{IResult, Unexpected},
    Instruction,
};

/// A builder for a frontier node.
#[derive(Derivative)]
#[derivative(Debug(bound = "Child: Built + Debug, Child::Builder: Debug"))]
pub struct Builder<Child: Built> {
    index: u64,
    global_position: u64,
    // This is populated when we first get a `Node` instruction, to the size requested
    remaining: Option<Remaining<Child>>,
    // Then as we complete siblings, we push them onto this
    children: Three<Insert<Child>>,
}

#[derive(Debug)]
struct Remaining<Child: Built> {
    hash: Option<Hash>,
    children: VecDeque<Child::Builder>,
}

impl<Child: GetHash + Height + Built> Built for Node<Child> {
    type Builder = Builder<Child>;

    fn build(global_position: u64, index: u64) -> Self::Builder {
        Builder {
            global_position,
            index,
            children: Three::new(),
            remaining: None,
        }
    }
}

impl<Child: GetHash + Height + Built> Build for Builder<Child> {
    type Output = Node<Child>;

    fn go(mut self, instruction: Instruction) -> Result<IResult<Self>, Unexpected> {
        use {IResult::*, Instruction::*};

        if let Some(ref mut remaining) = self.remaining {
            // If there is already a child under construction...
            if let Some(front_child) = remaining.children.pop_front() {
                // If the instruction is a `Leaf` instruction and we haven't started
                // construction of the front child, we want to skip the construction of
                // the front child and just abbreviate it with a hash -- but if we've
                // already started construction, then the instruction should just be
                // forwarded as usual
                let mut skipped = false;
                if !front_child.is_started() {
                    if let Leaf { here } = instruction {
                        self.children.push_mut(Insert::Hash(Hash::new(here)));
                        skipped = true;
                    }
                };

                // If we didn't skip constructing this sibling, then we should forward the
                // instruction to the front sibling builder
                if !skipped {
                    match front_child.go(instruction)? {
                        Incomplete(incomplete) => {
                            // We haven't finished with this builder, so push it back onto
                            // the front, so we'll pop it off again next instruction
                            remaining.children.push_front(incomplete);
                        }
                        Complete(complete) => {
                            // We finished building the sibling, so push it into list of siblings
                            self.children.push_mut(Insert::Keep(complete));
                        }
                    }
                }

                // If the remaining children builders are empty, then it's time to return a
                // completed `Node`
                if remaining.children.is_empty() {
                    // Pad out the children with `Hash::one()` until we fill them up
                    let children = loop {
                        self.children = match self.children.push(Insert::Hash(Hash::one())) {
                            Ok(children) => children,
                            Err(exactly_four_children) => break exactly_four_children,
                        };
                    };

                    let children = if let Ok(children) = Children::try_from(children) {
                        children
                    } else {
                        // If all the children were hashes, we can't keep constructing, because this
                        // is a structural invariant violation
                        return Err(Unexpected::Unwitnessed);
                    };

                    // If we were given a hash, use that; otherwise, calculate one
                    let hash = remaining.hash.unwrap_or_else(|| children.hash());

                    Ok(Complete(self::Node {
                        hash,
                        children,
                        forgotten: [Forgotten::default(); 4],
                    }))
                } else {
                    Ok(Incomplete(self))
                }
            } else {
                unreachable!("list of under-construction children is never empty");
            }
        } else {
            // If there is not a child under construction, then our first instruction must be a
            // `Node`:
            self.remaining = Some(match instruction {
                Leaf { .. } => {
                    // If we're given a `Leaf` instruction, then we can't construct a node,
                    // because this is a structural invariant violation
                    return Err(Unexpected::Unwitnessed);
                }
                Node { here, size } => {
                    let hash = here.map(Hash::new);
                    let size: usize = size.into();

                    // Pre-allocate builders for each of the siblings and the focus, giving each the
                    // correct index, so that when we continue with construction, we just need to
                    // pop off the next builder to work with it:

                    let mut children = VecDeque::with_capacity(size - 1);
                    for i in 0..size {
                        // Child index is parent index * 4 (because we're going down a level) plus
                        // the index of the child relative to the parent
                        let index = self.index * 4 + i as u64;
                        children.push_back(Child::build(self.global_position, index));
                    }

                    Remaining { hash, children }
                }
            });

            Ok(Incomplete(self))
        }
    }

    fn is_started(&self) -> bool {
        self.remaining.is_some()
    }

    fn index(&self) -> u64 {
        if let Some(ref remaining) = self.remaining {
            if let Some(first) = remaining.children.front() {
                first.index()
            } else {
                unreachable!("list of under-construction children is never empty")
            }
        } else {
            self.index
        }
    }

    fn height(&self) -> u8 {
        if let Some(ref remaining) = self.remaining {
            if let Some(first) = remaining.children.front() {
                first.height()
            } else {
                unreachable!("list of under-construction children is never empty")
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
                .children
                .iter()
                .map(|sibling| sibling.min_required())
                .sum::<usize>()
        } else {
            1
        }
    }
}
