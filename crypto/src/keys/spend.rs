use std::convert::TryFrom;

use rand_core::{CryptoRng, RngCore};
use serde::{Deserialize, Serialize};

use crate::{
    prf,
    rdsa::{SigningKey, SpendAuth},
};

use super::{FullViewingKey, IncomingViewingKey, NullifierKey, OutgoingViewingKey};

pub const SPENDSEED_LEN_BYTES: usize = 32;

/// The root key material for a [`SpendKey`].
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SpendSeed(pub [u8; SPENDSEED_LEN_BYTES]);

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
    /// Generates a new [`SpendKey`] from the supplied RNG.
    pub fn generate<R: RngCore + CryptoRng>(mut rng: R) -> Self {
        let mut seed_bytes = [0u8; SPENDSEED_LEN_BYTES];
        rng.fill_bytes(&mut seed_bytes);
        Self::from(SpendSeed(seed_bytes))
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
