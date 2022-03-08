//! Every possible shape of a non-empty quad-tree node, enumerated as 15 distinct structs.

#![allow(non_camel_case_types)]

use crate::Hash;

/// Children of a node having children in the positions: 3.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ___C<Child> {
    /// The children or hashes of this node shape.
    pub children: (Hash, Hash, Hash, Child),
}

/// Children of a node having children in the positions: 2.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct __C_<Child> {
    /// The children or hashes of this node shape.
    pub children: (Hash, Hash, Child, Hash),
}

/// Children of a node having children in the positions: 2, 3.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct __CC<Child> {
    /// The children or hashes of this node shape.
    pub children: (Hash, Hash, Child, Child),
}

/// Children of a node having children in the positions: 1.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct _C__<Child> {
    /// The children or hashes of this node shape.
    pub children: (Hash, Child, Hash, Hash),
}

/// Children of a node having children in the positions: 1, 3.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct _C_C<Child> {
    /// The children or hashes of this node shape.
    pub children: (Hash, Child, Hash, Child),
}

/// Children of a node having children in the positions: 1, 2.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct _CC_<Child> {
    /// The children or hashes of this node shape.
    pub children: (Hash, Child, Child, Hash),
}

/// Children of a node having children in the positions: 1, 2, 3.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct _CCC<Child> {
    /// The children or hashes of this node shape.
    pub children: (Hash, Child, Child, Child),
}

/// Children of a node having children in the positions: 0.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct C___<Child> {
    /// The children or hashes of this node shape.
    pub children: (Child, Hash, Hash, Hash),
}

/// Children of a node having children in the positions: 0, 3.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct C__C<Child> {
    /// The children or hashes of this node shape.
    pub children: (Child, Hash, Hash, Child),
}

/// Children of a node having children in the positions: 0, 2.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct C_C_<Child> {
    /// The children or hashes of this node shape.
    pub children: (Child, Hash, Child, Hash),
}

/// Children of a node having children in the positions: 0, 2, 3.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct C_CC<Child> {
    /// The children or hashes of this node shape.
    pub children: (Child, Hash, Child, Child),
}

/// Children of a node having children in the positions: 0, 1.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CC__<Child> {
    /// The children or hashes of this node shape.
    pub children: (Child, Child, Hash, Hash),
}

/// Children of a node having children in the positions: 0, 1, 3.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CC_C<Child> {
    /// The children or hashes of this node shape.
    pub children: (Child, Child, Hash, Child),
}

/// Children of a node having children in the positions: 0, 1, 2.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CCC_<Child> {
    /// The children or hashes of this node shape.
    pub children: (Child, Child, Child, Hash),
}

/// Children of a node having children in the positions: 0, 1, 2, 3.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CCCC<Child> {
    /// The children or hashes of this node shape.
    pub children: (Child, Child, Child, Child),
}
