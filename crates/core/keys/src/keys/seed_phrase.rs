use std::fmt;

use rand_core::{CryptoRng, RngCore};
use sha2::Digest;

mod words;
use words::BIP39_WORDS;

pub const NUM_PBKDF2_ROUNDS: u32 = 2048;
pub const NUM_WORDS_SHORT: usize = 12;
pub const NUM_WORDS_LONG: usize = 24;

pub const NUM_ENTROPY_BITS_SHORT: usize = 128;
pub const NUM_ENTROPY_BITS_LONG: usize = 256;
pub const NUM_BITS_PER_WORD: usize = 11;
pub const NUM_BITS_PER_BYTE: usize = 8;

/// Allowed seed phrases for a [`SeedPhrase`]. We support either
/// the minimum or maximum length.
pub enum SeedPhraseType {
    /// The minimum length for a BIP39 mnenomic seed phrase is 12 words.
    MinimumLength,
    /// The maximum length for a BIP39 mnenomic seed phrase is 24 words.
    MaximumLength,
}

impl SeedPhraseType {
    pub fn from_length(len: usize) -> anyhow::Result<Self> {
        match len {
            NUM_WORDS_SHORT => Ok(SeedPhraseType::MinimumLength),
            NUM_WORDS_LONG => Ok(SeedPhraseType::MaximumLength),
            _ => Err(anyhow::anyhow!("invalid seed phrase length")),
        }
    }

    pub fn from_randomness_length(len: usize) -> anyhow::Result<Self> {
        match len * NUM_BITS_PER_BYTE {
            NUM_ENTROPY_BITS_SHORT => Ok(SeedPhraseType::MinimumLength),
            NUM_ENTROPY_BITS_LONG => Ok(SeedPhraseType::MaximumLength),
            _ => Err(anyhow::anyhow!("invalid randomness length")),
        }
    }

    pub fn num_words(&self) -> usize {
        match self {
            SeedPhraseType::MinimumLength => NUM_WORDS_SHORT,
            SeedPhraseType::MaximumLength => NUM_WORDS_LONG,
        }
    }

    pub fn num_checksum_bits(&self) -> usize {
        match self {
            SeedPhraseType::MinimumLength => NUM_ENTROPY_BITS_SHORT / 32,
            SeedPhraseType::MaximumLength => NUM_ENTROPY_BITS_LONG / 32,
        }
    }

    pub fn num_entropy_bits(&self) -> usize {
        match self {
            SeedPhraseType::MinimumLength => NUM_ENTROPY_BITS_SHORT,
            SeedPhraseType::MaximumLength => NUM_ENTROPY_BITS_LONG,
        }
    }

    pub fn num_total_bits(&self) -> usize {
        self.num_entropy_bits() + self.num_checksum_bits()
    }
}

/// A mnemonic seed phrase. Used to generate [`SpendSeed`]s.
pub struct SeedPhrase(pub Vec<String>);

impl SeedPhrase {
    /// Randomly generates a 24 word BIP39 [`SeedPhrase`].
    pub fn generate<R: RngCore + CryptoRng>(mut rng: R) -> Self {
        let mut randomness = [0u8; NUM_ENTROPY_BITS_LONG / NUM_BITS_PER_BYTE];
        rng.fill_bytes(&mut randomness);
        Self::from_randomness(&randomness)
    }

    /// Randomly generates a 12 word BIP39 [`SeedPhrase`].
    pub fn short_generate<R: RngCore + CryptoRng>(mut rng: R) -> Self {
        let mut randomness = [0u8; NUM_ENTROPY_BITS_SHORT / NUM_BITS_PER_BYTE];
        rng.fill_bytes(&mut randomness);
        Self::from_randomness(&randomness)
    }

    /// Given bytes of randomness, generate a [`SeedPhrase`].
    pub fn from_randomness(randomness: &[u8]) -> Self {
        // We infer if the seed phrase will be a valid length based on the number of
        // random bytes generated.
        let seed_phrase_type = SeedPhraseType::from_randomness_length(randomness.len())
            .expect("can get seed phrase type from randomness length");

        let num_entropy_bits = seed_phrase_type.num_entropy_bits();
        let mut bits = vec![false; seed_phrase_type.num_total_bits()];
        for (i, bit) in bits[0..num_entropy_bits].iter_mut().enumerate() {
            *bit = (randomness[i / NUM_BITS_PER_BYTE] & (1 << (7 - (i % NUM_BITS_PER_BYTE)))) > 0
        }

        // We take the first (entropy length in bits)/32 of the SHA256 hash of the randomness and
        // treat it as a checksum. We append that checksum byte to the initial randomness.
        //
        // For 24-word seed phrases, the entropy length in bits is 256 (= 32 * 8), so the checksum
        // is 8 bits. For 12-word seed phrases, the entropy length in bits is 128 (= 16 * 8), so the
        // checksum is 4 bits.
        let mut hasher = sha2::Sha256::new();
        hasher.update(randomness);
        let checksum = hasher.finalize()[0];
        for (i, bit) in bits[num_entropy_bits..].iter_mut().enumerate() {
            *bit = (checksum & (1 << (7 - (i % NUM_BITS_PER_BYTE)))) > 0
        }

        // Concatenated bits are split into groups of 11 bits, each
        // encoding a number that is an index into the BIP39 word list.
        let mut words = vec![String::new(); seed_phrase_type.num_words()];
        for (i, word) in words.iter_mut().enumerate() {
            let bits_this_word = &bits[i * NUM_BITS_PER_WORD..(i + 1) * NUM_BITS_PER_WORD];
            let word_index = convert_bits_to_usize(bits_this_word);
            *word = BIP39_WORDS[word_index].to_string();
        }
        SeedPhrase(words)
    }

    /// Number of words in this [`SeedPhrase`].
    pub fn length(&self) -> usize {
        self.0.len()
    }

    /// Verify the checksum of this [`SeedPhrase`].
    fn verify_checksum(&self) -> anyhow::Result<()> {
        let seed_phrase_type = SeedPhraseType::from_length(self.length())?;
        let mut bits = vec![false; seed_phrase_type.num_total_bits()];
        for (i, word) in self.0.iter().enumerate() {
            if !BIP39_WORDS.contains(&word.as_str()) {
                anyhow::bail!("invalid word in BIP39 seed phrase");
            }

            let word_index = BIP39_WORDS
                .iter()
                .position(|&x| x == word)
                .expect("can get index of word");
            let word_bits = &mut bits[i * NUM_BITS_PER_WORD..(i + 1) * NUM_BITS_PER_WORD];
            word_bits
                .iter_mut()
                .enumerate()
                .for_each(|(j, bit)| *bit = (word_index >> (NUM_BITS_PER_WORD - 1 - j)) & 1 == 1);
        }

        let mut randomness = vec![0u8; seed_phrase_type.num_entropy_bits() / NUM_BITS_PER_BYTE];
        for (i, random_byte) in randomness.iter_mut().enumerate() {
            let bits_this_byte = &bits[i * NUM_BITS_PER_BYTE..(i + 1) * NUM_BITS_PER_BYTE];
            *random_byte = convert_bits_to_usize(bits_this_byte) as u8;
        }

        let mut hasher = sha2::Sha256::new();
        hasher.update(randomness);
        let calculated_checksum = hasher.finalize()[0];

        let mut calculated_checksum_bits = vec![false; seed_phrase_type.num_checksum_bits()];
        for (i, bit) in calculated_checksum_bits.iter_mut().enumerate() {
            *bit = (calculated_checksum & (1 << (7 - (i % NUM_BITS_PER_BYTE)))) > 0
        }

        let checksum_bits = &bits[seed_phrase_type.num_entropy_bits()..];
        for (expected_bit, checksum_bit) in checksum_bits.iter().zip(calculated_checksum_bits) {
            if checksum_bit != *expected_bit {
                return Err(anyhow::anyhow!("seed phrase checksum did not validate"));
            }
        }
        Ok(())
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

        if words.len() != NUM_WORDS_LONG && words.len() != NUM_WORDS_SHORT {
            anyhow::bail!(
                "seed phrases should have {} or {} words",
                NUM_WORDS_LONG,
                NUM_WORDS_SHORT
            );
        }

        let seed_phrase = SeedPhrase(words);
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
        let randomness_arr: [&str; 16] = [
            "00000000000000000000000000000000",
            "7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f",
            "80808080808080808080808080808080",
            "ffffffffffffffffffffffffffffffff",
            "9e885d952ad362caeb4efe34a8e91bd2",
            "c0ba5a8e914111210f2bd131f3d5e08d",
            "23db8160a31d3e0dca3688ed941adbf3",
            "f30f8c1da665478f49b001d94c5fc452",
            "0000000000000000000000000000000000000000000000000000000000000000",
            "7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f",
            "8080808080808080808080808080808080808080808080808080808080808080",
            "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
            "68a79eaca2324873eacc50cb9c6eca8cc68ea5d936f98787c60c7ebc74e6ce7c",
            "9f6a2878b2520799a44ef18bc7df394e7061a224d2c33cd015b157d746869863",
            "066dca1a2bb7e8a1db2832148ce9933eea0f3ac9548d793112d9a95c9407efad",
            "f585c11aec520db57dd353c69554b21a89b20fb0650966fa0a9d6f74fd989d8f",
        ];
        let expected_phrase_arr: [&str; 16] = [
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
            "legal winner thank year wave sausage worth useful legal winner thank yellow",
            "letter advice cage absurd amount doctor acoustic avoid letter advice cage above",
            "zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo wrong",
            "ozone drill grab fiber curtain grace pudding thank cruise elder eight picnic",
            "scheme spot photo card baby mountain device kick cradle pact join borrow",
            "cat swing flag economy stadium alone churn speed unique patch report train",
            "vessel ladder alter error federal sibling chat ability sun glass valve picture",
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
            let actual_phrase = SeedPhrase::from_randomness(&randomness[..]);
            assert_eq!(actual_phrase.to_string(), *expected_phrase);
            actual_phrase
                .verify_checksum()
                .expect("checksum should validate");
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
