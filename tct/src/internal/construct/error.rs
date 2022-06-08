pub use super::*;

/// An incremental result of a construction of a tree from a depth-first preorder traversal.
pub struct IResult<C: Construct>(IResultInner<C>);

impl<C: Construct> Debug for IResult<C>
where
    C::Output: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            IResultInner::Complete { output } => {
                f.debug_struct("Complete").field("output", &output).finish()
            }
            IResultInner::Incomplete {
                hit_bottom: Ok(()), ..
            } => f.debug_struct("Incomplete").finish_non_exhaustive(),
            IResultInner::Incomplete {
                hit_bottom: Err(_), ..
            } => f.debug_struct("HitBottom").finish_non_exhaustive(),
        }
    }
}

/// The inside of the incremental result.
enum IResultInner<C: Construct> {
    /// The tree is complete.
    Complete {
        /// The output.
        output: C::Output,
    },
    /// The tree is incomplete, and the last instruction may or may not have had an error.
    Incomplete {
        /// The constructor to continue with.
        continuation: C,
        /// Did the last instruction cause the traversal to bottom out erroneously?
        hit_bottom: Result<(), HitBottom>,
    },
}

impl<C: Construct> IResult<C> {
    /// Return a complete result.
    pub fn complete(output: C::Output) -> Self {
        Self(IResultInner::Complete { output })
    }

    /// Indicate that the construction is not yet complete, but the last instruction was accepted.
    pub fn incomplete(continuation: C) -> Self {
        Self(IResultInner::Incomplete {
            continuation,
            hit_bottom: Ok(()),
        })
    }

    /// Indicate that the construction is not yet complete, and the last instruction was rejected
    /// because it would have tried to create a child at the bottom of the tree.
    pub fn hit_bottom(continuation: C) -> Self {
        Self(IResultInner::Incomplete {
            continuation,
            hit_bottom: Err(HitBottom),
        })
    }

    /// Apply a function returning an [`IResult`] to the inner constructor of this [`IResult`].
    ///
    /// If the traversal was already complete, or returned an error, this short-circuits and does
    /// nothing (to override this and proceed in the case of an error, see [`IResult::proceed`]).
    pub fn and_then<F: FnOnce(C) -> IResult<C>>(self, f: F) -> Self {
        Self(match self.0 {
            IResultInner::Complete { output } => IResultInner::Complete { output },
            IResultInner::Incomplete {
                continuation,
                hit_bottom,
            } => {
                if let Err(hit_bottom) = hit_bottom {
                    IResultInner::Incomplete {
                        continuation,
                        hit_bottom: Err(hit_bottom),
                    }
                } else {
                    f(continuation).0
                }
            }
        })
    }

    /// Proceed with the construction of the tree, assuming that the [`IResult`] is not yet complete
    /// or carrying an error from the last action performed.
    ///
    /// If either of the above error cases is true, returns an error. Note that the return type is a
    /// nested [`Result`] to allow you to explicitly determine whether or not to specially handle
    /// the [`HitBottom`] case.
    pub fn proceed(self) -> Result<C, Result<C::Output, HitBottom>> {
        match self.0 {
            IResultInner::Complete { output } => Err(Ok(output)),
            IResultInner::Incomplete {
                continuation,
                hit_bottom,
            } => {
                if let Err(hit_bottom) = hit_bottom {
                    return Err(Err(hit_bottom));
                }
                Ok(continuation)
            }
        }
    }

    /// Finalize the [`IResult`] if it is complete, or return an error if it is not yet complete.
    pub fn finish(self) -> Result<C::Output, Error> {
        match self.0 {
            IResultInner::Complete { output } => Ok(output),
            IResultInner::Incomplete {
                continuation: _,
                hit_bottom,
            } => {
                hit_bottom?;
                Err(Error::Incomplete)
            }
        }
    }
}

/// An error occurred when constructing a tree from a depth-first preorder traversal.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Error)]
pub enum Error {
    #[error(transparent)]
    /// The traversal instructions attempted to create a child of a node at the bottom of the tree.
    HitBottom(HitBottom),
    /// The traversal instructions were incomplete, and the tree was not fully created.
    #[error("traversal incomplete, awaiting more instructions")]
    Incomplete,
}

impl From<HitBottom> for Error {
    fn from(hit_bottom: HitBottom) -> Self {
        Error::HitBottom(hit_bottom)
    }
}

/// The traversal said to continue down, but the thing under construction is the bottom of
/// the tree.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Error)]
#[error("attempted to create child at the bottom of the tree")]
pub struct HitBottom;
