//! Proofs of inclusion in the tree: how to create them, and how to verify them.

use std::fmt::Debug;

use decaf377::{FieldExt, Fq};
use penumbra_proto::transparent_proofs as pb;
use thiserror::Error;

use super::{
    hash,
    path::{self, AuthPath},
};
use crate::{Commitment, Hash, Height};

/// A proof of inclusion for a single [`Commitment`](crate::Commitment) commitment in a tree.
#[derive(Derivative)]
#[derivative(
    Debug(bound = "<Tree::Height as path::Path<Hasher>>::Path: Debug"),
    Clone(bound = "<Tree::Height as path::Path<Hasher>>::Path: Clone"),
    PartialEq(bound = "<Tree::Height as path::Path<Hasher>>::Path: PartialEq"),
    Eq(bound = "<Tree::Height as path::Path<Hasher>>::Path: Eq")
)]
pub struct Proof<Tree: Height, Hasher>
where
    Tree::Height: path::Path<Hasher>,
{
    pub(crate) position: u64,
    pub(crate) auth_path: AuthPath<Tree, Hasher>,
    pub(crate) leaf: Commitment,
}

impl<Tree: Height, Hasher: hash::Hasher> Proof<Tree, Hasher>
where
    Tree::Height: path::Path<Hasher>,
{
    /// Verify a [`Proof`] of inclusion against the root [`struct@Hash`] of a tree.
    ///
    /// Returns [`VerifyError`] if the proof is invalid.
    pub fn verify(&self, root: Hash<Hasher>) -> Result<(), VerifyError<Hasher>> {
        use path::Path;

        if root == Tree::Height::root(&self.auth_path, self.position, Hash::of(self.leaf)) {
            Ok(())
        } else {
            Err(VerifyError { root })
        }
    }

    /// Get the index of the item this proof claims to witness.
    pub fn index(&self) -> u64 {
        self.position
    }

    /// Get the [`AuthPath`] of this proof, representing the path from the root to the leaf of the
    /// tree that proves the leaf was included in the tree.
    pub fn auth_path(&self) -> &AuthPath<Tree, Hasher> {
        &self.auth_path
    }

    /// Get the commitment whose inclusion is witnessed by the proof.
    pub fn item(&self) -> Commitment {
        self.leaf
    }
}

/// A proof of inclusion did not verify against the provided root hash.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("invalid inclusion proof for root hash {root:?}")]
pub struct VerifyError<Hasher> {
    root: Hash<Hasher>,
}

impl<Hasher> VerifyError<Hasher> {
    /// Get the root hash against which the proof failed to verify.
    pub fn root(&self) -> Hash<Hasher> {
        self.root
    }
}

impl<Tree: Height, Hasher> From<Proof<Tree, Hasher>> for pb::MerkleProof
where
    Vec<pb::MerklePathChunk>: From<AuthPath<Tree, Hasher>>,
    Tree::Height: path::Path<Hasher>,
{
    fn from(proof: Proof<Tree, Hasher>) -> Self {
        Self {
            position: proof.position,
            auth_path: proof.auth_path.into(),
            note_commitment: proof.leaf.0.to_bytes().to_vec(),
        }
    }
}

/// When deserializing a proof, it was malformed.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Error)]
#[error("could not decode proof")]
pub struct ProofDecodeError;

impl<Tree: Height, Hasher> TryFrom<pb::MerkleProof> for Proof<Tree, Hasher>
where
    AuthPath<Tree, Hasher>: TryFrom<Vec<pb::MerklePathChunk>>,
    Tree::Height: path::Path<Hasher>,
{
    type Error = ProofDecodeError;

    fn try_from(proof: pb::MerkleProof) -> Result<Self, Self::Error> {
        let position = proof.position;
        let auth_path = proof.auth_path.try_into().map_err(|_| ProofDecodeError)?;
        let leaf = Fq::from_bytes(
            proof
                .note_commitment
                .try_into()
                .map_err(|_| ProofDecodeError)?,
        )
        .map_err(|_| ProofDecodeError)?
        .try_into()
        .map_err(|_| ProofDecodeError)?;

        Ok(Self {
            position,
            auth_path,
            leaf,
        })
    }
}
