use std::convert::TryInto;

use ark_ff::PrimeField;
use decaf377;
use once_cell::sync::Lazy;
use rand_core::{CryptoRng, RngCore};

use crate::{Fq, Fr};

pub const DIVERSIFIER_LEN_BYTES: usize = 11;
pub const SPEND_LEN_BYTES: usize = 32;
pub const OVK_LEN_BYTES: usize = 32;

pub struct SpendingKey(pub [u8; SPEND_LEN_BYTES]);

impl SpendingKey {
    pub fn generate<R: RngCore + CryptoRng>(mut rng: R) -> Self {
        let mut key = [0u8; SPEND_LEN_BYTES];
        // Better to use Rng trait here?
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

pub struct SpendAuthorizationKey(pub Fr);

impl SpendAuthorizationKey {
    pub fn derive(key: &SpendingKey) -> Self {
        todo!("need to_scalar for decaf377")
    }
}

pub struct NullifierPrivateKey(pub Fr);

impl NullifierPrivateKey {
    pub fn derive(key: &SpendingKey) -> Self {
        let mut input = Vec::<u8>::new();
        input.extend_from_slice(b"Penumbra_ExpandSeed");
        input.extend_from_slice(&key.0);
        input.extend_from_slice(&[0; 1]);
        let hash_input = blake2b_simd::blake2b(&input).as_bytes();
        todo!("need to_scalar for decaf377")
    }
}

pub struct IncomingViewingKey(pub Fr);

impl IncomingViewingKey {
    pub fn derive(ak: AuthorizationKey, nk: NullifierDerivingKey) -> Self {
        let mut input = Vec::<u8>::new();
        input.extend_from_slice(b"Penumbra_IncomingViewingKey");
        input.extend_from_slice(&ak.0.compress().into());
        input.extend_from_slice(&nk.0.compress().into());
        let hash = blake2b_simd::blake2b(&input).as_bytes();
        Self(hash)
    }
}

pub struct OutgoingViewingKey(pub [u8; OVK_LEN_BYTES]);

impl OutgoingViewingKey {
    pub fn derive(key: &SpendingKey) -> Self {
        let mut input = Vec::<u8>::new();
        input.extend_from_slice(b"Penumbra_ExpandSeed");
        input.extend_from_slice(&key.0);
        input.extend_from_slice(&[1; 2]);
        let hash_result = blake2b_simd::blake2b(&input).as_bytes();

        Self(
            hash_result[0..OVK_LEN_BYTES]
                .try_into()
                .expect("hash is long enough to convert to array"),
        )
    }
}

pub struct EphemeralPublicKey(pub decaf377::Element);

pub struct ProofAuthorizationKey {
    pub ak: AuthorizationKey,
    pub nsk: NullifierPrivateKey,
}

pub struct AuthorizationKey(pub decaf377::Element);

impl AuthorizationKey {
    #[allow(non_snake_case)]
    pub fn derive_public(ask: SpendAuthorizationKey) -> Self {
        let B = decaf377::Element::basepoint();
        AuthorizationKey(ask.0 * B)
    }
}

/// The `FullViewingKey` allows one to identify incoming and outgoing notes only.
pub struct FullViewingKey {
    pub ak: AuthorizationKey,
    pub nk: NullifierDerivingKey,
    pub ovk: OutgoingViewingKey,
}

pub struct NullifierDerivingKey(decaf377::Element);

// TODO: Add CompactFlagKey into the PaymentAddress
pub struct CompactFlagKey(pub decaf377::Element);

impl CompactDetectionKey {
    #[allow(non_snake_case)]
    pub fn derive_flag_key(&self) -> CompactFlagKey {
        let B = decaf377::Element::basepoint();
        CompactFlagKey(self.0 * B)
    }
}

/// Represents a diversified compact detection key.
pub struct CompactDetectionKey(Fr);

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

pub struct TransmissionKey(pub decaf377::Element);
