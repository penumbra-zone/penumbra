use ark_ff::PrimeField;
use rand_core::{CryptoRng, RngCore};

use crate::{
    rdsa::{SigningKey, SpendAuth},
    Fq, Fr,
};

use super::{FullViewingKey, IncomingViewingKey, NullifierKey, OutgoingViewingKey};

pub const SPENDSEED_LEN_BYTES: usize = 32;

/// The root key material for a [`SpendKey`].
#[derive(Clone, Debug)]
pub struct SpendSeed(pub [u8; SPENDSEED_LEN_BYTES]);

/// A key representing a single spending authority.
pub struct SpendKey {
    seed: SpendSeed,
    ask: SigningKey<SpendAuth>,
    fvk: FullViewingKey,
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
            hasher.update(b"Penumbra_ExpndSd");
            hasher.update(&seed.0);
            hasher.update(&[0; 1]);
            let hash_result = hasher.finalize();

            SigningKey::new_from_field(Fr::from_le_bytes_mod_order(hash_result.as_bytes()))
        };

        let nk = {
            let mut hasher = blake2b_simd::State::new();
            hasher.update(b"Penumbra_ExpndSd");
            hasher.update(&seed.0);
            hasher.update(&[1; 1]);
            let hash_result = hasher.finalize();

            NullifierKey(Fq::from_le_bytes_mod_order(hash_result.as_bytes()))
        };

        let fvk = FullViewingKey::from_components(ask.into(), nk);

        Self { seed, ask, fvk }
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

    pub fn full_viewing_key(&self) -> &FullViewingKey {
        &self.fvk
    }

    pub fn nullifier_key(&self) -> &NullifierKey {
        self.fvk.nullifier_key()
    }

    pub fn outgoing_viewing_key(&self) -> &OutgoingViewingKey {
        self.fvk.outgoing()
    }

    pub fn incoming_viewing_key(&self) -> &IncomingViewingKey {
        self.fvk.incoming()
    }
}
