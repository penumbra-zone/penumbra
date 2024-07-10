//! Encryption correctness proofs (WIP: currently, only placeholder "transparent
//! proofs").

use crate::{Ciphertext, EncryptionKey, Value};

/// Placeholder for a zk-SNARK proof that the encryption is well-formed.
///
/// Note: this proof reveals the ciphertext!!!
pub struct TransparentEncryptionProof {
    value: u64,
    blindings: [decaf377::Fr; 4],
}

impl TransparentEncryptionProof {
    pub fn new(value: u64, blindings: [decaf377::Fr; 4]) -> Self {
        TransparentEncryptionProof { value, blindings }
    }

    pub fn verify(&self, ctxt: &Ciphertext, encryption_key: &EncryptionKey) -> anyhow::Result<()> {
        let limbs = Value::from(self.value).to_limbs()?;
        let ctxts = [ctxt.c0, ctxt.c1, ctxt.c2, ctxt.c3];

        for i in 0..4 {
            let ctxt = &ctxts[i];
            let blinding = &self.blindings[i];
            let limb = &limbs[i];

            let c1 = blinding * decaf377::Element::GENERATOR;
            let c2 = blinding * encryption_key.0
                + decaf377::Fr::from(limb.0) * decaf377::Element::GENERATOR;

            if c1 != ctxt.c1 || c2 != ctxt.c2 {
                anyhow::bail!("TransparentEncryptionProof: verification failed");
            }
        }

        Ok(())
    }
}
