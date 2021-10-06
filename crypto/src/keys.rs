use std::convert::{TryFrom, TryInto};

use ark_ff::PrimeField;
use decaf377;
use decaf377_rdsa;
use once_cell::sync::Lazy;
use rand_core::{CryptoRng, RngCore};

use crate::{Fq, Fr};

pub const DIVERSIFIER_LEN_BYTES: usize = 11;
pub const SPEND_LEN_BYTES: usize = 32;
pub const NK_LEN_BYTES: usize = 32;
pub const OVK_LEN_BYTES: usize = 32;
pub const IVK_LEN_BYTES: usize = 32;

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
    pub ask: SpendAuthorizationKey,
    pub nsk: NullifierPrivateKey,
    pub ovk: OutgoingViewingKey,
}

impl ExpandedSpendingKey {
    pub fn derive(key: &SpendingKey) -> Self {
        Self {
            ask: SpendAuthorizationKey::derive(&key),
            nsk: NullifierPrivateKey::derive(&key),
            ovk: OutgoingViewingKey::derive(&key),
        }
    }
}

#[derive(Clone, Copy)]
pub struct SpendAuthorizationKey(pub decaf377_rdsa::SigningKey<decaf377_rdsa::SpendAuth>);

impl SpendAuthorizationKey {
    pub fn derive(key: &SpendingKey) -> Self {
        let mut hasher = blake2b_simd::State::new();
        hasher.update(b"Penumbra_ExpandSeed");
        hasher.update(&key.0);
        hasher.update(&[0; 1]);
        let hash_result = hasher.finalize();

        let ask_bytes: [u8; SPEND_LEN_BYTES] = hash_result.as_bytes()[0..SPEND_LEN_BYTES]
            .try_into()
            .expect("hash is long enough to convert to array");
        Self(decaf377_rdsa::SigningKey::try_from(ask_bytes).expect("can create SigningKey"))
    }

    pub fn randomize(
        &self,
        randomizer: Fr,
    ) -> decaf377_rdsa::VerificationKey<decaf377_rdsa::SpendAuth> {
        let rk: decaf377_rdsa::VerificationKey<decaf377_rdsa::SpendAuth> =
            self.0.randomize(&randomizer).into();
        rk
    }
}

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
    pub fn derive(ak: &AuthorizationKey, nk: &NullifierDerivingKey) -> Self {
        let mut hasher = blake2b_simd::State::new();
        hasher.update(b"Penumbra_IncomingViewingKey");
        let ak_bytes: [u8; SPEND_LEN_BYTES] = ak.0.into();
        hasher.update(&ak_bytes);
        let nk_bytes: [u8; NK_LEN_BYTES] = nk.0.compress().into();
        hasher.update(&nk_bytes);
        let hash_result = hasher.finalize();

        Self(Fr::from_le_bytes_mod_order(hash_result.as_bytes()))
    }
}

/// An `OutgoingViewingKey` allows one to identify outgoing notes.
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
    pub ak: AuthorizationKey,
    pub nsk: NullifierPrivateKey,
}

pub struct AuthorizationKey(pub decaf377_rdsa::VerificationKey<decaf377_rdsa::SpendAuth>);

impl AuthorizationKey {
    #[allow(non_snake_case)]
    /// Derive a verification key from the corresponding `SpendAuthorizationKey`.
    pub fn derive(ask: &SpendAuthorizationKey) -> Self {
        Self(ask.0.into())
    }
}

/// The `FullViewingKey` allows one to identify incoming and outgoing notes only.
pub struct FullViewingKey {
    pub ak: AuthorizationKey,
    pub nk: NullifierDerivingKey,
    pub ovk: OutgoingViewingKey,
}

pub struct NullifierDerivingKey(decaf377::Element);

#[derive(Copy, Clone)]
pub struct Diversifier(pub [u8; DIVERSIFIER_LEN_BYTES]);

/// The domain separator used to generate diversified generators.
static DIVERSIFY_GENERATOR_DOMAIN_SEP: Lazy<Fq> = Lazy::new(|| {
    Fq::from_le_bytes_mod_order(blake2b_simd::blake2b(b"penumbra.diversifier.generator").as_bytes())
});

impl Diversifier {
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
    pub fn derive_transmission_key(&self, ivk: IncomingViewingKey) -> TransmissionKey {
        let B_d = self.diversified_generator();
        TransmissionKey(ivk.0 * B_d)
    }
}

/// Represents a (diversified) transmission key.
pub struct TransmissionKey(pub decaf377::Element);
