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
    /// Change the type that is constructed by an [`IResult`], by providing a way to map over the
    /// complete and incomplete types.
    pub fn map<D: Build>(
        self,
        on_incomplete: impl FnOnce(C) -> D,
        on_complete: impl FnOnce(C::Output) -> D::Output,
    ) -> IResult<D> {
        match self {
            IResult::Complete(output) => IResult::Complete(on_complete(output)),
            IResult::Incomplete(builder) => IResult::Incomplete(on_incomplete(builder)),
        }
    }
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
