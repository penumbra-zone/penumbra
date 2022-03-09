//! Proofs of inclusion in the tree: how to create them, and how to verify them.

use std::fmt::Debug;

use super::path::{self, AuthPath};
use crate::{Hash, Height, Witness};

/// A proof of inclusion for a single commitment in a tree.
#[derive(Derivative)]
#[derivative(
    Debug(bound = "<Tree::Height as path::Path>::Path: Debug"),
    Clone(bound = "<Tree::Height as path::Path>::Path: Clone"),
    PartialEq(bound = "<Tree::Height as path::Path>::Path: PartialEq"),
    Eq(bound = "<Tree::Height as path::Path>::Path: Eq")
)]
pub struct Proof<Tree: Height> {
    index: usize,
    auth_path: AuthPath<Tree>,
    leaf: Hash,
}

impl<Tree: Height> Proof<Tree> {
    /// Verify a [`Proof`] of inclusion against the root hash of a tree.
    ///
    /// Returns `true` if and only if the proof was valid for this root.
    pub fn verify(self, root: Hash) -> Result<VerifiedProof<Tree>, VerificationError<Tree>> {
        if root == <Tree::Height as path::Path>::root(&self.auth_path, self.index, self.leaf) {
            Ok(VerifiedProof { proof: self, root })
        } else {
            Err(VerificationError { proof: self, root })
        }
    }

    /// Create a proof of inclusion for the given index in the tree, or return `None` if the index
    /// does not have a witness in the tree.
    pub fn of_inclusion(index: usize, tree: &Tree) -> Option<Self>
    where
        Tree: Witness,
    {
        tree.witness(index).map(|(auth_path, leaf)| Self {
            index,
            auth_path,
            leaf,
        })
    }
}

/// A proof of inclusion did not verify against the provided root hash.
#[derive(Derivative)]
#[derivative(
    Debug(bound = "<Tree::Height as path::Path>::Path: Debug"),
    Clone(bound = "<Tree::Height as path::Path>::Path: Clone"),
    PartialEq(bound = "<Tree::Height as path::Path>::Path: PartialEq"),
    Eq(bound = "<Tree::Height as path::Path>::Path: Eq")
)]
pub struct VerificationError<Tree: Height> {
    proof: Proof<Tree>,
    root: Hash,
}

impl<Tree: Height> VerificationError<Tree> {
    /// Get a reference to the proof that failed to verify.
    pub fn proof(&self) -> &Proof<Tree> {
        &self.proof
    }

    /// Get the root hash against which the proof failed to verify.
    pub fn root(&self) -> Hash {
        self.root
    }

    /// Extract the original proof from this error.
    pub fn into_proof(self) -> Proof<Tree> {
        self.proof
    }
}

/// A verified proof of inclusion in a tree, at a given root hash.
///
/// The only way to create this is to use [`Proof::verify`], and for it to succeed.
#[derive(Derivative)]
#[derivative(
    Debug(bound = "<Tree::Height as path::Path>::Path: Debug"),
    Clone(bound = "<Tree::Height as path::Path>::Path: Clone"),
    PartialEq(bound = "<Tree::Height as path::Path>::Path: PartialEq"),
    Eq(bound = "<Tree::Height as path::Path>::Path: Eq")
)]
pub struct VerifiedProof<Tree: Height> {
    proof: Proof<Tree>,
    root: Hash,
}

impl<Tree: Height> VerifiedProof<Tree> {
    /// Get a reference to the proof that was verified.
    pub fn proof(&self) -> &Proof<Tree> {
        &self.proof
    }

    /// Get the root hash against which the proof was verified.
    pub fn root(&self) -> Hash {
        self.root
    }

    /// Extract the original (pre-verified) proof from this verified proof.
    pub fn unverify(self) -> Proof<Tree> {
        self.proof
    }
}
