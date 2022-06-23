use crate::{limb, proofs, Ciphertext, EncryptionKey};

/// A plaintext integer value.
///
/// While only encryptions of 64-bit values are supported, the `Value` type
/// holds a `u128` internally, because the sum of 64-bit values may exceed 64
/// bits.  Attempting to encrypt a `Value` bigger than 64 bits will fail.
pub struct Value(pub u128);

impl From<u64> for Value {
    fn from(v: u64) -> Value {
        Value(v as u128)
    }
}

impl Value {
    pub(crate) fn from_limbs(
        x0: limb::Value,
        x1: limb::Value,
        x2: limb::Value,
        x3: limb::Value,
    ) -> Value {
        let x0 = x0.0 as u128;
        let x1 = x1.0 as u128;
        let x2 = x2.0 as u128;
        let x3 = x3.0 as u128;
        Value(x0 + (x1 << 16) + (x2 << 32) + (x3 << 48))
    }

    pub fn encrypt(
        &self,
        encryption_key: &EncryptionKey,
    ) -> anyhow::Result<(Ciphertext, proofs::TransparentEncryptionProof)> {
        todo!()
    }
}
