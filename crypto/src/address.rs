use std::io::{Cursor, Read, Write};

use ark_serialize::CanonicalDeserialize;
use f4jumble::{f4jumble, f4jumble_inv};
use penumbra_proto::{crypto as pb, serializers::bech32str, Protobuf};
use rand::{CryptoRng, Rng};
use serde::{Deserialize, Serialize};

use crate::{fmd, ka, keys::Diversifier, Fq};

pub const ADDRESS_LEN_BYTES: usize = 80;
/// Number of bits in the address short form divided by the number of bits per Bech32m character
pub const ADDRESS_NUM_CHARS_SHORT_FORM: usize = 24;

/// A valid payment address.
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "pb::Address", into = "pb::Address")]
pub struct Address {
    d: Diversifier,
    /// cached copy of the diversified base
    g_d: decaf377::Element,

    /// extra invariant: the bytes in pk_d should be the canonical encoding of an
    /// s value (whether or not it is a valid decaf377 encoding)
    /// this ensures we can use a PaymentAddress to form a note commitment,
    /// which involves hashing s as a field element.
    pk_d: ka::Public,
    /// transmission key s value
    transmission_key_s: Fq,

    ck_d: fmd::ClueKey,
}

impl Address {
    /// Constructs a payment address from its components.
    ///
    /// Returns `None` if the bytes in pk_d are a non-canonical representation
    /// of an [`Fq`] `s` value.
    pub(crate) fn from_components(
        d: Diversifier,
        pk_d: ka::Public,
        ck_d: fmd::ClueKey,
    ) -> Option<Self> {
        // XXX ugly -- better way to get our hands on the s value?
        // add to decaf377::Encoding? there's compress_to_field already...
        if let Ok(transmission_key_s) = Fq::deserialize(&pk_d.0[..]) {
            // don't need an error type here, caller will probably .expect anyways
            Some(Self {
                d,
                g_d: d.diversified_generator(),
                pk_d,
                ck_d,
                transmission_key_s,
            })
        } else {
            None
        }
    }

    pub fn diversifier(&self) -> &Diversifier {
        &self.d
    }

    pub fn diversified_generator(&self) -> &decaf377::Element {
        &self.g_d
    }

    pub fn transmission_key(&self) -> &ka::Public {
        &self.pk_d
    }

    pub fn clue_key(&self) -> &fmd::ClueKey {
        &self.ck_d
    }

    pub fn transmission_key_s(&self) -> &Fq {
        &self.transmission_key_s
    }

    pub fn to_vec(&self) -> Vec<u8> {
        let mut bytes = std::io::Cursor::new(Vec::new());
        bytes
            .write_all(&self.diversifier().0)
            .expect("can write diversifier into vec");
        bytes
            .write_all(&self.transmission_key().0)
            .expect("can write transmission key into vec");
        bytes
            .write_all(&self.clue_key().0)
            .expect("can write clue key into vec");

        f4jumble(bytes.get_ref()).expect("can jumble")
    }

    /// A randomized dummy address.
    pub fn dummy<R: CryptoRng + Rng>(rng: &mut R) -> Self {
        let mut diversifier_bytes = [0u8; 16];
        rng.fill_bytes(&mut diversifier_bytes);

        let mut pk_d_bytes = [0u8; 32];
        rng.fill_bytes(&mut pk_d_bytes);

        let mut clue_key_bytes = [0; 32];
        rng.fill_bytes(&mut clue_key_bytes);

        let diversifier = Diversifier(diversifier_bytes);
        Address::from_components(
            diversifier,
            ka::Public(pk_d_bytes),
            fmd::ClueKey(clue_key_bytes),
        )
        .expect("generated dummy address")
    }

    /// Short form suitable for displaying in a UI.
    pub fn display_short_form(&self) -> String {
        let full_address = format!("{}", self);
        // Fixed prefix is `penumbrav2t` plus the Bech32m separator `1`.
        let fixed_prefix = format!("{}{}", bech32str::address::BECH32_PREFIX, '1');
        let num_chars_to_display = fixed_prefix.len() + ADDRESS_NUM_CHARS_SHORT_FORM;

        format!("{}…", &full_address[0..num_chars_to_display])
    }
}

impl Protobuf<pb::Address> for Address {}

impl From<Address> for pb::Address {
    fn from(a: Address) -> Self {
        pb::Address { inner: a.to_vec() }
    }
}

impl TryFrom<pb::Address> for Address {
    type Error = anyhow::Error;

    fn try_from(value: pb::Address) -> Result<Self, Self::Error> {
        (&value.inner).try_into()
    }
}

impl std::fmt::Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let proto_address = pb::Address::from(self.clone());
        f.write_str(&bech32str::encode(
            &proto_address.inner,
            bech32str::address::BECH32_PREFIX,
            bech32str::Bech32m,
        ))
    }
}

impl std::fmt::Debug for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        <Self as std::fmt::Display>::fmt(self, f)
    }
}

impl std::str::FromStr for Address {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        pb::Address {
            inner: bech32str::decode(s, bech32str::address::BECH32_PREFIX, bech32str::Bech32m)?,
        }
        .try_into()
    }
}

impl TryFrom<Vec<u8>> for Address {
    type Error = anyhow::Error;

    fn try_from(jumbled_vec: Vec<u8>) -> Result<Self, Self::Error> {
        (&jumbled_vec[..]).try_into()
    }
}

impl TryFrom<&Vec<u8>> for Address {
    type Error = anyhow::Error;

    fn try_from(jumbled_vec: &Vec<u8>) -> Result<Self, Self::Error> {
        (jumbled_vec[..]).try_into()
    }
}

impl TryFrom<&[u8]> for Address {
    type Error = anyhow::Error;

    fn try_from(jumbled_bytes: &[u8]) -> Result<Self, Self::Error> {
        if jumbled_bytes.len() != ADDRESS_LEN_BYTES {
            return Err(anyhow::anyhow!("address malformed"));
        }

        let unjumbled_bytes =
            f4jumble_inv(jumbled_bytes).ok_or_else(|| anyhow::anyhow!("invalid address"))?;
        let mut bytes = Cursor::new(unjumbled_bytes);

        let mut diversifier_bytes = [0u8; 16];
        bytes
            .read_exact(&mut diversifier_bytes)
            .map_err(|_| anyhow::anyhow!("address malformed"))?;

        let mut pk_d_bytes = [0u8; 32];
        bytes
            .read_exact(&mut pk_d_bytes)
            .map_err(|_| anyhow::anyhow!("address malformed"))?;

        let mut clue_key_bytes = [0; 32];
        bytes
            .read_exact(&mut clue_key_bytes)
            .map_err(|_| anyhow::anyhow!("address malformed"))?;

        let diversifier = Diversifier(diversifier_bytes);

        let address = Address::from_components(
            diversifier,
            ka::Public(pk_d_bytes),
            fmd::ClueKey(clue_key_bytes),
        );

        if address.is_none() {
            return Err(anyhow::anyhow!("address malformed"));
        }

        Ok(address.unwrap())
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use rand_core::OsRng;

    use super::*;
    use crate::keys::{SeedPhrase, SpendKey};

    #[test]
    fn test_address_encoding() {
        let mut rng = OsRng;
        let seed_phrase = SeedPhrase::generate(&mut rng);
        let sk = SpendKey::from_seed_phrase(seed_phrase, 0);
        let fvk = sk.full_viewing_key();
        let ivk = fvk.incoming();
        let (dest, _dtk_d) = ivk.payment_address(0u64.into());

        let encoded_addr = format!("{}", dest);

        let addr = Address::from_str(&encoded_addr).expect("can decode valid address");

        assert_eq!(addr, dest);
    }

    #[test]
    fn test_bytes_roundtrip() {
        let mut rng = OsRng;
        let seed_phrase = SeedPhrase::generate(&mut rng);
        let sk = SpendKey::from_seed_phrase(seed_phrase, 0);
        let fvk = sk.full_viewing_key();
        let ivk = fvk.incoming();
        let (dest, _dtk_d) = ivk.payment_address(0u64.into());

        let bytes = dest.to_vec();
        let addr: Address = bytes.try_into().expect("can decode valid address");

        assert_eq!(addr, dest);
    }

    #[test]
    fn test_address_keys_are_diversified() {
        let mut rng = OsRng;
        let seed_phrase = SeedPhrase::generate(&mut rng);
        let sk = SpendKey::from_seed_phrase(seed_phrase, 0);
        let fvk = sk.full_viewing_key();
        let ivk = fvk.incoming();
        let (dest1, dtk_d1) = ivk.payment_address(0u64.into());
        let (dest2, dtk_d2) = ivk.payment_address(1u64.into());

        assert!(dest1.transmission_key() != dest2.transmission_key());
        assert!(dest1.clue_key() != dest2.clue_key());
        assert!(dtk_d1.to_bytes() != dtk_d2.to_bytes());
    }
}
