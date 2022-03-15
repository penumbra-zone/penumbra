//! Errors that can occur when inserting into an [`Eternity`].

use thiserror::Error;

#[cfg(doc)]
use super::Eternity;
use super::{Block, Epoch};

/// An error occurred when trying to insert an item into an [`Eternity`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum InsertError {
    /// The [`Eternity`] was full.
    #[error("eternity is full")]
    #[non_exhaustive]
    Full,
    /// The most recent [`Epoch`] of the [`Eternity`] was full.
    #[error("most recent epoch in eternity is full")]
    EpochFull,
    /// The most recent [`Epoch`] of the [`Eternity`] was forgotten.
    #[error("most recent epoch in eternity was forgotten")]
    EpochForgotten,
    /// The most recent [`Block`] of the most recent [`Epoch`] of the [`Eternity`] was full.
    #[error("most recent block in most recent epoch of eternity is full")]
    BlockFull,
    /// The most recent [`Block`] of the most recent [`Epoch`] of the [`Eternity`] was forgotten.
    #[error("most recent block in most recent epoch of eternity was forgotten")]
    BlockForgotten,
}

/// An error occurred when trying to insert a [`Block`] root into the [`Eternity`].
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum InsertBlockError {
    /// The [`Eternity`] was full.
    #[error("eternity is full")]
    #[non_exhaustive]
    Full(Block),
    /// The most recent [`Epoch`] of the [`Eternity`] was full.
    #[error("most recent epoch is full")]
    #[non_exhaustive]
    EpochFull(Block),
    /// The most recent [`Epoch`] of the [`Eternity`] was forgotten.
    #[error("most recent epoch was forgotten")]
    #[non_exhaustive]
    EpochForgotten(Block),
}

impl From<InsertBlockError> for Block {
    fn from(error: InsertBlockError) -> Self {
        match error {
            InsertBlockError::Full(block) => block,
            InsertBlockError::EpochFull(block) => block,
            InsertBlockError::EpochForgotten(block) => block,
        }
    }
}

/// An error occurred when trying to insert a [`Block`] root into the [`Eternity`].
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum InsertBlockRootError {
    /// The [`Eternity`] was full.
    #[error("eternity is full")]
    #[non_exhaustive]
    Full,
    /// The most recent [`Epoch`] of the [`Eternity`] was full.
    #[error("most recent epoch is full")]
    #[non_exhaustive]
    EpochFull,
    /// The most recent [`Epoch`] of the [`Eternity`] was forgotten.
    #[error("most recent epoch was forgotten")]
    #[non_exhaustive]
    EpochForgotten,
}

/// The [`Eternity`] was full when trying to insert an [`Epoch`].
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("eternity is full")]
#[non_exhaustive]
pub struct InsertEpochError(pub Epoch);

impl From<InsertEpochError> for Epoch {
    fn from(error: InsertEpochError) -> Self {
        error.0
    }
}

/// The [`Eternity`] was full when trying to insert an [`Epoch`] root.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("eternity is full")]
#[non_exhaustive]
pub struct InsertEpochRootError;
