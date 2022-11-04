//! Errors that can occur when inserting into a [`Tree`], deserializing [`Proof`](super::Proof)s, or
//! checking internal invariants.

use archery::SharedPointerKind;

use crate::builder;
#[cfg(doc)]
use crate::prelude::*;

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

pub mod block {
    //! Errors for [`block`](self) builders.

    /// An error occurred when decoding a [`block::Root`](crate::builder::block::Root) from bytes.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
    #[error("could not decode block root")]
    pub struct RootDecodeError;

    /// When inserting into a [`block::Builder`](crate::builder::block::Builder), this error is
    /// returned when it is full.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
    #[error("block is full")]
    #[non_exhaustive]
    pub struct InsertError;
}

pub mod epoch {
    //! Errors for [`epoch`] builders.
    use archery::SharedPointerKind;

    use super::*;

    /// An error occurred when decoding an [`epoch::Root`](builder::epoch::Root) from bytes.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
    #[error("could not decode epoch root")]
    pub struct RootDecodeError;

    /// A [`Commitment`] could not be inserted into the [`epoch::Builder`](builder::epoch::Builder).
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
    pub enum InsertError {
        /// The [`epoch::Builder`](builder::epoch::Builder) was full.
        #[error("epoch is full")]
        #[non_exhaustive]
        Full,
        /// The most recent block in the [`epoch::Builder`](builder::epoch::Builder) was full.
        #[error("most recent block in epoch is full")]
        #[non_exhaustive]
        BlockFull,
    }

    /// The [`epoch::Builder`](builder::epoch::Builder) was full when attempting to insert a block.
    #[derive(Debug, Clone, Error)]
    #[error("epoch is full")]
    #[non_exhaustive]
    pub struct InsertBlockError<RefKind: SharedPointerKind>(pub builder::block::Finalized<RefKind>);
}

/// An error occurred when trying to insert a [`Commitment`] into a [`Tree`].
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
pub enum InsertBlockError<RefKind: SharedPointerKind> {
    /// The [`Tree`] was full.
    #[error("tree is full")]
    #[non_exhaustive]
    Full(builder::block::Finalized<RefKind>),
    /// The most recent epoch of the [`Tree`] was full.
    #[error("most recent epoch is full")]
    #[non_exhaustive]
    EpochFull(builder::block::Finalized<RefKind>),
}

impl<RefKind: SharedPointerKind> From<InsertBlockError<RefKind>>
    for builder::block::Finalized<RefKind>
{
    fn from(error: InsertBlockError<RefKind>) -> Self {
        match error {
            InsertBlockError::Full(block) => block,
            InsertBlockError::EpochFull(block) => block,
        }
    }
}

/// The [`Tree`] was full when trying to insert an epoch into it.
#[derive(Debug, Clone, Error)]
#[error("tree is full")]
#[non_exhaustive]
pub struct InsertEpochError<RefKind: SharedPointerKind>(pub builder::epoch::Finalized<RefKind>);

impl<RefKind: SharedPointerKind> From<InsertEpochError<RefKind>>
    for builder::epoch::Finalized<RefKind>
{
    fn from(error: InsertEpochError<RefKind>) -> Self {
        error.0
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn insert_errors_sync_send() {
        static_assertions::assert_impl_all!(InsertError: Sync, Send);
        static_assertions::assert_impl_all!(InsertBlockError<archery::ArcK>: Sync, Send);
        static_assertions::assert_impl_all!(InsertEpochError<archery::ArcK>: Sync, Send);
    }
}
