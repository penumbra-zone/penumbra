//! Transparent merkle inclusion proofs defined generically for trees of any height.
//!
//! These are wrapped in mode specific domain types by the exposed crate API to make it more
//! comprehensible.

use std::fmt::Debug;

use crate::prelude::*;

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
    pub(crate) leaf: StateCommitment,
}

impl<Tree: Height> Proof<Tree> {
    /// Verify a [`Proof`] of inclusion against the root [`struct@Hash`] of a tree.
    ///
    /// Returns [`VerifyError`] if the proof is invalid.
    pub fn verify(&self, root: Hash) -> Result<(), VerifyError> {
        if root == self.root() {
            Ok(())
        } else {
            Err(VerifyError { root })
        }
    }

    /// Get the root of the tree from which the proof was generated.
    pub fn root(&self) -> Hash {
        Tree::Height::root(&self.auth_path, self.position, Hash::of(self.leaf))
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

use decaf377::Fq;
use penumbra_proto::penumbra::crypto::tct::v1 as pb;

impl<Tree: Height> From<Proof<Tree>> for pb::StateCommitmentProof
where
    Vec<pb::MerklePathChunk>: From<AuthPath<Tree>>,
{
    fn from(proof: Proof<Tree>) -> Self {
        Self {
            position: proof.position,
            auth_path: proof.auth_path.into(),
            note_commitment: Some(proof.leaf.into()),
        }
    }
}

impl<Tree: Height> TryFrom<pb::StateCommitmentProof> for Proof<Tree>
where
    AuthPath<Tree>: TryFrom<Vec<pb::MerklePathChunk>>,
{
    type Error = ProofDecodeError;

    fn try_from(proof: pb::StateCommitmentProof) -> Result<Self, Self::Error> {
        let position = proof.position;
        let auth_path = proof.auth_path.try_into().map_err(|_| ProofDecodeError)?;
        let leaf = StateCommitment(
            Fq::from_bytes_checked(
                &proof
                    .note_commitment
                    .ok_or(ProofDecodeError)?
                    .inner
                    .try_into()
                    .map_err(|_| ProofDecodeError)?,
            )
            .map_err(|_| ProofDecodeError)?,
        );

        Ok(Self {
            position,
            auth_path,
            leaf,
        })
    }
}
