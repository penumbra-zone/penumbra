#![allow(clippy::unusual_byte_groupings)]

//! Non-incremental deserialization for the [`Tree`](crate::Tree).

use futures::{stream, Stream, StreamExt};
use std::{fmt::Debug, pin::Pin};

use crate::prelude::*;
use crate::storage::{Instruction, Point, Read};

// pub mod packed; // TODO: fix this module

pub mod read;

use crate::internal::frontier::TrackForgotten;

/// An error occurred when constructing a tree from a depth-first preorder traversal.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Error)]
#[error("traversal incomplete, awaiting more instructions")]
pub struct Incomplete;

/// An instruction for constructing the tree was given which was not valid.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Error)]
pub enum Unexpected {
    /// The instruction said to construct a node, but only a leaf could be constructed.
    #[error("unexpected `Node` instruction; expected `Leaf`")]
    Node,
    /// The instruction said to construct a leaf, but only a node could be constructed.
    #[error("unexpected `Leaf` instruction; expected `Node`")]
    Leaf,
    /// A sequence of instructions tried to construct an internal node which didn't have any children.
    ///
    /// One structural invariant of the tree is that all internal nodes have at least one child,
    /// which preserves the fact that the tree is _maximally pruned_: no dangling internal nodes
    /// exist without witnessed children beneath them.
    #[error("unexpected: at least one child of internal node must be witnessed")]
    Unwitnessed,
}

/// An error when constructing something, indicative of an incorrect sequence of instructions.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Error)]
pub enum Error<E> {
    /// An unexpected instruction was provided.
    #[error(
        "instruction {instruction} building node at index {index}, height {height}: {unexpected}"
    )]
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
    #[error("instruction {instruction} building node at index {index}, height {height}: not enough instructions supplied, needed at least {min_required} more")]
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
    #[error("instruction {instruction}: already completed construction")]
    AlreadyComplete {
        /// The number of instructions that were used to successfully construct the object.
        instruction: usize,
    },
    /// An underlying error in the stream of instructions occurred.
    #[error("{error}")]
    Underlying {
        /// The underlying error.
        #[from]
        error: E,
    },
}

type Tree = frontier::Top<frontier::Tier<frontier::Tier<frontier::Item>>>;
