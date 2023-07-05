use std::fmt;

use rand_core::{CryptoRng, RngCore};
use sha2::Digest;

mod words;
use words::BIP39_WORDS;

pub const NUM_PBKDF2_ROUNDS: u32 = 2048;
pub const NUM_WORDS: usize = 24;
pub const NUM_ENTROPY_BITS: usize = 256;
pub const NUM_CHECKSUM_BITS: usize = 8;
pub const NUM_TOTAL_BITS: usize = NUM_ENTROPY_BITS + NUM_CHECKSUM_BITS;
pub const NUM_BITS_PER_WORD: usize = 11;
pub const NUM_BITS_PER_BYTE: usize = 8;

/// A mnemonic seed phrase. Used to generate [`SpendSeed`]s.
pub struct SeedPhrase(pub [String; NUM_WORDS]);

impl SeedPhrase {
    /// Randomly generates a BIP39 [`SeedPhrase`].
    pub fn generate<R: RngCore + CryptoRng>(mut rng: R) -> Self {
        let mut randomness = [0u8; NUM_ENTROPY_BITS / NUM_BITS_PER_BYTE];
        rng.fill_bytes(&mut randomness);
        Self::from_randomness(randomness)
    }

    /// Given 32 bytes of randomness, generate a [`SeedPhrase`].
    pub fn from_randomness(randomness: [u8; 32]) -> Self {
        let mut bits = [false; NUM_TOTAL_BITS];
        for (i, bit) in bits[0..NUM_ENTROPY_BITS].iter_mut().enumerate() {
            *bit = (randomness[i / NUM_BITS_PER_BYTE] & (1 << (7 - (i % NUM_BITS_PER_BYTE)))) > 0
        }

        // We take the first 256/32 = 8 bits of the SHA256 hash of the randomness and
        // treat it as a checksum. We append that checksum byte to the initial randomness.
        let mut hasher = sha2::Sha256::new();
        hasher.update(randomness);
        let checksum = hasher.finalize()[0];
        for (i, bit) in bits[NUM_ENTROPY_BITS..].iter_mut().enumerate() {
            *bit = (checksum & (1 << (7 - (i % NUM_BITS_PER_BYTE)))) > 0
        }

        // Concatenated bits are split into groups of 11 bits, each
        // encoding a number that is an index into the BIP39 word list.
        let mut words: [String; NUM_WORDS] = Default::default();
        for (i, word) in words.iter_mut().enumerate() {
            let bits_this_word = &bits[i * NUM_BITS_PER_WORD..(i + 1) * NUM_BITS_PER_WORD];
            let word_index = convert_bits_to_usize(bits_this_word);
            *word = BIP39_WORDS[word_index].to_string();
        }
        SeedPhrase(words)
    }

    /// Verify the checksum of this [`SeedPhrase`].
    fn verify_checksum(&self) -> Result<(), anyhow::Error> {
        let mut bits = [false; NUM_TOTAL_BITS];
        for (i, word) in self.0.iter().enumerate() {
            if !BIP39_WORDS.contains(&word.as_str()) {
                return Err(anyhow::anyhow!("invalid word in BIP39 seed phrase"));
            }

            let word_index = BIP39_WORDS.iter().position(|&x| x == word).unwrap();
            let word_bits = &mut bits[i * NUM_BITS_PER_WORD..(i + 1) * NUM_BITS_PER_WORD];
            word_bits
                .iter_mut()
                .enumerate()
                .for_each(|(j, bit)| *bit = (word_index >> (NUM_BITS_PER_WORD - 1 - j)) & 1 == 1);
        }

        let mut randomness = [0u8; NUM_ENTROPY_BITS / NUM_BITS_PER_BYTE];
        for (i, random_byte) in randomness.iter_mut().enumerate() {
            let bits_this_byte = &bits[i * NUM_BITS_PER_BYTE..(i + 1) * NUM_BITS_PER_BYTE];
            *random_byte = convert_bits_to_usize(bits_this_byte) as u8;
        }

        let checksum_bits = &bits[NUM_ENTROPY_BITS..];
        let checksum = convert_bits_to_usize(checksum_bits) as u8;

        let mut hasher = sha2::Sha256::new();
        hasher.update(randomness);
        if hasher.finalize()[0] != checksum {
            Err(anyhow::anyhow!("seed phrase checksum did not validate"))
        } else {
            Ok(())
        }
    }
}

impl fmt::Display for SeedPhrase {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (i, word) in self.0.iter().enumerate() {
            if i > 0 {
                f.write_str(" ")?;
            }
            f.write_str(word)?;
        }
        Ok(())
    }
}

impl std::str::FromStr for SeedPhrase {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let words = s
            .split_whitespace()
            .map(|w| w.to_lowercase())
            .collect::<Vec<String>>();

        if words.len() != NUM_WORDS {
            return Err(anyhow::anyhow!(
                "seed phrases should have {} words",
                NUM_WORDS
            ));
        }

        let seed_phrase = SeedPhrase(words.try_into().expect("can convert vec to arr"));
        seed_phrase.verify_checksum()?;

        Ok(seed_phrase)
    }
}

fn convert_bits_to_usize(bits: &[bool]) -> usize {
    bits.iter()
        .enumerate()
        .map(|(i, bit)| if *bit { 1 << (bits.len() - 1 - i) } else { 0 })
        .sum::<usize>()
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn bip39_mnemonic_derivation() {
        // These test vectors are taken from: https://github.com/trezor/python-mnemonic/blob/master/vectors.json
        let randomness_arr: [&str; 8] = [
            "0000000000000000000000000000000000000000000000000000000000000000",
            "7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f",
            "8080808080808080808080808080808080808080808080808080808080808080",
            "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
            "68a79eaca2324873eacc50cb9c6eca8cc68ea5d936f98787c60c7ebc74e6ce7c",
            "9f6a2878b2520799a44ef18bc7df394e7061a224d2c33cd015b157d746869863",
            "066dca1a2bb7e8a1db2832148ce9933eea0f3ac9548d793112d9a95c9407efad",
            "f585c11aec520db57dd353c69554b21a89b20fb0650966fa0a9d6f74fd989d8f",
        ];
        let expected_phrase_arr: [&str; 8] = [
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art",
            "legal winner thank year wave sausage worth useful legal winner thank year wave sausage worth useful legal winner thank year wave sausage worth title",
            "letter advice cage absurd amount doctor acoustic avoid letter advice cage absurd amount doctor acoustic avoid letter advice cage absurd amount doctor acoustic bless",
            "zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo vote",
            "hamster diagram private dutch cause delay private meat slide toddler razor book happy fancy gospel tennis maple dilemma loan word shrug inflict delay length",
            "panda eyebrow bullet gorilla call smoke muffin taste mesh discover soft ostrich alcohol speed nation flash devote level hobby quick inner drive ghost inside",
            "all hour make first leader extend hole alien behind guard gospel lava path output census museum junior mass reopen famous sing advance salt reform",
            "void come effort suffer camp survey warrior heavy shoot primary clutch crush open amazing screen patrol group space point ten exist slush involve unfold",
        ];

        for (hex_randomness, expected_phrase) in
            randomness_arr.iter().zip(expected_phrase_arr.iter())
        {
            let randomness = hex::decode(hex_randomness).expect("can decode test vector");
            let actual_phrase = SeedPhrase::from_randomness(randomness.clone().try_into().unwrap());
            assert_eq!(actual_phrase.to_string(), *expected_phrase);
        }
    }

    #[test]
    fn seed_phrase_from_str() {
        let invalid_phrases = [
            "too short",
            "zoo zoooooooo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo vote", // Invalid word
            "legal winner thank year wave sausage worth useful legal winner thank year wave sausage worth useful legal winner thank year wave sausage worth vote", // Invalid checksum
        ];
        for phrase in invalid_phrases {
            assert!(SeedPhrase::from_str(phrase).is_err());
        }

        let valid_phrases = [
            "zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo vote",
            "ZOO zoo ZOO zoo ZOO zoo ZOO zoo ZOO zoo ZOO zoo ZOO zoo ZOO zoo ZOO zoo ZOO zoo ZOO zoo ZOO VOTE",
            "legal winner thank year wave sausage worth useful legal winner thank year wave sausage worth useful legal winner thank year wave sausage worth title"
        ];
        for phrase in valid_phrases {
            assert!(SeedPhrase::from_str(phrase).is_ok());
        }
    }
}
