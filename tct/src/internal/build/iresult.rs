pub use super::*;

/// An incremental result of a construction of a tree from a depth-first preorder traversal.
#[derive(Debug)]
pub enum IResult<C: Build> {
    /// The tree is complete.
    Complete(C::Output),
    /// The tree is incomplete, and the last instruction may or may not have had an error.
    Incomplete(C),
}

impl<C: Build> IResult<C> {
    /// Finalize the [`IResult`] if it is complete, or return an error if it is not yet complete.
    pub fn finish(self) -> Result<C::Output, Incomplete> {
        match self {
            IResult::Complete(output) => Ok(output),
            IResult::Incomplete(_) => Err(Incomplete),
        }
    }
}

/// An error occurred when constructing a tree from a depth-first preorder traversal.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Error)]
#[error("traversal incomplete, awaiting more instructions")]
pub struct Incomplete;

/// The traversal said to continue down, but the thing under construction is the bottom of
/// the tree.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Error)]
#[error("attempted to create child at the bottom of the tree")]
pub struct HitBottom<C>(pub C);
