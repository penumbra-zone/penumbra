pub use thiserror::Error;

pub use crate::Commitment;

pub use super::{Eternity, Root};

/// An inclusion proof in a [`Eternity`] which has not yet been verified.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Proof(pub(super) crate::proof::Proof<Eternity>);

impl Proof {
    /// Verify a [`Proof`] of inclusion against the [`Root`] of an [`Eternity`].
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

/// A verified inclusion [`Proof`] in an [`Eternity`], witnessing the presence of a single [`Commitment`].
///
/// The only way to produce this is via [`Proof::verify`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedProof(crate::proof::VerifiedProof<Eternity>);

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

/// A [`Proof`] of inclusion did not verify against the provided root of the [`Eternity`].
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("invalid inclusion proof for eternity root hash {0:?}")]
pub struct VerifyError(crate::proof::VerifyError<Eternity>);

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
