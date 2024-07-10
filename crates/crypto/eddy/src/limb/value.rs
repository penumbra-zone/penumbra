use crate::limb::Ciphertext;
use crate::EncryptionKey;
use rand_core::{CryptoRng, RngCore};

/// An individual limb value.
///
/// While only encryptions of 16-bit limbs are supported, the `Value` type
/// holds a `u32` internally, because the sum of 16-bit values may exceed 16
/// bits.
#[derive(Default, Debug, Clone, Copy)]
pub struct Value(pub u32);

impl From<u32> for Value {
    fn from(value: u32) -> Self {
        Value(value)
    }
}

type Blinding = decaf377::Fr;

impl Value {
    pub fn transparent_encrypt<R: RngCore + CryptoRng>(
        &self,
        encryption_key: &EncryptionKey,
        mut rng: R,
    ) -> (Ciphertext, Blinding) {
        let elgamal_blind = decaf377::Fr::rand(&mut rng);
        let c1 = elgamal_blind * decaf377::Element::GENERATOR;
        let c2 = elgamal_blind * encryption_key.0
            + decaf377::Fr::from(self.0) * decaf377::Element::GENERATOR;

        (Ciphertext { c1, c2 }, elgamal_blind)
    }
}
