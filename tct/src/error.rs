//! Errors that can occur when inserting into a [`Tree`], deserializing [`Proof`](super::Proof)s, or
//! checking internal invariants.

use thiserror::Error;

#[cfg(doc)]
use super::Tree;
use crate::builder;

#[doc(inline)]
pub use crate::tree::RootDecodeError;

pub mod proof {
    //! Errors from deserializing or verifying inclusion proofs.
    #[doc(inline)]
    pub use crate::internal::{
        path::PathDecodeError,
        proof::{ProofDecodeError as DecodeError, VerifyError},
    };
}

pub mod validate {
    //! Errors resulting from internal validation.
    #[doc(inline)]
    pub use crate::tree::verify::{
        IndexError, IndexMalformed, InvalidCachedHash, InvalidCachedHashes, InvalidWitnesses,
        WitnessError,
    };
}

pub mod block {
    //! Errors for [`block`] builders.
    use super::*;

    /// An error occurred when decoding a block root from bytes.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
    #[error("could not decode block root")]
    pub struct RootDecodeError;

    /// When inserting into a block, this error is returned when it is full.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
    #[error("block is full")]
    #[non_exhaustive]
    pub struct InsertError;
}

pub mod epoch {
    //! Errors for [`epoch`] builders.
    use super::*;

    /// An error occurred when decoding an epoch root from bytes.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
    #[error("could not decode epoch root")]
    pub struct RootDecodeError;

    /// A [`Commitment`] could not be inserted into the [`epoch::Builder`](Builder).
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
    pub enum InsertError {
        /// The [`epoch::Builder`](Builder) was full.
        #[error("epoch is full")]
        #[non_exhaustive]
        Full,
        /// The most recent block in the [`epoch::Builder`](Builder) was full.
        #[error("most recent block in epoch is full")]
        #[non_exhaustive]
        BlockFull,
    }

    /// The [`epoch::Builder`](Builder) was full when attempting to insert a block.
    #[derive(Debug, Clone, Error)]
    #[error("epoch is full")]
    #[non_exhaustive]
    pub struct InsertBlockError(pub builder::block::Finalized);
}

/// An error occurred when trying to insert an commitment into a [`Tree`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum InsertError {
    /// The [`Tree`] was full.
    #[error("tree is full")]
    Full,
    /// The most recent epoch of the [`Tree`] was full.
    #[error("most recent epoch in tree is full")]
    EpochFull,
    /// The most recent block of the most recent epoch of the [`Tree`] was full.
    #[error("most recent block in most recent epoch of tree is full")]
    BlockFull,
}

/// An error occurred when trying to insert a block into the [`Tree`].
#[derive(Debug, Clone, Error)]
pub enum InsertBlockError {
    /// The [`Tree`] was full.
    #[error("tree is full")]
    #[non_exhaustive]
    Full(builder::block::Finalized),
    /// The most recent epoch of the [`Tree`] was full.
    #[error("most recent epoch is full")]
    #[non_exhaustive]
    EpochFull(builder::block::Finalized),
}

impl From<InsertBlockError> for builder::block::Finalized {
    fn from(error: InsertBlockError) -> Self {
        match error {
            InsertBlockError::Full(block) => block,
            InsertBlockError::EpochFull(block) => block,
        }
    }
}

/// The [`Tree`] was full when trying to insert an epoch.
#[derive(Debug, Clone, Error)]
#[error("tree is full")]
#[non_exhaustive]
pub struct InsertEpochError(pub builder::epoch::Finalized);

impl From<InsertEpochError> for builder::epoch::Finalized {
    fn from(error: InsertEpochError) -> Self {
        error.0
    }
}

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
