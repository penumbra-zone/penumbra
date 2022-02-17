use std::convert::TryFrom;

use hmac::Hmac;
use pbkdf2::pbkdf2;
use serde::{Deserialize, Serialize};

use super::{
    seed_phrase::{SeedPhrase, SEED_PHRASE_PBKDF2_ROUNDS},
    FullViewingKey, IncomingViewingKey, NullifierKey, OutgoingViewingKey,
};
use crate::{
    prf,
    rdsa::{SigningKey, SpendAuth},
};

pub const SPENDSEED_LEN_BYTES: usize = 32;

/// The root key material for a [`SpendKey`].
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SpendSeed(pub [u8; SPENDSEED_LEN_BYTES]);

impl SpendSeed {
    /// Deterministically generate a [`SpendSeed`] from a [`SeedPhrase`].
    ///
    /// The choice of KDF (PBKDF2), iteration count, and PRF (HMAC-SHA512) are specified
    /// in [`BIP39`]. The salt is specified in BIP39 as the string "mnemonic" plus an optional
    /// passphrase, which we set to an index. This allows us to derive multiple spend
    /// authorities from a single seed phrase.
    ///
    /// [`BIP39`]: https://github.com/bitcoin/bips/blob/master/bip-0039.mediawiki
    pub fn from_seed_phrase(seed_phrase: SeedPhrase, index: u64) -> Self {
        let password = format!("{}", seed_phrase);
        let salt = format!("mnemonic{}", index);
        let mut spend_seed_bytes = [0u8; 32];
        pbkdf2::<Hmac<sha2::Sha512>>(
            password.as_bytes(),
            salt.as_bytes(),
            SEED_PHRASE_PBKDF2_ROUNDS,
            &mut spend_seed_bytes,
        );
        SpendSeed(spend_seed_bytes)
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
