use thiserror;

#[derive(thiserror::Error, Debug)]
pub enum ProtoError {
    #[error("OutputBody is malformed")]
    OutputBodyMalformed,
    #[error("Memo ciphertext malformed")]
    MemoCiphertextMalformed,
    #[error("Outgoing viewing key malformed")]
    OutgoingViewingKeyMalformed,
    #[error("Value commitment malformed")]
    ValueCommitmentMalformed,
    #[error("Encrypted note malformed")]
    EncryptedNoteMalformed,
    #[error("Ephemeral public key malformed")]
    EphemeralPubKeyMalformed,
    #[error("Note commitment malformed")]
    NoteCommitmentMalformed,
    #[error("Nullifier malformed")]
    NullifierMalformed,
    #[error("Randomized key malformed")]
    RandomizedKeyMalformed,
    #[error("Spend proof malformed")]
    SpendProofMalformed,
}
