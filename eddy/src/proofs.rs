use crate::{Ciphertext, EncryptionKey};

/// Placeholder for a zk-SNARK proof that the encryption is well-formed.
///
/// Note: this proof reveals the ciphertext!!!
pub struct TransparentEncryptionProof {
    _value: u64,
}

impl TransparentEncryptionProof {
    pub fn check(&self, _ctxt: &Ciphertext, _encryption_key: &EncryptionKey) -> anyhow::Result<()> {
        todo!()
    }
}
