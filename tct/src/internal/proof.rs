//! Transparent merkle inclusion proofs defined generically for trees of any height.
//!
//! These are wrapped in mode specific domain types by the exposed crate API to make it more
//! comprehensible.

use std::fmt::Debug;

use thiserror::Error;

use super::path::{self, AuthPath};
use crate::{Commitment, Hash, Height};

/// A proof of inclusion for a single [`Commitment`](crate::Commitment) commitment in a tree.
#[derive(Derivative)]
#[derivative(
    Debug(bound = "<Tree::Height as path::Path>::Path: Debug"),
    Clone(bound = "<Tree::Height as path::Path>::Path: Clone"),
    PartialEq(bound = "<Tree::Height as path::Path>::Path: PartialEq"),
    Eq(bound = "<Tree::Height as path::Path>::Path: Eq")
)]
pub struct Proof<Tree: Height> {
    pub(crate) position: u64,
    pub(crate) auth_path: AuthPath<Tree>,
    pub(crate) leaf: Commitment,
}

impl<Tree: Height> Proof<Tree> {
    /// Verify a [`Proof`] of inclusion against the root [`struct@Hash`] of a tree.
    ///
    /// Returns [`VerifyError`] if the proof is invalid.
    pub fn verify(&self, root: Hash) -> Result<(), VerifyError> {
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
    pub fn auth_path(&self) -> &AuthPath<Tree> {
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
pub struct VerifyError {
    root: Hash,
}

impl VerifyError {
    /// Get the root hash against which the proof failed to verify.
    pub fn root(&self) -> Hash {
        self.root
    }
}

/// When deserializing a proof, it was malformed.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Error)]
#[error("could not decode proof")]
pub struct ProofDecodeError;

// TODO: re-enable these protobuf impls once we adapt the protobuf crate to these types:

/*
use decaf377::{FieldExt, Fq};
use penumbra_proto::transparent_proofs as pb;

impl<Tree: Height> From<Proof<Tree>> for pb::MerkleProof
where
    Vec<pb::MerklePathChunk>: From<AuthPath<Tree>>,
{
    fn from(proof: Proof<Tree>) -> Self {
        Self {
            position: proof.position,
            auth_path: proof.auth_path.into(),
            note_commitment: proof.leaf.0.to_bytes().to_vec(),
        }
    }
}

impl<Tree: Height> TryFrom<pb::MerkleProof> for Proof<Tree>
where
    AuthPath<Tree>: TryFrom<Vec<pb::MerklePathChunk>>,
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
*/
