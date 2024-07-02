use rand_core::{CryptoRng, RngCore};

use crate::{limb, proofs, Ciphertext, EncryptionKey};

/// A plaintext integer value.
///
/// While only encryptions of 64-bit values are supported, the `Value` type
/// holds a `u128` internally, because the sum of 64-bit values may exceed 64
/// bits.  Attempting to encrypt a `Value` bigger than 64 bits will fail.
#[derive(Default, PartialEq, Eq)]
pub struct Value(pub u128);

impl From<u64> for Value {
    fn from(v: u64) -> Value {
        Value(v as u128)
    }
}

impl Value {
    pub(crate) fn to_limbs(&self) -> anyhow::Result<[limb::Value; 4]> {
        let v = u64::try_from(self.0)?;
        Ok([
            ((v & 0xffff) as u32).into(),
            (((v >> 16) & 0xffff) as u32).into(),
            (((v >> 32) & 0xffff) as u32).into(),
            (((v >> 48) & 0xffff) as u32).into(),
        ])
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
        Value(x0 + (x1 << 16) + (x2 << 32) + (x3 << 48))
    }

    /// Encrypt this value to the given [`EncryptionKey`], producing a
    /// [`Ciphertext`] and a (transparent) encryption proof.
    ///
    /// While the transparent encryption proof reveals the ciphertext, the rest
    /// of the encryption should be secure -- this is a stub interface prior to
    /// implementing ZK-SNARK based encryption proofs.
    pub fn transparent_encrypt<R: RngCore + CryptoRng>(
        &self,
        encryption_key: &EncryptionKey,
        mut rng: R,
    ) -> anyhow::Result<(Ciphertext, proofs::TransparentEncryptionProof)> {
        let encrypted_limbs = self
            .to_limbs()?
            .iter()
            .map(|limb| limb.transparent_encrypt(encryption_key, &mut rng))
            .collect::<Vec<_>>();

        let mut blindings: [decaf377::Fr; 4] = Default::default();
        for (i, (_, blinding)) in encrypted_limbs.iter().enumerate() {
            blindings[i] = *blinding;
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

    use proptest::prelude::*;

    proptest! {
        #[test]
        fn limb_value_addition_roundtrip(value1: u64, value2: u64) {
            let value = Value::from(value1);
            let value2 = Value::from(value2);
            let limbs = value.to_limbs().expect("unable to convert to limbs");
            let limbs2 = value2.to_limbs().expect("unable to convert to limbs");
            let limbs3 = [
                limbs[0].0 + limbs2[0].0,
                limbs[1].0 + limbs2[1].0,
                limbs[2].0 + limbs2[2].0,
                limbs[3].0 + limbs2[3].0,
            ];
            let value3 = Value::from_limbs(limbs3[0].into(), limbs3[1].into(), limbs3[2].into(), limbs3[3].into());
            assert_eq!(value3.0, value.0 + value2.0);
        }

        #[test]
        fn limb_value_roundtrip(value: u64) {
            let value = Value::from(value);
            let limbs = value.to_limbs().expect("unable to convert to limbs");
            let value2 = Value::from_limbs(limbs[0], limbs[1], limbs[2], limbs[3]);
            assert_eq!(value.0, value2.0);
        }

        #[test]
        fn encrypt_verify_roundtrip(value: u64) {
            let mut rng = rand::thread_rng();
            let encryption_key = EncryptionKey(decaf377::Element::GENERATOR * decaf377::Fr::rand(&mut rng));
            let value = Value::from(value);
            let (ciphertext, proof) = value
                .transparent_encrypt(&encryption_key, &mut rng)
                .expect("unable to encrypt");

            assert!(proof.verify(&ciphertext, &encryption_key).is_ok());
        }
    }
}
