pub use super::*;

/// The traversal said to continue down, but the thing under construction is the bottom of
/// the tree.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Error)]
#[error("attempted to create child at the bottom of the tree at position {position:?}")]
pub struct HitBottom {
    /// The position at the bottom of the tree where the error occurred.
    position: u64,
}

/// The traversal is not yet completed, but was expected to be complete.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Error)]
#[error("traversal incomplete, paused at index {index:?}, height {height:?}, requiring at least {min_remaining:?} more instructions")]
pub struct Incomplete {
    /// The minimum number of additional instructions needed to complete the construction.
    min_remaining: usize,
    /// The index of the node awaiting further instructions.
    index: u64,
    /// The height of the node awaiting further instructions.
    height: u8,
}

/// Information about the current status of an incomplete construction.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Error)]
#[error("traversal incomplete, paused at index {index:?}, height {height:?}, requiring at least {min_remaining:?} more instructions{}", if *.hit_bottom { ", having failed to create child at the bottom of the tree" } else { "" })]
pub struct IncompleteInfo {
    /// The index of the node awaiting further instructions.
    pub index: u64,
    /// The height of the node awaiting further instructions.
    pub height: u8,
    /// The minimum number of additional instructions needed to complete the construction.
    pub min_remaining: usize,
    /// The size of the [`HitBottom`] error, if any just occurred.
    pub hit_bottom: bool,
}

/// An incremental result of a construction of a tree from a depth-first preorder traversal.
pub enum IResult<C: Construct> {
    /// The tree is complete.
    Complete {
        /// The output.
        output: C::Output,
    },
    /// The tree is incomplete, and the last instruction may or may not have had an error.
    Incomplete {
        /// The constructor to continue with.
        continuation: C,
        /// Information about the incomplete result.
        info: IncompleteInfo,
    },
}

/// An error occurred when constructing a tree from a depth-first preorder traversal.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Error)]
pub enum Error {
    #[error(transparent)]
    /// The traversal instructions attempted to create a child of a node at the bottom of the tree.
    HitBottom(HitBottom),
    /// The traversal instructions were incomplete, and the tree was not fully created.
    #[error(transparent)]
    Incomplete(Incomplete),
}

impl From<HitBottom> for Error {
    fn from(hit_bottom: HitBottom) -> Self {
        Error::HitBottom(hit_bottom)
    }
}

impl From<Incomplete> for Error {
    fn from(incomplete: Incomplete) -> Self {
        Error::Incomplete(incomplete)
    }
}

impl<C: Construct> IResult<C> {
    /// Proceed with the construction of the tree, assuming that the [`IResult`] is not yet
    /// [`Complete`](IResult::Complete) or carrying a [`HitBottom`] error from the last action
    /// performed.
    ///
    /// If either of the above error cases is true, returns an error. Note that the return type is a
    /// nested [`Result`] to allow you to explicitly determine whether or not to specially handle
    /// the [`HitBottom`] case.
    pub fn proceed(self) -> Result<C, Result<C::Output, HitBottom>> {
        match self {
            IResult::Complete { output } => Err(Ok(output)),
            IResult::Incomplete { continuation, info } => {
                if info.hit_bottom {
                    Err(Err(HitBottom {
                        position: info.index,
                    }))
                } else {
                    Ok(continuation)
                }
            }
        }
    }

    /// Finalize the [`IResult`] if it is [`Complete`](IResult::Complete), or return an error if it
    /// is not yet complete.
    pub fn finish(self) -> Result<C::Output, Error> {
        match self {
            IResult::Complete { output } => Ok(output),
            IResult::Incomplete {
                continuation: _,
                info:
                    IncompleteInfo {
                        min_remaining: min_remaining_instructions,
                        index,
                        height,
                        hit_bottom: error,
                    },
            } => {
                if error {
                    return Err(HitBottom { position: index }.into());
                };
                Err(Error::Incomplete(Incomplete {
                    min_remaining: min_remaining_instructions,
                    index,
                    height,
                }))
            }
        }
    }
}
