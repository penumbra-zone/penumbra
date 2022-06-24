#![allow(clippy::unusual_byte_groupings)]

//! Interface for constructing trees from depth-first traversals of them.

use decaf377::Fq;
use std::fmt::Debug;

use crate::prelude::*;

mod iresult;
// pub mod packed; // TODO: fix this module
pub use iresult::{IResult, Unexpected};

use super::frontier::TrackForgotten;

/// In a depth-first traversal, is the next node below, or to the right? If this is the last
/// represented sibling, then we should go up instead of (illegally) right.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Instruction {
    /// This node is an internal node, with a non-zero number of children and an optional cached
    /// value. We should create it, then continue the traversal to create its children.
    Node {
        /// The element at this leaf, if any (internal nodes can always calculate their element, so
        /// this is optional).
        here: Option<Fq>,
        /// The number of children of this node.
        size: Size,
    },
    /// This node is a leaf, with no children and a mandatory value. We should create it, then
    /// return it as completed, to continue the traversal at the parent.
    Leaf {
        /// The element at this leaf.
        here: Fq,
    },
}

/// Proptest generators for things relevant to construction.
#[cfg(feature = "arbitrary")]
pub mod arbitrary {
    use super::*;

    /// Generate an arbitrary instruction.
    pub fn instruction() -> impl proptest::prelude::Strategy<Value = Instruction> {
        use proptest::prelude::*;

        proptest::option::of(crate::commitment::FqStrategy::arbitrary()).prop_flat_map(
            |option_fq| {
                Size::arbitrary().prop_flat_map(move |children| {
                    bool::arbitrary().prop_map(move |variant| {
                        if let Some(here) = option_fq {
                            if variant {
                                Instruction::Node {
                                    here: Some(here),
                                    size: children,
                                }
                            } else {
                                Instruction::Leaf { here }
                            }
                        } else {
                            Instruction::Node {
                                here: None,
                                size: children,
                            }
                        }
                    })
                })
            },
        )
    }
}

/// The number of children of a node we're creating.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "arbitrary", derive(proptest_derive::Arbitrary))]
pub enum Size {
    /// 1 child.
    One,
    /// 2 children.
    Two,
    /// 3 children.
    Three,
    /// 4 children.
    Four,
}

impl From<Size> for usize {
    fn from(size: Size) -> Self {
        match size {
            Size::One => 1,
            Size::Two => 2,
            Size::Three => 3,
            Size::Four => 4,
        }
    }
}

impl Debug for Size {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Size::One => write!(f, "1"),
            Size::Two => write!(f, "2"),
            Size::Three => write!(f, "3"),
            Size::Four => write!(f, "4"),
        }
    }
}

/// A builder that can incrementally consume a pre-order depth-first traversal of node values to
/// build a tree.
pub trait Build: Sized {
    /// The output of this constructor.
    type Output: Built<Builder = Self>;

    /// Continue with the traversal using the given [`Instruction`].
    ///
    /// Depending on location, the [`Fq`] contained in the instruction may be interpreted either as
    /// a [`Hash`] or as a [`Commitment`].
    fn go(self, instruction: Instruction) -> Result<IResult<Self>, Unexpected>;

    /// Checks if the builder has been started, i.e. it has received > 0 instructions.
    fn is_started(&self) -> bool;

    /// Get the current index under construction in the traversal.
    fn index(&self) -> u64;

    /// Get the current height under construction in the traversal.
    fn height(&self) -> u8;

    /// Get the minimum number of instructions necessary to complete construction.
    fn min_required(&self) -> usize;
}

/// Trait uniquely identifying the builder for any given type, if it is constructable.
pub trait Built {
    /// The builder for this type.
    type Builder: Build<Output = Self>;

    // TODO: remove the global position; always construct frontier tiers; nudge the frontier after
    // the fact by checking the position!

    /// Create a new constructor for a node at the given index, given the global position of the
    /// tree.
    ///
    /// The global position and index are used to calculate the location of the frontier.
    fn build(global_position: u64, index: u64) -> Self::Builder;
}

/// An error when constructing something, indicative of an incorrect sequence of instructions.
pub enum Error {
    /// An unexpected instruction was provided.
    Unexpected {
        /// The instruction at which the error occurred.
        instruction: usize,
        /// The index of the node in question.
        index: u64,
        /// The height of the node in question.
        height: u8,
        /// The unexpected instruction.
        unexpected: Unexpected,
    },
    /// Not enough instructions were supplied to construct the object.
    Incomplete {
        /// The number of instructions supplied.
        instruction: usize,
        /// The height of the node that was currently incomplete when instructions ran out.
        height: u8,
        /// The index of the node that was currently incomplete when instructions ran out.
        index: u64,
        /// The minimum required number of instructions that would have been needed to complete construction.
        min_required: usize,
    },
    /// Too many instructions were supplied.
    AlreadyComplete {
        /// The number of instructions that were used to successfully construct the object.
        instruction: usize,
    },
}

type Tree = frontier::Top<frontier::Tier<frontier::Tier<frontier::Item>>>;

/// Build a tree by iterating over a sequence of [`Instruction`]s.
pub fn build(
    position: u64,
    instructions: impl IntoIterator<Item = impl Into<Instruction>>,
) -> Result<Tree, Error> {
    let mut instructions = instructions.into_iter().peekable();
    if instructions.peek().is_none() {
        return Ok(crate::internal::frontier::Top::new(TrackForgotten::Yes));
    }

    // Count the instructions as we go along, for error reporting
    let mut instruction: usize = 0;

    // The incremental result, either an incomplete builder or a complete output
    let mut result = IResult::Incomplete(<Tree as Built>::build(position, 0));

    // For each instruction, tell the builder to use that instruction
    for this_instruction in &mut instructions {
        let builder = match result {
            IResult::Complete(_) => break, // stop if complete, even if instructions aren't
            IResult::Incomplete(builder) => builder,
        };

        // Step forward the builder by one instruction
        let index = builder.index();
        let height = builder.height();
        result = builder
            .go(this_instruction.into())
            .map_err(|unexpected| Error::Unexpected {
                instruction,
                unexpected,
                index,
                height,
            })?;

        // Update the instruction count
        instruction += 1;
    }

    // Examine whether we successfully constructed the tree
    match result {
        // If complete, return the output tree
        IResult::Complete(output) => {
            // Ensure that no more instructions are remaining
            if instructions.peek().is_some() {
                return Err(Error::AlreadyComplete { instruction });
            }
            Ok(output)
        }
        // If incomplete, return an error indicating the situation we stopped in
        IResult::Incomplete(builder) => Err(Error::Incomplete {
            instruction,
            height: builder.height(),
            index: builder.index(),
            min_required: builder.min_required(),
        }),
    }
}
