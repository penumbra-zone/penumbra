use rand_core::{CryptoRng, RngCore};

use crate::{limb, proofs, Ciphertext, EncryptionKey};

/// A plaintext integer value.
///
/// While only encryptions of 64-bit values are supported, the `Value` type
/// holds a `u128` internally, because the sum of 64-bit values may exceed 64
/// bits.  Attempting to encrypt a `Value` bigger than 64 bits will fail.
#[derive(Default, PartialEq)]
pub struct Value(pub u128);

impl From<u64> for Value {
    fn from(v: u64) -> Value {
        Value(v as u128)
    }
}

impl Value {
    pub(crate) fn to_limbs(&self) -> [limb::Value; 4] {
        [
            ((self.0 & 0xffff) as u32).into(),
            (((self.0 >> 16) & 0xffff) as u32).into(),
            (((self.0 >> 32) & 0xffff) as u32).into(),
            (((self.0 >> 48) & 0xffff) as u32).into(),
        ]
    }

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
        Value(x0 | (x1 << 16) | (x2 << 32) | (x3 << 48))
    }

    pub fn transparent_encrypt<R: RngCore + CryptoRng>(
        &self,
        encryption_key: &EncryptionKey,
        mut rng: R,
    ) -> anyhow::Result<(Ciphertext, proofs::TransparentEncryptionProof)> {
        let encrypted_limbs = self
            .to_limbs()
            .iter()
            .map(|limb| limb.transparent_encrypt(encryption_key, &mut rng))
            .collect::<Vec<_>>();

        let mut blindings: [decaf377::Fr; 4] = Default::default();
        for (i, limb) in encrypted_limbs.iter().enumerate() {
            blindings[i] = limb.1;
        }

        let ciphertext = Ciphertext {
            c0: encrypted_limbs[0].0,
            c1: encrypted_limbs[1].0,
            c2: encrypted_limbs[2].0,
            c3: encrypted_limbs[3].0,
        };

        let proof = proofs::TransparentEncryptionProof::new(self.0 as u64, blindings);

        Ok((ciphertext, proof))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use ark_ff::UniformRand;

    #[test]
    fn test_limb_decomposition() {
        let value = Value(6545536u128);
        let limbs = value.to_limbs();
        let val_back = Value::from_limbs(limbs[0], limbs[1], limbs[2], limbs[3]);
        assert_eq!(value.0, val_back.0);
    }

    #[test]
    fn test_encrypt_verify_transparent() {
        let mut rng = rand::thread_rng();
        let value = Value::from(0x12345678);
        let encryption_key = EncryptionKey(decaf377::basepoint() * decaf377::Fr::rand(&mut rng));
        let (ciphertext, proof) = value
            .transparent_encrypt(&encryption_key, &mut rng)
            .unwrap();

        assert!(proof.verify(&ciphertext, &encryption_key).is_ok());
    }
}
