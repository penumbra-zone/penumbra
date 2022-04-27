//! Errors that can occur when inserting into a [`Tree`].

use thiserror::Error;

#[cfg(doc)]
use super::Tree;
use super::{block, epoch};

/// An error occurred when trying to insert an commitment into a [`Tree`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum InsertError {
    /// The [`Tree`] was full.
    #[error("tree is full")]
    Full,
    /// The most recent [`Epoch`] of the [`Tree`] was full.
    #[error("most recent epoch in tree is full")]
    EpochFull,
    /// The most recent [`Block`] of the most recent [`Epoch`] of the [`Tree`] was full.
    #[error("most recent block in most recent epoch of tree is full")]
    BlockFull,
}

/// An error occurred when trying to insert a [`Block`] root into the [`Tree`].
#[derive(Debug, Clone, Error)]
pub enum InsertBlockError {
    /// The [`Tree`] was full.
    #[error("tree is full")]
    #[non_exhaustive]
    Full(block::Finalized),
    /// The most recent [`Epoch`] of the [`Tree`] was full.
    #[error("most recent epoch is full")]
    #[non_exhaustive]
    EpochFull(block::Finalized),
}

impl From<InsertBlockError> for block::Finalized {
    fn from(error: InsertBlockError) -> Self {
        match error {
            InsertBlockError::Full(block) => block,
            InsertBlockError::EpochFull(block) => block,
        }
    }
}

/// The [`Tree`] was full when trying to insert an [`Epoch`].
#[derive(Debug, Clone, Error)]
#[error("tree is full")]
#[non_exhaustive]
pub struct InsertEpochError(pub epoch::Finalized);

impl From<InsertEpochError> for epoch::Finalized {
    fn from(error: InsertEpochError) -> Self {
        error.0
    }
}

/// The [`Tree`] was full when trying to insert an [`Epoch`] root.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("tree is full")]
#[non_exhaustive]
pub struct InsertEpochRootError;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn insert_errors_sync_send() {
        static_assertions::assert_impl_all!(InsertError: Sync, Send);
        static_assertions::assert_impl_all!(InsertBlockError: Sync, Send);
        static_assertions::assert_impl_all!(InsertEpochError: Sync, Send);
    }
}
