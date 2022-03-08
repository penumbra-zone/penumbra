//! Enumeration of the possible shapes of the children of a node.
//!
//! Each complete node must have at least one child, but some other children may be missing because
//! they have been pruned from the sparse tree.

#![allow(non_camel_case_types)]

pub mod shape;
use shape::*;

/// The children of a [`Node`](super::Node).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Children<Child> {
    /// Children of a node having children in the positions: 3.
    ___C(Box<___C<Child>>),
    /// Children of a node having children in the positions: 2.
    __C_(Box<__C_<Child>>),
    /// Children of a node having children in the positions: 2, 3.
    __CC(Box<__CC<Child>>),
    /// Children of a node having children in the positions: 1.
    _C__(Box<_C__<Child>>),
    /// Children of a node having children in the positions: 1, 3.
    _C_C(Box<_C_C<Child>>),
    /// Children of a node having children in the positions: 1, 2.
    _CC_(Box<_CC_<Child>>),
    /// Children of a node having children in the positions: 1, 2, 3.
    _CCC(Box<_CCC<Child>>),
    /// Children of a node having children in the positions: 0.
    C___(Box<C___<Child>>),
    /// Children of a node having children in the positions: 0, 3.
    C__C(Box<C__C<Child>>),
    /// Children of a node having children in the positions: 0, 2.
    C_C_(Box<C_C_<Child>>),
    /// Children of a node having children in the positions: 0, 2, 3.
    C_CC(Box<C_CC<Child>>),
    /// Children of a node having children in the positions: 0, 1.
    CC__(Box<CC__<Child>>),
    /// Children of a node having children in the positions: 0, 1, 3.
    CC_C(Box<CC_C<Child>>),
    /// Children of a node having children in the positions: 0, 1, 2.
    CCC_(Box<CCC_<Child>>),
    /// Children of a node having children in the positions: 0, 1, 2, 3.
    CCCC(Box<CCCC<Child>>),
}
