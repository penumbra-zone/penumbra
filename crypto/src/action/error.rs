use thiserror;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Error converting from protobuf: field is missing")]
    ProtobufMissingFieldError,
    #[error("OutputBody could not be converted from protobuf")]
    ProtobufOutputBodyMalformed,
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
}
