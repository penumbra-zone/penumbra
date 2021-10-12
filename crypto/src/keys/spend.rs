use std::convert::{TryFrom, TryInto};

use ark_ff::PrimeField;
use decaf377::{self, FrExt};
use rand_core::{CryptoRng, RngCore};

use crate::Fr;

use super::{
    FullViewingKey, NullifierDerivingKey, NullifierPrivateKey, OutgoingViewingKey,
    ProofAuthorizationKey, SigningKey, SpendAuth,
};

pub const SPEND_LEN_BYTES: usize = 32;

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

    pub fn derive_proof_authorization_key(&self) -> ProofAuthorizationKey {
        ProofAuthorizationKey {
            ak: self.ask.into(),
            nsk: self.nsk,
        }
    }

    pub fn derive_full_viewing_key(&self) -> FullViewingKey {
        FullViewingKey {
            ak: self.ask.into(),
            nk: NullifierDerivingKey::derive(&self.nsk),
            ovk: self.ovk,
        }
    }
}
