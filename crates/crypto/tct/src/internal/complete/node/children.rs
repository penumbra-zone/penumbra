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

use std::{fmt::Debug, sync::Arc};

mod shape;
pub use shape::*;

use crate::prelude::*;

/// The children of a [`Node`](super::Node).
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Children<Child> {
    /// Children of a node having children in the positions: 3.
    ___C(Arc<___C<Child>>),
    /// Children of a node having children in the positions: 2.
    __C_(Arc<__C_<Child>>),
    /// Children of a node having children in the positions: 2, 3.
    __CC(Arc<__CC<Child>>),
    /// Children of a node having children in the positions: 1.
    _C__(Arc<_C__<Child>>),
    /// Children of a node having children in the positions: 1, 3.
    _C_C(Arc<_C_C<Child>>),
    /// Children of a node having children in the positions: 1, 2.
    _CC_(Arc<_CC_<Child>>),
    /// Children of a node having children in the positions: 1, 2, 3.
    _CCC(Arc<_CCC<Child>>),
    /// Children of a node having children in the positions: 0.
    C___(Arc<C___<Child>>),
    /// Children of a node having children in the positions: 0, 3.
    C__C(Arc<C__C<Child>>),
    /// Children of a node having children in the positions: 0, 2.
    C_C_(Arc<C_C_<Child>>),
    /// Children of a node having children in the positions: 0, 2, 3.
    C_CC(Arc<C_CC<Child>>),
    /// Children of a node having children in the positions: 0, 1.
    CC__(Arc<CC__<Child>>),
    /// Children of a node having children in the positions: 0, 1, 3.
    CC_C(Arc<CC_C<Child>>),
    /// Children of a node having children in the positions: 0, 1, 2.
    CCC_(Arc<CCC_<Child>>),
    /// Children of a node having children in the positions: 0, 1, 2, 3.
    CCCC(Arc<CCCC<Child>>),
}

impl<Child: Debug + Clone> Debug for Children<Child> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.children().fmt(f)
    }
}

impl<Child: Height> Height for Children<Child> {
    type Height = Succ<<Child as Height>::Height>;
}

impl<Child: Height + GetHash + Clone> GetHash for Children<Child> {
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
            [Hash(a), Hash(b), Hash(c), Keep(d)] => Children::___C(Arc::new(___C(a, b, c, d))),
            [Hash(a), Hash(b), Keep(c), Hash(d)] => Children::__C_(Arc::new(__C_(a, b, c, d))),
            [Hash(a), Hash(b), Keep(c), Keep(d)] => Children::__CC(Arc::new(__CC(a, b, c, d))),
            [Hash(a), Keep(b), Hash(c), Hash(d)] => Children::_C__(Arc::new(_C__(a, b, c, d))),
            [Hash(a), Keep(b), Hash(c), Keep(d)] => Children::_C_C(Arc::new(_C_C(a, b, c, d))),
            [Hash(a), Keep(b), Keep(c), Hash(d)] => Children::_CC_(Arc::new(_CC_(a, b, c, d))),
            [Hash(a), Keep(b), Keep(c), Keep(d)] => Children::_CCC(Arc::new(_CCC(a, b, c, d))),
            [Keep(a), Hash(b), Hash(c), Hash(d)] => Children::C___(Arc::new(C___(a, b, c, d))),
            [Keep(a), Hash(b), Hash(c), Keep(d)] => Children::C__C(Arc::new(C__C(a, b, c, d))),
            [Keep(a), Hash(b), Keep(c), Hash(d)] => Children::C_C_(Arc::new(C_C_(a, b, c, d))),
            [Keep(a), Hash(b), Keep(c), Keep(d)] => Children::C_CC(Arc::new(C_CC(a, b, c, d))),
            [Keep(a), Keep(b), Hash(c), Hash(d)] => Children::CC__(Arc::new(CC__(a, b, c, d))),
            [Keep(a), Keep(b), Hash(c), Keep(d)] => Children::CC_C(Arc::new(CC_C(a, b, c, d))),
            [Keep(a), Keep(b), Keep(c), Hash(d)] => Children::CCC_(Arc::new(CCC_(a, b, c, d))),
            [Keep(a), Keep(b), Keep(c), Keep(d)] => Children::CCCC(Arc::new(CCCC(a, b, c, d))),
        })
    }
}

impl<Child: Clone> Children<Child> {
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

    /// Get an array of references to the children or hashes stored in this [`Children`].
    pub fn children_mut(&mut self) -> [InsertMut<'_, Child>; 4] {
        use Children::*;
        use InsertMut::*;

        match self {
            ___C(c) => {
                let c = Arc::make_mut(c);
                [
                    Hash(&mut c.0),
                    Hash(&mut c.1),
                    Hash(&mut c.2),
                    Keep(&mut c.3),
                ]
            }
            __C_(c) => {
                let c = Arc::make_mut(c);
                [
                    Hash(&mut c.0),
                    Hash(&mut c.1),
                    Keep(&mut c.2),
                    Hash(&mut c.3),
                ]
            }
            __CC(c) => {
                let c = Arc::make_mut(c);
                [
                    Hash(&mut c.0),
                    Hash(&mut c.1),
                    Keep(&mut c.2),
                    Keep(&mut c.3),
                ]
            }
            _C__(c) => {
                let c = Arc::make_mut(c);
                [
                    Hash(&mut c.0),
                    Keep(&mut c.1),
                    Hash(&mut c.2),
                    Hash(&mut c.3),
                ]
            }
            _C_C(c) => {
                let c = Arc::make_mut(c);
                [
                    Hash(&mut c.0),
                    Keep(&mut c.1),
                    Hash(&mut c.2),
                    Keep(&mut c.3),
                ]
            }
            _CC_(c) => {
                let c = Arc::make_mut(c);
                [
                    Hash(&mut c.0),
                    Keep(&mut c.1),
                    Keep(&mut c.2),
                    Hash(&mut c.3),
                ]
            }
            _CCC(c) => {
                let c = Arc::make_mut(c);
                [
                    Hash(&mut c.0),
                    Keep(&mut c.1),
                    Keep(&mut c.2),
                    Keep(&mut c.3),
                ]
            }
            C___(c) => {
                let c = Arc::make_mut(c);
                [
                    Keep(&mut c.0),
                    Hash(&mut c.1),
                    Hash(&mut c.2),
                    Hash(&mut c.3),
                ]
            }
            C__C(c) => {
                let c = Arc::make_mut(c);
                [
                    Keep(&mut c.0),
                    Hash(&mut c.1),
                    Hash(&mut c.2),
                    Keep(&mut c.3),
                ]
            }
            C_C_(c) => {
                let c = Arc::make_mut(c);
                [
                    Keep(&mut c.0),
                    Hash(&mut c.1),
                    Keep(&mut c.2),
                    Hash(&mut c.3),
                ]
            }
            C_CC(c) => {
                let c = Arc::make_mut(c);
                [
                    Keep(&mut c.0),
                    Hash(&mut c.1),
                    Keep(&mut c.2),
                    Keep(&mut c.3),
                ]
            }
            CC__(c) => {
                let c = Arc::make_mut(c);
                [
                    Keep(&mut c.0),
                    Keep(&mut c.1),
                    Hash(&mut c.2),
                    Hash(&mut c.3),
                ]
            }
            CC_C(c) => {
                let c = Arc::make_mut(c);
                [
                    Keep(&mut c.0),
                    Keep(&mut c.1),
                    Hash(&mut c.2),
                    Keep(&mut c.3),
                ]
            }
            CCC_(c) => {
                let c = Arc::make_mut(c);
                [
                    Keep(&mut c.0),
                    Keep(&mut c.1),
                    Keep(&mut c.2),
                    Hash(&mut c.3),
                ]
            }
            CCCC(c) => {
                let c = Arc::make_mut(c);
                [
                    Keep(&mut c.0),
                    Keep(&mut c.1),
                    Keep(&mut c.2),
                    Keep(&mut c.3),
                ]
            }
        }
    }
}

impl<Child: Clone> From<Children<Child>> for [Insert<Child>; 4] {
    /// Get an array of references to the children or hashes stored in this [`Children`].
    fn from(children: Children<Child>) -> [Insert<Child>; 4] {
        use Children::*;
        use Insert::*;

        match children {
            ___C(c) => [Hash(c.0), Hash(c.1), Hash(c.2), Keep(c.3.clone())],
            __C_(c) => [Hash(c.0), Hash(c.1), Keep(c.2.clone()), Hash(c.3)],
            __CC(c) => [Hash(c.0), Hash(c.1), Keep(c.2.clone()), Keep(c.3.clone())],
            _C__(c) => [Hash(c.0), Keep(c.1.clone()), Hash(c.2), Hash(c.3)],
            _C_C(c) => [Hash(c.0), Keep(c.1.clone()), Hash(c.2), Keep(c.3.clone())],
            _CC_(c) => [Hash(c.0), Keep(c.1.clone()), Keep(c.2.clone()), Hash(c.3)],
            _CCC(c) => [
                Hash(c.0),
                Keep(c.1.clone()),
                Keep(c.2.clone()),
                Keep(c.3.clone()),
            ],
            C___(c) => [Keep(c.0.clone()), Hash(c.1), Hash(c.2), Hash(c.3)],
            C__C(c) => [Keep(c.0.clone()), Hash(c.1), Hash(c.2), Keep(c.3.clone())],
            C_C_(c) => [Keep(c.0.clone()), Hash(c.1), Keep(c.2.clone()), Hash(c.3)],
            C_CC(c) => [
                Keep(c.0.clone()),
                Hash(c.1),
                Keep(c.2.clone()),
                Keep(c.3.clone()),
            ],
            CC__(c) => [Keep(c.0.clone()), Keep(c.1.clone()), Hash(c.2), Hash(c.3)],
            CC_C(c) => [
                Keep(c.0.clone()),
                Keep(c.1.clone()),
                Hash(c.2),
                Keep(c.3.clone()),
            ],
            CCC_(c) => [
                Keep(c.0.clone()),
                Keep(c.1.clone()),
                Keep(c.2.clone()),
                Hash(c.3),
            ],
            CCCC(c) => [
                Keep(c.0.clone()),
                Keep(c.1.clone()),
                Keep(c.2.clone()),
                Keep(c.3.clone()),
            ],
        }
    }
}
