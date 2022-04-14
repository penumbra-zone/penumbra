//! Errors that can occur when inserting into an [`Epoch`].

use thiserror::Error;

use crate::internal::hash;

use super::Block;
#[cfg(doc)]
use super::{Commitment, Epoch};

/// A [`Commitment`] could not be inserted into the [`Epoch`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum InsertError {
    /// The [`Epoch`] was full.
    #[error("epoch is full")]
    #[non_exhaustive]
    Full,
    /// The most recent [`Block`] in the [`Epoch`] was full.
    #[error("most recent block in epoch is full")]
    #[non_exhaustive]
    BlockFull,
    /// The most recent [`Block`] in the [`Epoch`] was forgotten.
    #[error("most recent block in epoch was forgotten")]
    #[non_exhaustive]
    BlockForgotten,
}

/// The [`Epoch`] was full when attempting to insert a block.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("epoch is full")]
#[non_exhaustive]
pub struct InsertBlockError<Hasher: hash::Hasher>(pub Block<Hasher>);

impl<Hasher: hash::Hasher> From<InsertBlockError<Hasher>> for Block<Hasher> {
    fn from(error: InsertBlockError<Hasher>) -> Self {
        error.0
    }
}

/// The [`Epoch`] was full when attempting to insert a block root.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("epoch is full")]
#[non_exhaustive]
pub struct InsertBlockRootError;
