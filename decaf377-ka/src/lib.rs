use ark_ff::UniformRand;

use decaf377;
use rand_core::{CryptoRng, RngCore};

/// A `SharedSecret` derived at the end of the key agreement protocol.
#[derive(Debug, PartialEq)]
pub struct SharedSecret(pub(crate) decaf377::Element);

impl SharedSecret {
    pub fn derive(other_public_key: &EphemeralPublicKey, secret_key: EphemeralSecretKey) -> Self {
        Self(other_public_key.0 * secret_key.0)
    }
}

/// An `EphemeralSecretKey` is used once and consumed when forming a `SharedSecret`.
pub struct EphemeralSecretKey(pub(crate) decaf377::Fr);

impl EphemeralSecretKey {
    pub fn generate<R: RngCore + CryptoRng>(mut rng: R) -> Self {
        Self(decaf377::Fr::rand(&mut rng))
    }

    pub fn derive_public(&self) -> EphemeralPublicKey {
        EphemeralPublicKey(self.0 * decaf377::Element::basepoint())
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

        let alice_sharedsecret = SharedSecret::derive(&bob_pubkey, alice_secret);
        let bob_sharedsecret = SharedSecret::derive(&alice_pubkey, bob_secret);

        assert_eq!(alice_sharedsecret, bob_sharedsecret);
    }
}
