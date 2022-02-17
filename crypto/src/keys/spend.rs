use std::convert::TryFrom;
use std::fmt;

use hmac::Hmac;
use pbkdf2::pbkdf2;
use rand_core::{CryptoRng, RngCore};
use serde::{Deserialize, Serialize};
use sha2::Digest;

use super::{
    seed_phrase::BIP39_WORDS, FullViewingKey, IncomingViewingKey, NullifierKey, OutgoingViewingKey,
};
use crate::{
    prf,
    rdsa::{SigningKey, SpendAuth},
};

pub const SEED_PHRASE_PBKDF2_ROUNDS: u32 = 2048;
pub const SEED_PHRASE_LEN: usize = 24;
pub const SEED_PHRASE_ENTROPY_BITS: usize = 256;
pub const SEED_PHRASE_CHECKSUM_BITS: usize = 8;
pub const SEED_PHRASE_BITS_PER_WORD: usize = 11;

/// A mnemonic seed phrase. Used to generate [`SpendSeed`]s.
pub struct SeedPhrase(pub [String; SEED_PHRASE_LEN]);

impl SeedPhrase {
    /// Randomly generates a BIP39 [`SeedPhrase`].
    pub fn generate<R: RngCore + CryptoRng>(mut rng: R) -> Self {
        // We get 256 bits of entropy.
        let mut randomness = [0u8; SEED_PHRASE_ENTROPY_BITS / 8];
        rng.fill_bytes(&mut randomness);
        Self::from_randomness(randomness)
    }

    /// Given 32 bytes, generate a [`SeedPhrase`].
    fn from_randomness(randomness: [u8; 32]) -> Self {
        // Convert to bits.
        const SEED_PHRASE_TOTAL_BITS: usize = SEED_PHRASE_ENTROPY_BITS + SEED_PHRASE_CHECKSUM_BITS;
        let mut bits = [false; SEED_PHRASE_TOTAL_BITS];

        // Add the random bits.
        for (i, bit) in bits[0..SEED_PHRASE_ENTROPY_BITS].iter_mut().enumerate() {
            *bit = (randomness[i / 8] & (1 << (7 - (i % 8)))) > 0
        }

        // We take the first 256/32 = 8 bits = 1 byte of the SHA256
        // hash of the randomness and treat it as a checksum, that we append
        // to the initial randomness.
        let mut hasher = sha2::Sha256::new();
        hasher.update(randomness);

        // Checksum is just the first byte of `r_hash`.
        let r_hash = hasher.finalize();
        for (i, bit) in bits[SEED_PHRASE_ENTROPY_BITS..].iter_mut().enumerate() {
            *bit = (r_hash[0] & (1 << (7 - (i % 8)))) > 0
        }

        // Concatenated bits are split into groups of 11 bits, each
        // encoding a number that is an index into the BIP39 word list.
        let mut phrases: [String; SEED_PHRASE_LEN] = Default::default();
        for (i, phrase) in phrases.iter_mut().enumerate() {
            let bits_this_word =
                &bits[i * SEED_PHRASE_BITS_PER_WORD..(i + 1) * SEED_PHRASE_BITS_PER_WORD];
            let index = bits_this_word
                .iter()
                .enumerate()
                .map(|(i, bit)| {
                    if *bit {
                        1 << (SEED_PHRASE_BITS_PER_WORD - 1 - i)
                    } else {
                        0
                    }
                })
                .sum::<usize>();
            *phrase = BIP39_WORDS[index].to_string();
        }
        SeedPhrase(phrases)
    }
}

impl fmt::Display for SeedPhrase {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (i, word) in self.0.iter().enumerate() {
            if i > 0 || i != SEED_PHRASE_LEN - 1 {
                f.write_str(" ")?;
            }
            f.write_str(word)?;
        }
        Ok(())
    }
}

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
