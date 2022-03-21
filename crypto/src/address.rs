use std::io::{Cursor, Read, Write};

use ark_serialize::CanonicalDeserialize;
use f4jumble::{f4jumble, f4jumble_inv};
use penumbra_proto::{crypto as pb, serializers::bech32str};
use serde::{Deserialize, Serialize};

use crate::{fmd, ka, keys::Diversifier, Fq};

// We pad addresses to 80 bytes (before jumbling and Bech32m encoding)
// using this 5 byte padding.
const ADDR_PADDING: &[u8] = "pen00".as_bytes();

/// A valid payment address.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
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
    /// cached s value
    cached_s: Fq,

    ck_d: fmd::ClueKey,
}

impl Address {
    /// Constructs a payment address from its components.
    ///
    /// Returns `None` if the bytes in pk_d are a non-canonical representation
    /// of an [`Fq`] `s` value.
    pub(crate) fn from_components(
        d: Diversifier,
        g_d: decaf377::Element,
        pk_d: ka::Public,
        ck_d: fmd::ClueKey,
    ) -> Option<Self> {
        // XXX ugly -- better way to get our hands on the s value?
        // add to decaf377::Encoding? there's compress_to_field already...
        if let Ok(cached_s) = Fq::deserialize(&pk_d.0[..]) {
            // don't need an error type here, caller will probably .expect anyways
            Some(Self {
                d,
                g_d,
                pk_d,
                ck_d,
                cached_s,
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
}

impl From<Address> for pb::Address {
    fn from(a: Address) -> Self {
        let mut bytes = std::io::Cursor::new(Vec::new());
        bytes
            .write_all(&a.diversifier().0)
            .expect("can write diversifier into vec");
        bytes
            .write_all(&a.transmission_key().0)
            .expect("can write transmission key into vec");
        bytes
            .write_all(&a.clue_key().0)
            .expect("can write clue key into vec");
        bytes
            .write_all(ADDR_PADDING)
            .expect("can write padding into vec");

        let jumbled_bytes = f4jumble(bytes.get_ref()).expect("can jumble");
        pb::Address {
            inner: jumbled_bytes,
        }
    }
}

impl TryFrom<pb::Address> for Address {
    type Error = anyhow::Error;
    fn try_from(value: pb::Address) -> Result<Self, Self::Error> {
        let unjumbled_bytes =
            f4jumble_inv(&value.inner).ok_or_else(|| anyhow::anyhow!("invalid address"))?;
        let mut bytes = Cursor::new(unjumbled_bytes);

        let mut diversifier_bytes = [0u8; 11];
        bytes.read_exact(&mut diversifier_bytes)?;

        let mut pk_d_bytes = [0u8; 32];
        bytes.read_exact(&mut pk_d_bytes)?;

        let mut clue_key_bytes = [0; 32];
        bytes.read_exact(&mut clue_key_bytes)?;

        let mut padding_bytes = [0; 5];
        bytes.read_exact(&mut padding_bytes)?;

        if padding_bytes != ADDR_PADDING {
            return Err(anyhow::anyhow!("invalid address"));
        }

        let diversifier = Diversifier(diversifier_bytes);
        Address::from_components(
            diversifier,
            diversifier.diversified_generator(),
            ka::Public(pk_d_bytes),
            fmd::ClueKey(clue_key_bytes),
        )
        .ok_or_else(|| anyhow::anyhow!("invalid address"))
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

impl std::str::FromStr for Address {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        pb::Address {
            inner: bech32str::decode(s, bech32str::address::BECH32_PREFIX, bech32str::Bech32m)?,
        }
        .try_into()
    }
}

/// Parse v0 testnet address string (temporary migration used in `pcli`)
pub fn parse_v0_testnet_address(v0_address: String) -> Result<Address, anyhow::Error> {
    let decoded_bytes = &bech32str::decode(&v0_address, "penumbrav0t", bech32str::Bech32m)?;

    let mut bytes = Cursor::new(decoded_bytes);
    let mut diversifier_bytes = [0u8; 11];
    bytes.read_exact(&mut diversifier_bytes)?;
    let mut pk_d_bytes = [0u8; 32];
    bytes.read_exact(&mut pk_d_bytes)?;
    let mut clue_key_bytes = [0; 32];
    bytes.read_exact(&mut clue_key_bytes)?;

    let diversifier = Diversifier(diversifier_bytes);
    let addr = Address::from_components(
        diversifier,
        diversifier.diversified_generator(),
        ka::Public(pk_d_bytes),
        fmd::ClueKey(clue_key_bytes),
    )
    .ok_or_else(|| anyhow::anyhow!("invalid address"))?;

    Ok(addr)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use rand_core::OsRng;

    use super::*;
    use crate::keys::{SeedPhrase, SpendKey, SpendSeed};

    #[test]
    fn test_address_encoding() {
        let mut rng = OsRng;
        let seed_phrase = SeedPhrase::generate(&mut rng);
        let spend_seed = SpendSeed::from_seed_phrase(seed_phrase, 0);
        let sk = SpendKey::new(spend_seed);
        let fvk = sk.full_viewing_key();
        let ivk = fvk.incoming();
        let (dest, _dtk_d) = ivk.payment_address(0u64.into());

        let encoded_addr = format!("{}", dest);

        let addr = Address::from_str(&encoded_addr).expect("can decode valid address");

        assert_eq!(addr, dest);
    }
}
