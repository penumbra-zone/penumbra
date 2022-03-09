//! Proofs of inclusion in the tree: how to create them, and how to verify them.

// TODO: make errors implement Error

use std::fmt::Debug;

use super::path::{self, AuthPath};
use crate::{Fq, Hash, Height, Witness};

/// A proof of inclusion for a single [`Fq`](crate::Fq) commitment in a tree.
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
    leaf: Fq,
}

impl<Tree: Height> Proof<Tree> {
    /// Verify a [`Proof`] of inclusion against the root [`struct@Hash`] of a tree.
    ///
    /// Returns a [`VerifiedProof`] if and only if this proof verified against the hash.
    pub fn verify(self, root: Hash) -> Result<VerifiedProof<Tree>, VerifyError<Tree>> {
        use path::Path;

        if root == Tree::Height::root(&self.auth_path, self.index, Hash::of(self.leaf)) {
            Ok(VerifiedProof { proof: self, root })
        } else {
            Err(VerifyError { proof: self, root })
        }
    }

    /// Create a proof of inclusion for the given index in the tree, or return `None` if the index
    /// does not have a witness in the tree.
    pub fn prove(index: usize, item: Fq, tree: &Tree) -> Result<Self, ProveError>
    where
        Tree: Witness,
    {
        // Try to witness this index in the tree and retrieve the authentication path and leaf hash
        if let Some((auth_path, leaf)) = tree.witness(index) {
            // Check that the leaf hash matches the provided item
            if leaf == Hash::of(item) {
                Ok(Self {
                    index,
                    auth_path,
                    leaf: item,
                })
            } else {
                Err(ProveError::ItemHashMismatch)
            }
        } else {
            Err(ProveError::IndexNotWitnessed)
        }
    }

    /// Get the index of the item this proof claims to witness.
    pub fn index(&self) -> usize {
        self.index
    }

    /// Get the [`AuthPath`] of this proof, representing the path from the root to the leaf of the
    /// tree that proves the leaf was included in the tree.
    pub fn auth_path(&self) -> &AuthPath<Tree> {
        &self.auth_path
    }

    /// Get the commitment whose inclusion is witnessed by the proof.
    pub fn item(&self) -> Fq {
        self.leaf
    }
}

/// A proof of inclusion could not be created for the given index and item in the tree.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ProveError {
    /// The index provided is not witnessed in the tree.
    IndexNotWitnessed,
    /// The item provided does not match the hash witnessed in the tree at the index.
    ItemHashMismatch,
}

/// A proof of inclusion did not verify against the provided root hash.
#[derive(Derivative)]
#[derivative(
    Debug(bound = "<Tree::Height as path::Path>::Path: Debug"),
    Clone(bound = "<Tree::Height as path::Path>::Path: Clone"),
    PartialEq(bound = "<Tree::Height as path::Path>::Path: PartialEq"),
    Eq(bound = "<Tree::Height as path::Path>::Path: Eq")
)]
pub struct VerifyError<Tree: Height> {
    proof: Proof<Tree>,
    root: Hash,
}

impl<Tree: Height> VerifyError<Tree> {
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
