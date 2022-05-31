//! Enumeration of the possible shapes of the children of a [`Node`](super::Node).
//!
//! Each complete node must have at least one child, but some other children may be missing because
//! they have been pruned from the sparse tree.
//!
//! The reason for this enumeration is to save heap space in the case of many nodes: because
//! different nodes can have different sizes, we save on average a few words of memory by placing
//! the box inside each enum variant rather than outside the whole enum (which would end up
//! occupying the space of its largest variant).

#![allow(non_camel_case_types, clippy::upper_case_acronyms)]

use std::fmt::Debug;

mod shape;
pub use shape::*;

use crate::prelude::*;

/// The children of a [`Node`](super::Node).
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
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

impl<Child: Debug> Debug for Children<Child> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.children().fmt(f)
    }
}

impl<Child: Height> Height for Children<Child> {
    type Height = Succ<<Child as Height>::Height>;
}

impl<Child: Height + GetHash> GetHash for Children<Child> {
    fn hash(&self) -> Hash {
        let [a, b, c, d] = self.children().map(|x| x.hash());
        Hash::node(<Self as Height>::Height::HEIGHT, a, b, c, d)
    }

    fn cached_hash(&self) -> Option<Hash> {
        None
    }
}

impl<Child> TryFrom<[Insert<Child>; 4]> for Children<Child>
where
    Child: Height,
{
    type Error = [Hash; 4];

    fn try_from(children: [Insert<Child>; 4]) -> Result<Self, Self::Error> {
        use shape::*;
        use Insert::*;

        Ok(match children {
            // All the children are hashes, so we should prune this node (we just hand back the
            // hashes so the parent can implement pruning):
            [Hash(a), Hash(b), Hash(c), Hash(d)] => return Err([a, b, c, d]),
            // There is at least one witnessed child:
            [Hash(a), Hash(b), Hash(c), Keep(d)] => Children::___C(Box::new(___C(a, b, c, d))),
            [Hash(a), Hash(b), Keep(c), Hash(d)] => Children::__C_(Box::new(__C_(a, b, c, d))),
            [Hash(a), Hash(b), Keep(c), Keep(d)] => Children::__CC(Box::new(__CC(a, b, c, d))),
            [Hash(a), Keep(b), Hash(c), Hash(d)] => Children::_C__(Box::new(_C__(a, b, c, d))),
            [Hash(a), Keep(b), Hash(c), Keep(d)] => Children::_C_C(Box::new(_C_C(a, b, c, d))),
            [Hash(a), Keep(b), Keep(c), Hash(d)] => Children::_CC_(Box::new(_CC_(a, b, c, d))),
            [Hash(a), Keep(b), Keep(c), Keep(d)] => Children::_CCC(Box::new(_CCC(a, b, c, d))),
            [Keep(a), Hash(b), Hash(c), Hash(d)] => Children::C___(Box::new(C___(a, b, c, d))),
            [Keep(a), Hash(b), Hash(c), Keep(d)] => Children::C__C(Box::new(C__C(a, b, c, d))),
            [Keep(a), Hash(b), Keep(c), Hash(d)] => Children::C_C_(Box::new(C_C_(a, b, c, d))),
            [Keep(a), Hash(b), Keep(c), Keep(d)] => Children::C_CC(Box::new(C_CC(a, b, c, d))),
            [Keep(a), Keep(b), Hash(c), Hash(d)] => Children::CC__(Box::new(CC__(a, b, c, d))),
            [Keep(a), Keep(b), Hash(c), Keep(d)] => Children::CC_C(Box::new(CC_C(a, b, c, d))),
            [Keep(a), Keep(b), Keep(c), Hash(d)] => Children::CCC_(Box::new(CCC_(a, b, c, d))),
            [Keep(a), Keep(b), Keep(c), Keep(d)] => Children::CCCC(Box::new(CCCC(a, b, c, d))),
        })
    }
}

impl<Child> Children<Child> {
    /// Get an array of references to the children or hashes stored in this [`Children`].
    pub fn children(&self) -> [Insert<&Child>; 4] {
        use Children::*;
        use Insert::*;

        match self {
            ___C(c) => [Hash(c.0), Hash(c.1), Hash(c.2), Keep(&c.3)],
            __C_(c) => [Hash(c.0), Hash(c.1), Keep(&c.2), Hash(c.3)],
            __CC(c) => [Hash(c.0), Hash(c.1), Keep(&c.2), Keep(&c.3)],
            _C__(c) => [Hash(c.0), Keep(&c.1), Hash(c.2), Hash(c.3)],
            _C_C(c) => [Hash(c.0), Keep(&c.1), Hash(c.2), Keep(&c.3)],
            _CC_(c) => [Hash(c.0), Keep(&c.1), Keep(&c.2), Hash(c.3)],
            _CCC(c) => [Hash(c.0), Keep(&c.1), Keep(&c.2), Keep(&c.3)],
            C___(c) => [Keep(&c.0), Hash(c.1), Hash(c.2), Hash(c.3)],
            C__C(c) => [Keep(&c.0), Hash(c.1), Hash(c.2), Keep(&c.3)],
            C_C_(c) => [Keep(&c.0), Hash(c.1), Keep(&c.2), Hash(c.3)],
            C_CC(c) => [Keep(&c.0), Hash(c.1), Keep(&c.2), Keep(&c.3)],
            CC__(c) => [Keep(&c.0), Keep(&c.1), Hash(c.2), Hash(c.3)],
            CC_C(c) => [Keep(&c.0), Keep(&c.1), Hash(c.2), Keep(&c.3)],
            CCC_(c) => [Keep(&c.0), Keep(&c.1), Keep(&c.2), Hash(c.3)],
            CCCC(c) => [Keep(&c.0), Keep(&c.1), Keep(&c.2), Keep(&c.3)],
        }
    }
}

impl<Child> From<Children<Child>> for [Insert<Child>; 4] {
    /// Get an array of references to the children or hashes stored in this [`Children`].
    fn from(children: Children<Child>) -> [Insert<Child>; 4] {
        use Children::*;
        use Insert::*;

        match children {
            ___C(c) => [Hash(c.0), Hash(c.1), Hash(c.2), Keep(c.3)],
            __C_(c) => [Hash(c.0), Hash(c.1), Keep(c.2), Hash(c.3)],
            __CC(c) => [Hash(c.0), Hash(c.1), Keep(c.2), Keep(c.3)],
            _C__(c) => [Hash(c.0), Keep(c.1), Hash(c.2), Hash(c.3)],
            _C_C(c) => [Hash(c.0), Keep(c.1), Hash(c.2), Keep(c.3)],
            _CC_(c) => [Hash(c.0), Keep(c.1), Keep(c.2), Hash(c.3)],
            _CCC(c) => [Hash(c.0), Keep(c.1), Keep(c.2), Keep(c.3)],
            C___(c) => [Keep(c.0), Hash(c.1), Hash(c.2), Hash(c.3)],
            C__C(c) => [Keep(c.0), Hash(c.1), Hash(c.2), Keep(c.3)],
            C_C_(c) => [Keep(c.0), Hash(c.1), Keep(c.2), Hash(c.3)],
            C_CC(c) => [Keep(c.0), Hash(c.1), Keep(c.2), Keep(c.3)],
            CC__(c) => [Keep(c.0), Keep(c.1), Hash(c.2), Hash(c.3)],
            CC_C(c) => [Keep(c.0), Keep(c.1), Hash(c.2), Keep(c.3)],
            CCC_(c) => [Keep(c.0), Keep(c.1), Keep(c.2), Hash(c.3)],
            CCCC(c) => [Keep(c.0), Keep(c.1), Keep(c.2), Keep(c.3)],
        }
    }
}
