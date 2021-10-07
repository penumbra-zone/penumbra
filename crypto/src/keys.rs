use std::convert::{TryFrom, TryInto};

use crate::{Fq, Fr};
use ark_ff::PrimeField;
use decaf377;
use decaf377::FrExt;
use decaf377_rdsa;
use once_cell::sync::Lazy;
use rand_core::{CryptoRng, RngCore};

pub const DIVERSIFIER_LEN_BYTES: usize = 11;
pub const SPEND_LEN_BYTES: usize = 32;
pub const NK_LEN_BYTES: usize = 32;
pub const OVK_LEN_BYTES: usize = 32;
pub const IVK_LEN_BYTES: usize = 32;

pub use decaf377_rdsa::SigningKey;
pub use decaf377_rdsa::SpendAuth;
pub use decaf377_rdsa::VerificationKey;

pub struct SpendingKey(pub [u8; SPEND_LEN_BYTES]);

impl SpendingKey {
    pub fn generate<R: RngCore + CryptoRng>(mut rng: R) -> Self {
        let mut key = [0u8; SPEND_LEN_BYTES];
        // Better to use Rng trait here (instead of RngCore)?
        rng.fill_bytes(&mut key);
        SpendingKey(key)
    }
}

pub struct ExpandedSpendingKey {
    pub ask: SigningKey<SpendAuth>,
    pub nsk: NullifierPrivateKey,
    pub ovk: OutgoingViewingKey,
}

impl ExpandedSpendingKey {
    pub fn derive(key: &SpendingKey) -> Self {
        // Generation of the spend authorization key.
        let mut hasher = blake2b_simd::State::new();
        hasher.update(b"Penumbra_ExpandSeed");
        hasher.update(&key.0);
        hasher.update(&[0; 1]);
        let hash_result = hasher.finalize();

        let ask_bytes: [u8; SPEND_LEN_BYTES] = hash_result.as_bytes()[0..SPEND_LEN_BYTES]
            .try_into()
            .expect("hash is long enough to convert to array");

        let field_elem = Fr::from_le_bytes_mod_order(&ask_bytes);
        let ask = SigningKey::try_from(field_elem.to_bytes()).expect("can create SigningKey");

        Self {
            ask,
            nsk: NullifierPrivateKey::derive(&key),
            ovk: OutgoingViewingKey::derive(&key),
        }
    }
}

#[derive(Copy, Clone)]
pub struct NullifierPrivateKey(pub Fr);

impl NullifierPrivateKey {
    pub fn derive(key: &SpendingKey) -> Self {
        let mut hasher = blake2b_simd::State::new();
        hasher.update(b"Penumbra_ExpandSeed");
        hasher.update(&key.0);
        hasher.update(&[1; 1]);
        let hash_result = hasher.finalize();

        Self(Fr::from_le_bytes_mod_order(hash_result.as_bytes()))
    }
}

/// An `IncomingViewingKey` allows one to identify incoming notes.
pub struct IncomingViewingKey(pub Fr);

impl IncomingViewingKey {
    pub fn derive(ak: &VerificationKey<SpendAuth>, nk: &NullifierDerivingKey) -> Self {
        let mut hasher = blake2b_simd::State::new();
        hasher.update(b"Penumbra_IncomingViewingKey");
        let ak_bytes: [u8; SPEND_LEN_BYTES] = ak.into();
        hasher.update(&ak_bytes);
        let nk_bytes: [u8; NK_LEN_BYTES] = nk.0.compress().into();
        hasher.update(&nk_bytes);
        let hash_result = hasher.finalize();

        Self(Fr::from_le_bytes_mod_order(hash_result.as_bytes()))
    }
}

/// An `OutgoingViewingKey` allows one to identify outgoing notes.
#[derive(Copy, Clone)]
pub struct OutgoingViewingKey(pub [u8; OVK_LEN_BYTES]);

impl OutgoingViewingKey {
    pub fn derive(key: &SpendingKey) -> Self {
        let mut hasher = blake2b_simd::State::new();
        hasher.update(b"Penumbra_ExpandSeed");
        hasher.update(&key.0);
        hasher.update(&[2; 1]);
        let hash_result = hasher.finalize();

        Self(
            hash_result.as_bytes()[0..OVK_LEN_BYTES]
                .try_into()
                .expect("hash is long enough to convert to array"),
        )
    }
}

pub struct EphemeralPublicKey(pub decaf377::Element);

// This is going away when key agreement is in place
impl EphemeralPublicKey {
    pub fn new() -> EphemeralPublicKey {
        todo!("key agreement")
    }
}

pub struct ProofAuthorizationKey {
    pub ak: VerificationKey<SpendAuth>,
    pub nsk: NullifierPrivateKey,
}

impl ProofAuthorizationKey {
    pub fn derive(ask: &SigningKey<SpendAuth>, nsk: &NullifierPrivateKey) -> Self {
        Self {
            ak: ask.into(),
            nsk: *nsk,
        }
    }
}

/// The `FullViewingKey` allows one to identify incoming and outgoing notes only.
pub struct FullViewingKey {
    pub ak: VerificationKey<SpendAuth>,
    pub nk: NullifierDerivingKey,
    pub ovk: OutgoingViewingKey,
}

impl FullViewingKey {
    pub fn derive(
        ak: &VerificationKey<SpendAuth>,
        nsk: &NullifierPrivateKey,
        ovk: &OutgoingViewingKey,
    ) -> Self {
        Self {
            ak: *ak,
            nk: NullifierDerivingKey::derive(&nsk),
            ovk: *ovk,
        }
    }
}

pub struct NullifierDerivingKey(decaf377::Element);

impl NullifierDerivingKey {
    pub fn derive(nsk: &NullifierPrivateKey) -> Self {
        Self(decaf377::Element::basepoint() * nsk.0)
    }
}

#[derive(Copy, Clone)]
pub struct Diversifier(pub [u8; DIVERSIFIER_LEN_BYTES]);

/// The domain separator used to generate diversified generators.
static DIVERSIFY_GENERATOR_DOMAIN_SEP: Lazy<Fq> = Lazy::new(|| {
    Fq::from_le_bytes_mod_order(blake2b_simd::blake2b(b"penumbra.diversifier.generator").as_bytes())
});

impl Diversifier {
    /// Generate a new random diversifier.
    pub fn generate<R: RngCore + CryptoRng>(mut rng: R) -> Self {
        let mut diversifier = [0u8; DIVERSIFIER_LEN_BYTES];
        rng.fill_bytes(&mut diversifier);
        Diversifier(diversifier)
    }

    /// Generate the diversified basepoint.
    pub fn diversified_generator(&self) -> decaf377::Element {
        use crate::poseidon_hash::hash_1;
        let hash = hash_1(
            &DIVERSIFY_GENERATOR_DOMAIN_SEP,
            Fq::from_le_bytes_mod_order(&self.0[..]),
        );
        decaf377::Element::map_to_group_cdh(&hash)
    }

    #[allow(non_snake_case)]
    pub fn derive_transmission_key(&self, ivk: &IncomingViewingKey) -> TransmissionKey {
        let B_d = self.diversified_generator();
        TransmissionKey(ivk.0 * B_d)
    }
}

/// Represents a (diversified) transmission key.
#[derive(Copy, Clone)]
pub struct TransmissionKey(pub decaf377::Element);

#[cfg(test)]
mod tests {
    use super::*;

    use rand_core::OsRng;

    use crate::addresses::PaymentAddress;

    #[test]
    fn complete_key_generation_happy_path() {
        let mut rng = OsRng;
        let diversifier = Diversifier::generate(&mut rng);
        let sk = SpendingKey::generate(&mut rng);
        let expanded_sk = ExpandedSpendingKey::derive(&sk);
        let proof_auth_key = ProofAuthorizationKey::derive(&expanded_sk.ask, &expanded_sk.nsk);
        let fvk = FullViewingKey::derive(&proof_auth_key.ak, &proof_auth_key.nsk, &expanded_sk.ovk);
        let ivk = IncomingViewingKey::derive(&proof_auth_key.ak, &fvk.nk);
        let pk_d = diversifier.derive_transmission_key(&ivk);
        let _dest = PaymentAddress::new(&diversifier, &pk_d);
    }
}
