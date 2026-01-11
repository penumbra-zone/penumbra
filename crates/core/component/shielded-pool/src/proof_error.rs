//! Error types for zero-knowledge proof generation and verification.

use thiserror::Error;

/// Errors that can occur during proof generation or verification.
#[derive(Debug, Error)]
pub enum ProofError {
    /// Circuit constraints are not satisfied.
    #[error("circuit constraints not satisfied: {0}")]
    UnsatisfiedConstraints(String),

    /// Proof generation failed.
    #[error("proof generation failed: {0}")]
    ProofGenerationFailed(String),

    /// Proof verification failed.
    #[error("proof verification failed: {0}")]
    ProofVerificationFailed(String),

    /// Constraint synthesis error.
    #[error("constraint synthesis error: {0}")]
    SynthesisError(#[from] ark_relations::r1cs::SynthesisError),

    /// Invalid public input.
    #[error("invalid public input: {0}")]
    InvalidPublicInput(String),

    /// Asset registry verification failed.
    #[error("asset registry verification failed: wrong asset anchor or path")]
    AssetRegistryVerificationFailed,

    /// Compliance registry verification failed.
    #[error("compliance registry verification failed: user not registered or wrong path")]
    ComplianceRegistryVerificationFailed,

    /// Compliance ciphertext binding failed.
    #[error("compliance ciphertext binding failed: encryption mismatch")]
    ComplianceCiphertextBindingFailed,

    /// Generic error for other cases.
    #[error("{0}")]
    Other(String),
}

impl From<anyhow::Error> for ProofError {
    fn from(err: anyhow::Error) -> Self {
        ProofError::Other(err.to_string())
    }
}

/// Result type for proof operations.
pub type ProofResult<T> = Result<T, ProofError>;
