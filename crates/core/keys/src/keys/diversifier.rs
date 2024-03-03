use std::convert::{TryFrom, TryInto};
use std::fmt::Debug;
use std::str::FromStr;

use aes::cipher::{generic_array::GenericArray, BlockDecrypt, BlockEncrypt, KeyInit};
use aes::Aes128;

use anyhow::Context;
use derivative::Derivative;
use penumbra_proto::{penumbra::core::keys::v1 as pb, DomainType};
use rand::{CryptoRng, RngCore};
use serde::{Deserialize, Serialize};

use decaf377::Fq;

pub const DIVERSIFIER_LEN_BYTES: usize = 16;

#[derive(Copy, Clone, PartialEq, Eq, Derivative, Serialize, Deserialize)]
#[derivative(Debug)]
#[serde(try_from = "pb::Diversifier", into = "pb::Diversifier")]
pub struct Diversifier(
    #[derivative(Debug(bound = "", format_with = "crate::fmt_hex"))] pub [u8; DIVERSIFIER_LEN_BYTES],
);

impl Diversifier {
    /// Generate the diversified basepoint associated to this diversifier.
    pub fn diversified_generator(&self) -> decaf377::Element {
        let hash = blake2b_simd::Params::new()
            .personal(b"Penumbra_Divrsfy")
            .hash(&self.0);

        decaf377::Element::encode_to_curve(&Fq::from_le_bytes_mod_order(hash.as_bytes()))
    }
}

impl AsRef<[u8; DIVERSIFIER_LEN_BYTES]> for Diversifier {
    fn as_ref(&self) -> &[u8; DIVERSIFIER_LEN_BYTES] {
        &self.0
    }
}

impl TryFrom<&[u8]> for Diversifier {
    type Error = anyhow::Error;

    fn try_from(slice: &[u8]) -> Result<Diversifier, Self::Error> {
        if slice.len() != DIVERSIFIER_LEN_BYTES {
            anyhow::bail!("diversifier must be 16 bytes, got {:?}", slice.len());
        }

        let mut bytes = [0u8; DIVERSIFIER_LEN_BYTES];
        bytes.copy_from_slice(&slice[0..16]);
        Ok(Diversifier(bytes))
    }
}

impl DomainType for Diversifier {
    type Proto = pb::Diversifier;
}

impl From<Diversifier> for pb::Diversifier {
    fn from(d: Diversifier) -> pb::Diversifier {
        pb::Diversifier {
            inner: d.as_ref().to_vec(),
        }
    }
}

impl TryFrom<pb::Diversifier> for Diversifier {
    type Error = anyhow::Error;

    fn try_from(d: pb::Diversifier) -> Result<Diversifier, Self::Error> {
        d.inner.as_slice().try_into()
    }
}

#[derive(Clone, Derivative)]
#[derivative(Debug, PartialEq, Eq)]
pub struct DiversifierKey(
    #[derivative(Debug(bound = "", format_with = "crate::fmt_hex"))] pub(super) [u8; 16],
);

impl DiversifierKey {
    pub fn diversifier_for_index(&self, index: &AddressIndex) -> Diversifier {
        let mut key_bytes = [0u8; 16];
        key_bytes.copy_from_slice(&self.0);
        let key = GenericArray::from(key_bytes);

        let mut plaintext_bytes = [0u8; 16];
        plaintext_bytes.copy_from_slice(&index.to_bytes());
        let mut block = GenericArray::from(plaintext_bytes);

        let cipher = Aes128::new(&key);
        cipher.encrypt_block(&mut block);

        let mut ciphertext_bytes = [0u8; 16];
        ciphertext_bytes.copy_from_slice(&block);
        Diversifier(ciphertext_bytes)
    }

    pub fn index_for_diversifier(&self, diversifier: &Diversifier) -> AddressIndex {
        let mut key_bytes = [0u8; 16];
        key_bytes.copy_from_slice(&self.0);
        let key = GenericArray::from(key_bytes);

        let mut block = GenericArray::from(diversifier.0);

        let cipher = Aes128::new(&key);
        cipher.decrypt_block(&mut block);

        let mut index_bytes = [0; DIVERSIFIER_LEN_BYTES];
        index_bytes.copy_from_slice(&block);

        AddressIndex {
            account: u32::from_le_bytes(
                index_bytes[0..4].try_into().expect("can form 4 byte array"),
            ),
            randomizer: index_bytes[4..16]
                .try_into()
                .expect("can form 12 byte array"),
        }
    }
}

#[derive(
    Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Derivative, Serialize, Deserialize,
)]
#[serde(try_from = "pb::AddressIndex", into = "pb::AddressIndex")]
pub struct AddressIndex {
    pub account: u32,
    pub randomizer: [u8; 12],
}

impl Debug for AddressIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AddressIndex")
            .field("account", &self.account)
            .field("randomizer", &hex::encode(self.randomizer))
            .finish()
    }
}

impl AddressIndex {
    pub fn to_bytes(&self) -> [u8; 16] {
        let mut bytes = [0; DIVERSIFIER_LEN_BYTES];
        bytes[0..4].copy_from_slice(&self.account.to_le_bytes());
        bytes[4..16].copy_from_slice(&self.randomizer);
        bytes
    }

    pub fn is_ephemeral(&self) -> bool {
        self.randomizer != [0; 12]
    }

    pub fn new(account: u32) -> Self {
        AddressIndex::from(account)
    }

    pub fn new_ephemeral<R: RngCore + CryptoRng>(account: u32, mut rng: R) -> Self {
        let mut bytes = [0u8; 12];

        rng.fill_bytes(&mut bytes);

        Self {
            account,
            randomizer: bytes,
        }
    }
}

impl From<u32> for AddressIndex {
    fn from(x: u32) -> Self {
        Self {
            account: x,
            randomizer: [0; 12],
        }
    }
}

// TODO: add support for ephemeral addresses to FromStr impl.
impl FromStr for AddressIndex {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self> {
        let i: u32 = s.parse()?;
        Ok(Self {
            account: i,
            randomizer: [0; 12],
        })
    }
}

impl From<AddressIndex> for u128 {
    fn from(x: AddressIndex) -> Self {
        u128::from_le_bytes(x.to_bytes())
    }
}

impl TryFrom<AddressIndex> for u64 {
    type Error = anyhow::Error;
    fn try_from(address_index: AddressIndex) -> Result<Self, Self::Error> {
        let mut bytes = [0; 8];
        bytes[0..4].copy_from_slice(&address_index.account.to_le_bytes());
        bytes[5..8].copy_from_slice(address_index.randomizer.as_slice());

        Ok(u64::from_le_bytes(bytes))
    }
}

impl TryFrom<&[u8]> for AddressIndex {
    type Error = anyhow::Error;

    fn try_from(slice: &[u8]) -> Result<AddressIndex, Self::Error> {
        if slice.len() != DIVERSIFIER_LEN_BYTES {
            anyhow::bail!("address index must be 16 bytes, got {:?}", slice.len());
        }

        Ok(AddressIndex {
            account: u32::from_le_bytes(slice[0..4].try_into().expect("can form 4 byte array")),
            randomizer: slice[4..16].try_into().expect("can form 12 byte array"),
        })
    }
}

impl DomainType for AddressIndex {
    type Proto = pb::AddressIndex;
}

impl From<AddressIndex> for pb::AddressIndex {
    fn from(d: AddressIndex) -> pb::AddressIndex {
        if d.is_ephemeral() {
            pb::AddressIndex {
                account: d.account,
                randomizer: d.randomizer.to_vec(),
            }
        } else {
            pb::AddressIndex {
                account: d.account,
                randomizer: Vec::new(),
            }
        }
    }
}

impl TryFrom<pb::AddressIndex> for AddressIndex {
    type Error = anyhow::Error;

    fn try_from(d: pb::AddressIndex) -> Result<AddressIndex, Self::Error> {
        let randomizer: [u8; 12] = if d.randomizer.is_empty() {
            [0; 12]
        } else {
            d.randomizer
                .as_slice()
                .try_into()
                .context("could not parse 12-byte array")?
        };

        Ok(Self {
            account: d.account,
            randomizer,
        })
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    fn address_index_strategy() -> BoxedStrategy<AddressIndex> {
        any::<u32>().prop_map(AddressIndex::from).boxed()
    }

    fn diversifier_key_strategy() -> BoxedStrategy<DiversifierKey> {
        any::<[u8; 16]>().prop_map(DiversifierKey).boxed()
    }

    proptest! {
        #[test]
        fn diversifier_encryption_roundtrip(
            key in diversifier_key_strategy(),
            index in address_index_strategy(),
        ) {
            let diversifier = key.diversifier_for_index(&index);
            let index2 = key.index_for_diversifier(&diversifier);
            assert_eq!(index2, index);
        }
    }
}
