#![allow(clippy::unusual_byte_groupings)]

//! Interface for constructing trees from depth-first traversals of them.

use decaf377::Fq;
use std::fmt::Debug;

mod error;
pub mod packed;
pub use error::{Error, HitBottom, IResult, Incomplete, IncompleteInfo};

/// In a depth-first traversal, is the next node below, or to the right? If this is the last
/// represented sibling, then we should go up instead of (illegally) right.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Instruction {
    /// Go down, and remember that the size of the thing below is as specified (`None` for a
    /// terminal node).
    Down {
        here: Option<Fq>,
        size: Option<Size>,
    },
    /// Go right, or up if we can't go right.
    RightOrUp { here: Fq },
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
    /// The global position and index are used to report errors and to calculate the location of the
    /// frontier.
    fn build(global_position: u64, index: u64) -> Self;

    /// Continue with the traversal, going either [`Down`](Direction::Down) or
    /// [`RightOrUp`](Direction::RightOrUp), and setting the value at this node to the given [`Fq`].
    ///
    /// Depending on location, the [`Fq`] may be interpreted either as a [`Hash`] or as a
    /// [`Commitment`].
    fn go(self, instruction: Instruction) -> IResult<Self>;
}
