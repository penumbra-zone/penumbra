use std::convert::TryFrom;

use rand::seq::SliceRandom;
use rand_core::{CryptoRng, RngCore};
use serde::{Deserialize, Serialize};

use super::{
    seed_phrase::BIP39_WORDS, FullViewingKey, IncomingViewingKey, NullifierKey, OutgoingViewingKey,
};
use crate::{
    prf,
    rdsa::{SigningKey, SpendAuth},
};

pub const SEED_PHRASE_LEN: usize = 24;

/// A mnemonic seed phrase. Used to generate [`SpendSeed`]s.
pub struct SeedPhrase(pub [String; SEED_PHRASE_LEN]);

impl SeedPhrase {
    /// Randomly generates a [`SeedPhrase`].
    pub fn generate<R: RngCore + CryptoRng>(mut rng: R) -> Self {
        let mut phrases: [String; SEED_PHRASE_LEN] = Default::default();
        for phrase in phrases.iter_mut() {
            *phrase = BIP39_WORDS.choose(&mut rng).unwrap().to_string();
        }
        SeedPhrase(phrases)
    }
}

pub const SPENDSEED_LEN_BYTES: usize = 32;

/// The root key material for a [`SpendKey`].
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SpendSeed(pub [u8; SPENDSEED_LEN_BYTES]);

impl SpendSeed {
    /// Deterministically generate a [`SpendSeed`] from a [`SeedPhrase`].
    pub fn from_seed_phrase(seed_phrase: SeedPhrase, index: u64) -> Self {
        todo!()
    }
}

/// A key representing a single spending authority.
#[derive(Debug, Clone)]
pub struct SpendKey {
    seed: SpendSeed,
    ask: SigningKey<SpendAuth>,
    fvk: FullViewingKey,
}

impl From<SpendSeed> for SpendKey {
    fn from(seed: SpendSeed) -> Self {
        let ask = SigningKey::new_from_field(prf::expand_ff(b"Penumbra_ExpndSd", &seed.0, &[0; 1]));
        let nk = NullifierKey(prf::expand_ff(b"Penumbra_ExpndSd", &seed.0, &[1; 1]));
        let fvk = FullViewingKey::from_components(ask.into(), nk);

        Self { seed, ask, fvk }
    }
}

impl SpendKey {
    /// Create a [`SpendKey`] from a [`SpendSeed`].
    pub fn new(seed: SpendSeed) -> Self {
        Self::from(seed)
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

impl TryFrom<&[u8]> for SpendSeed {
    type Error = anyhow::Error;
    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        if slice.len() != SPENDSEED_LEN_BYTES {
            return Err(anyhow::anyhow!(
                "spendseed must be 32 bytes, got {:?}",
                slice.len()
            ));
        }

        let mut bytes = [0u8; SPENDSEED_LEN_BYTES];
        bytes.copy_from_slice(&slice[0..32]);
        Ok(SpendSeed(bytes))
    }
}
