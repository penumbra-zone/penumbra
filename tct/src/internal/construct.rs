#![allow(clippy::unusual_byte_groupings)]

//! Interface for constructing trees from depth-first traversals of them.

use decaf377::Fq;
use std::fmt::Debug;

mod error;
// pub mod packed; // TODO: fix this module
pub use error::{Error, HitBottom, IResult};

/// In a depth-first traversal, is the next node below, or to the right? If this is the last
/// represented sibling, then we should go up instead of (illegally) right.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Instruction {
    /// This node is an internal node, with a non-zero number of children and an optional cached
    /// value. We should create it, then continue the traversal to create its children.
    Node { here: Option<Fq>, children: Size },
    /// This node is a leaf, with no children and a mandatory value. We should create it, then
    /// return it as completed, to continue the traversal at the parent.
    Leaf { here: Fq },
}

#[cfg(feature = "arbitrary")]
fn arbitrary_instruction() -> impl proptest::prelude::Strategy<Value = Instruction> {
    use proptest::prelude::*;

    proptest::option::of(crate::commitment::FqStrategy::arbitrary()).prop_flat_map(|option_fq| {
        Size::arbitrary().prop_flat_map(move |children| {
            bool::arbitrary().prop_map(move |variant| {
                if let Some(here) = option_fq {
                    if variant {
                        Instruction::Node {
                            here: Some(here),
                            children,
                        }
                    } else {
                        Instruction::Leaf { here }
                    }
                } else {
                    Instruction::Node {
                        here: None,
                        children,
                    }
                }
            })
        })
    })
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
pub trait Construct: Sized {
    /// The output of this constructor.
    type Output;

    /// Create a new constructor for a node at the given index, given the global position of the
    /// tree.
    ///
    /// The global position and index are used to calculate the location of the frontier.
    fn build(global_position: u64, index: u64) -> Self;

    /// Continue with the traversal using the given [`Instruction`].
    ///
    /// Depending on location, the [`Fq`] contained in the instruction may be interpreted either as
    /// a [`Hash`] or as a [`Commitment`].
    fn go(self, instruction: Instruction) -> IResult<Self>;

    /// Get the current index under construction in the traversal.
    fn index(&self) -> u64;

    /// Get the current height under construction in the traversal.
    fn height(&self) -> u8;

    /// Get the minimum number of instructions necessary to complete construction.
    fn min_required(&self) -> usize;
}
