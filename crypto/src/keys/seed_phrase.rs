use std::fmt;

use rand_core::{CryptoRng, RngCore};
use sha2::Digest;

mod words;
use words::BIP39_WORDS;

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
