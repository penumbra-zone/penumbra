use std::convert::TryInto;

use ark_ff::PrimeField;
use decaf377;
use rand_core::{CryptoRng, RngCore};

use crate::Fr;

use super::{
    FullViewingKey, NullifierDerivingKey, NullifierPrivateKey, OutgoingViewingKey, ProofAuthKey,
    SigningKey, SpendAuth, OVK_LEN_BYTES,
};

pub const SPENDSEED_LEN_BYTES: usize = 32;

/// The root key material for a [`SpendKey`].
#[derive(Clone, Debug)]
pub struct SpendSeed(pub [u8; SPENDSEED_LEN_BYTES]);

/// A key representing a single spending authority.
pub struct SpendKey {
    seed: SpendSeed,
    ask: SigningKey<SpendAuth>,
    fvk: FullViewingKey,
    pak: ProofAuthKey,
}

impl SpendKey {
    /// Generates a new [`SpendKey`] from the supplied RNG.
    pub fn generate<R: RngCore + CryptoRng>(mut rng: R) -> Self {
        let mut seed_bytes = [0u8; SPENDSEED_LEN_BYTES];
        rng.fill_bytes(&mut seed_bytes);
        Self::from_seed(SpendSeed(seed_bytes))
    }

    /// Loads a [`SpendKey`] from the provided [`SpendSeed`].
    pub fn from_seed(seed: SpendSeed) -> Self {
        let ask = {
            let mut hasher = blake2b_simd::State::new();
            hasher.update(b"Penumbra_ExpandSeed");
            hasher.update(&seed.0);
            hasher.update(&[0; 1]);
            let hash_result = hasher.finalize();

            SigningKey::new_from_field(Fr::from_le_bytes_mod_order(hash_result.as_bytes()))
        };

        let nsk = {
            let mut hasher = blake2b_simd::State::new();
            hasher.update(b"Penumbra_ExpandSeed");
            hasher.update(&seed.0);
            hasher.update(&[1; 1]);
            let hash_result = hasher.finalize();

            NullifierPrivateKey(Fr::from_le_bytes_mod_order(hash_result.as_bytes()))
        };

        // XXX naming abbreviations don't match / but we might not even want this
        let pak = ProofAuthKey {
            nsk,
            ak: ask.into(),
        };

        let ovk = {
            let mut hasher = blake2b_simd::State::new();
            hasher.update(b"Penumbra_ExpandSeed");
            hasher.update(&seed.0);
            hasher.update(&[2; 1]);
            let hash_result = hasher.finalize();

            OutgoingViewingKey(
                hash_result.as_bytes()[0..OVK_LEN_BYTES]
                    .try_into()
                    .expect("hash is long enough to convert to array"),
            )
        };

        let fvk = FullViewingKey {
            ak: ask.into(),
            nk: NullifierDerivingKey(decaf377::basepoint() * &pak.nsk.0),
            ovk,
        };

        Self {
            seed,
            ask,
            pak,
            fvk,
        }
    }

    /// Get the [`SpendSeed`] this [`SpendKey`] was derived from.
    ///
    /// This is useful for serialization.
    pub fn seed(&self) -> &SpendSeed {
        &self.seed
    }

    // XXX how many of these do we need? leave them for now
    // but don't document until design is more settled

    pub fn spend_auth_key(&self) -> &SigningKey<SpendAuth> {
        &self.ask
    }

    pub fn nullifier_private_key(&self) -> &NullifierPrivateKey {
        &self.pak.nsk
    }

    pub fn outgoing_viewing_key(&self) -> &OutgoingViewingKey {
        &self.fvk.ovk
    }

    pub fn proof_authorization_key(&self) -> &ProofAuthKey {
        &self.pak
    }

    pub fn full_viewing_key(&self) -> &FullViewingKey {
        &self.fvk
    }
}
