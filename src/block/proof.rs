pub use thiserror::Error;

pub use crate::Commitment;

pub use super::{Block, Root};

/// An as-yet-unverified proof of the inclusion of some [`Commitment`] in a [`Block`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Proof(pub(super) crate::proof::Proof<Block>);

impl Proof {
    /// Verify a [`Proof`] of inclusion against the [`Root`] of an [`Block`].
    ///
    /// Returns a [`VerifiedProof`] if and only if this proof verified against the hash.
    pub fn verify(self, root: &Root) -> Result<VerifiedProof, VerifyError> {
        self.0
            .verify(root.0)
            .map(VerifiedProof)
            .map_err(VerifyError)
    }

    /// Get the commitment whose inclusion is witnessed by the proof.
    pub fn commitment(&self) -> Commitment {
        self.0.leaf
    }
}

/// A verified [`Proof`] of the inclusion of a single [`Commitment`] in a [`Block`].
///
/// The only way to produce this is via [`Proof::verify`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedProof(crate::proof::VerifiedProof<Block>);

impl VerifiedProof {
    /// Get the root hash against which the proof failed to verify.
    pub fn root(&self) -> Root {
        Root(self.0.root())
    }

    /// Extract the original proof from this error.
    pub fn unverify(self) -> Proof {
        Proof(self.0.unverify())
    }
}

/// A [`Proof`] of inclusion did not verify against the provided root of the [`Block`].
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("invalid inclusion proof for block root hash {0:?}")]
pub struct VerifyError(crate::proof::VerifyError<Block>);

impl VerifyError {
    /// Get the root hash against which the proof failed to verify.
    pub fn root(&self) -> Root {
        Root(self.0.root())
    }

    /// Extract the original proof from this error.
    pub fn into_proof(self) -> Proof {
        Proof(self.0.into_proof())
    }
}
