use ark_ff::UniformRand;

use decaf377;
use rand_core::{CryptoRng, RngCore};

/// A `SharedSecret` derived at the end of the key agreement protocol.
#[derive(Debug, PartialEq)]
pub struct SharedSecret(pub [u8; 32]);

/// An `EphemeralSecretKey` is used once and consumed when forming a `SharedSecret`.
pub struct EphemeralSecretKey(pub(crate) decaf377::Fr);

impl EphemeralSecretKey {
    pub fn generate<R: RngCore + CryptoRng>(mut rng: R) -> Self {
        Self(decaf377::Fr::rand(&mut rng))
    }

    pub fn derive_public(&self) -> EphemeralPublicKey {
        EphemeralPublicKey(self.0 * decaf377::Element::basepoint())
    }

    pub fn key_agreement_with(self, other: &EphemeralPublicKey) -> SharedSecret {
        SharedSecret((other.0 * self.0).compress().into())
    }
}

/// An `EphemeralPublicKey` sent to the other participant in the key agreement protocol.
pub struct EphemeralPublicKey(pub(crate) decaf377::Element);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_agreement_happy_path() {
        use rand_core::OsRng;

        let mut rng = OsRng;
        let alice_secret = EphemeralSecretKey::generate(&mut rng);
        let bob_secret = EphemeralSecretKey::generate(&mut rng);

        let alice_pubkey = alice_secret.derive_public();
        let bob_pubkey = bob_secret.derive_public();

        let alice_sharedsecret = alice_secret.key_agreement_with(&bob_pubkey);
        let bob_sharedsecret = bob_secret.key_agreement_with(&alice_pubkey);

        assert_eq!(alice_sharedsecret, bob_sharedsecret);
    }
}
