//! Every possible shape of a non-empty quad-tree node, enumerated as 15 distinct structs.

#![allow(non_camel_case_types, clippy::upper_case_acronyms)]

use crate::prelude::*;

/// Children of a node having children in the positions: 3.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ___C<Child>(pub Hash, pub Hash, pub Hash, pub Child);

/// Children of a node having children in the positions: 2.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct __C_<Child>(pub Hash, pub Hash, pub Child, pub Hash);

/// Children of a node having children in the positions: 2, 3.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct __CC<Child>(pub Hash, pub Hash, pub Child, pub Child);

/// Children of a node having children in the positions: 1.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct _C__<Child>(pub Hash, pub Child, pub Hash, pub Hash);

/// Children of a node having children in the positions: 1, 3.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct _C_C<Child>(pub Hash, pub Child, pub Hash, pub Child);

/// Children of a node having children in the positions: 1, 2.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct _CC_<Child>(pub Hash, pub Child, pub Child, pub Hash);

/// Children of a node having children in the positions: 1, 2, 3.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct _CCC<Child>(pub Hash, pub Child, pub Child, pub Child);

/// Children of a node having children in the positions: 0.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct C___<Child>(pub Child, pub Hash, pub Hash, pub Hash);

/// Children of a node having children in the positions: 0, 3.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct C__C<Child>(pub Child, pub Hash, pub Hash, pub Child);

/// Children of a node having children in the positions: 0, 2.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct C_C_<Child>(pub Child, pub Hash, pub Child, pub Hash);

/// Children of a node having children in the positions: 0, 2, 3.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct C_CC<Child>(pub Child, pub Hash, pub Child, pub Child);

/// Children of a node having children in the positions: 0, 1.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CC__<Child>(pub Child, pub Child, pub Hash, pub Hash);

/// Children of a node having children in the positions: 0, 1, 3.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CC_C<Child>(pub Child, pub Child, pub Hash, pub Child);

/// Children of a node having children in the positions: 0, 1, 2.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CCC_<Child>(pub Child, pub Child, pub Child, pub Hash);

/// Children of a node having children in the positions: 0, 1, 2, 3.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CCCC<Child>(pub Child, pub Child, pub Child, pub Child);
