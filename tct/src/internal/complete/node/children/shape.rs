//! Every possible shape of a non-empty quad-tree node, enumerated as 15 distinct structs.

#![allow(non_camel_case_types, clippy::upper_case_acronyms)]

use serde::{Deserialize, Serialize};

use crate::Hash;

/// Children of a node having children in the positions: 3.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ___C<Child, Hasher>(
    pub Hash<Hasher>,
    pub Hash<Hasher>,
    pub Hash<Hasher>,
    pub Child,
);

/// Children of a node having children in the positions: 2.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct __C_<Child, Hasher>(
    pub Hash<Hasher>,
    pub Hash<Hasher>,
    pub Child,
    pub Hash<Hasher>,
);

/// Children of a node having children in the positions: 2, 3.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct __CC<Child, Hasher>(pub Hash<Hasher>, pub Hash<Hasher>, pub Child, pub Child);

/// Children of a node having children in the positions: 1.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct _C__<Child, Hasher>(
    pub Hash<Hasher>,
    pub Child,
    pub Hash<Hasher>,
    pub Hash<Hasher>,
);

/// Children of a node having children in the positions: 1, 3.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct _C_C<Child, Hasher>(pub Hash<Hasher>, pub Child, pub Hash<Hasher>, pub Child);

/// Children of a node having children in the positions: 1, 2.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct _CC_<Child, Hasher>(pub Hash<Hasher>, pub Child, pub Child, pub Hash<Hasher>);

/// Children of a node having children in the positions: 1, 2, 3.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct _CCC<Child, Hasher>(pub Hash<Hasher>, pub Child, pub Child, pub Child);

/// Children of a node having children in the positions: 0.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct C___<Child, Hasher>(
    pub Child,
    pub Hash<Hasher>,
    pub Hash<Hasher>,
    pub Hash<Hasher>,
);

/// Children of a node having children in the positions: 0, 3.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct C__C<Child, Hasher>(pub Child, pub Hash<Hasher>, pub Hash<Hasher>, pub Child);

/// Children of a node having children in the positions: 0, 2.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct C_C_<Child, Hasher>(pub Child, pub Hash<Hasher>, pub Child, pub Hash<Hasher>);

/// Children of a node having children in the positions: 0, 2, 3.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct C_CC<Child, Hasher>(pub Child, pub Hash<Hasher>, pub Child, pub Child);

/// Children of a node having children in the positions: 0, 1.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CC__<Child, Hasher>(pub Child, pub Child, pub Hash<Hasher>, pub Hash<Hasher>);

/// Children of a node having children in the positions: 0, 1, 3.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CC_C<Child, Hasher>(pub Child, pub Child, pub Hash<Hasher>, pub Child);

/// Children of a node having children in the positions: 0, 1, 2.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CCC_<Child, Hasher>(pub Child, pub Child, pub Child, pub Hash<Hasher>);

/// Children of a node having children in the positions: 0, 1, 2, 3.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CCCC<Child>(pub Child, pub Child, pub Child, pub Child);
