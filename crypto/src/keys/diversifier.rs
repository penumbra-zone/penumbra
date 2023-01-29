use std::convert::{TryFrom, TryInto};
use std::fmt::Debug;

use aes::cipher::{generic_array::GenericArray, BlockDecrypt, BlockEncrypt, KeyInit};
use aes::Aes128;

use anyhow::anyhow;
use ark_ff::PrimeField;
use derivative::Derivative;
use penumbra_proto::{core::crypto::v1alpha1 as pb, DomainType};
use serde::{Deserialize, Serialize};

use crate::Fq;

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
            return Err(anyhow!(
                "diversifier must be 16 bytes, got {:?}",
                slice.len()
            ));
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
#[derivative(Debug)]
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
        if index_bytes[8..16] == [0u8; 8] {
            AddressIndex::Numeric(u64::from_le_bytes(
                index_bytes[0..8].try_into().expect("can form 8 byte array"),
            ))
        } else {
            AddressIndex::Random(index_bytes)
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Derivative, Serialize, Deserialize)]
#[serde(try_from = "pb::AddressIndex", into = "pb::AddressIndex")]
pub enum AddressIndex {
    /// Reserved for client applications.
    Numeric(u64),
    /// Randomly generated.
    Random([u8; 16]),
}

impl Default for AddressIndex {
    fn default() -> Self {
        AddressIndex::Numeric(0)
    }
}

// Workaround for https://github.com/mcarton/rust-derivative/issues/91
impl Debug for AddressIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        crate::fmt_hex(self.to_bytes(), f)
    }
}

impl AddressIndex {
    pub fn to_bytes(&self) -> [u8; 16] {
        match self {
            Self::Numeric(x) => {
                let mut bytes = [0; DIVERSIFIER_LEN_BYTES];
                bytes[0..8].copy_from_slice(&x.to_le_bytes());
                bytes
            }
            Self::Random(bytes) => bytes.clone(),
        }
    }
}

impl From<u8> for AddressIndex {
    fn from(x: u8) -> Self {
        AddressIndex::Numeric(x as u64)
    }
}

impl From<u16> for AddressIndex {
    fn from(x: u16) -> Self {
        AddressIndex::Numeric(x as u64)
    }
}

impl From<u32> for AddressIndex {
    fn from(x: u32) -> Self {
        AddressIndex::Numeric(x as u64)
    }
}

impl From<u64> for AddressIndex {
    fn from(x: u64) -> Self {
        AddressIndex::Numeric(x as u64)
    }
}

impl From<usize> for AddressIndex {
    fn from(x: usize) -> Self {
        AddressIndex::Numeric(x as u64)
    }
}

impl From<AddressIndex> for u128 {
    fn from(x: AddressIndex) -> Self {
        match x {
            AddressIndex::Numeric(x) => u128::from(x),
            AddressIndex::Random(x) => u128::from_le_bytes(x),
        }
    }
}

impl TryFrom<AddressIndex> for u64 {
    type Error = anyhow::Error;
    fn try_from(address_index: AddressIndex) -> Result<Self, Self::Error> {
        match address_index {
            AddressIndex::Numeric(x) => Ok(x),
            AddressIndex::Random(_) => Err(anyhow::anyhow!(
                "address index {:?} is not AddressIndex::Numeric",
                address_index
            )),
        }
    }
}

impl TryFrom<&[u8]> for AddressIndex {
    type Error = anyhow::Error;

    fn try_from(slice: &[u8]) -> Result<AddressIndex, Self::Error> {
        if slice.len() != DIVERSIFIER_LEN_BYTES {
            return Err(anyhow!(
                "address index must be 16 bytes, got {:?}",
                slice.len()
            ));
        }

        // Numeric addresses have the last eight bytes as zero.
        if slice[8..16] == [0u8; 8] {
            let mut bytes = [0; 8];
            bytes[0..8].copy_from_slice(&slice[0..8]);
            Ok(AddressIndex::Numeric(u64::from_le_bytes(bytes)))
        } else {
            let mut bytes = [0u8; DIVERSIFIER_LEN_BYTES];
            bytes.copy_from_slice(&slice[0..16]);
            Ok(AddressIndex::Random(bytes))
        }
    }
}

impl DomainType for AddressIndex {
    type Proto = pb::AddressIndex;
}

impl From<AddressIndex> for pb::AddressIndex {
    fn from(d: AddressIndex) -> pb::AddressIndex {
        match d {
            AddressIndex::Numeric(x) => {
                let mut bytes = [0; DIVERSIFIER_LEN_BYTES];
                bytes[0..8].copy_from_slice(&x.to_le_bytes());
                pb::AddressIndex {
                    inner: bytes.to_vec(),
                }
            }
            AddressIndex::Random(x) => pb::AddressIndex { inner: x.to_vec() },
        }
    }
}

impl TryFrom<pb::AddressIndex> for AddressIndex {
    type Error = anyhow::Error;

    fn try_from(d: pb::AddressIndex) -> Result<AddressIndex, Self::Error> {
        d.inner.as_slice().try_into()
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    fn address_index_strategy_numeric() -> BoxedStrategy<AddressIndex> {
        any::<u64>().prop_map(AddressIndex::Numeric).boxed()
    }

    fn address_index_strategy_random() -> BoxedStrategy<AddressIndex> {
        any::<[u8; 16]>().prop_map(AddressIndex::Random).boxed()
    }

    fn diversifier_key_strategy() -> BoxedStrategy<DiversifierKey> {
        any::<[u8; 16]>().prop_map(DiversifierKey).boxed()
    }

    proptest! {
        #[test]
        fn diversifier_encryption_roundtrip(
            key in diversifier_key_strategy(),
            index in address_index_strategy_numeric(),
        ) {
            let diversifier = key.diversifier_for_index(&index);
            let index2 = key.index_for_diversifier(&diversifier);
            assert_eq!(index2, index);
        }

        #[test]
        fn diversifier_encryption_roundtrip_numeric(
            key in diversifier_key_strategy(),
            index in address_index_strategy_random(),
        ) {
            let diversifier = key.diversifier_for_index(&index);
            let index2 = key.index_for_diversifier(&diversifier);
            assert_eq!(index2, index );
        }
    }
}
