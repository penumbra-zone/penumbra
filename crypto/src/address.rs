use std::io::Write;

use ark_serialize::CanonicalDeserialize;
use bech32::{ToBase32, Variant};
use decaf377;

use crate::{fmd, ka, keys::Diversifier, Fq};

/// A valid payment address.
#[derive(Clone)]
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

impl std::fmt::Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut addr_content = std::io::Cursor::new(Vec::new());
        addr_content
            .write_all(&self.diversifier().0)
            .expect("can write diversifier into vec");
        addr_content
            .write_all(&self.transmission_key().0)
            .expect("can write transmission key into vec");
        addr_content
            .write_all(&self.clue_key().0)
            .expect("can write clue key into vec");

        // Mainnet "pm" prefix, testnet "pt" prefix
        let human_part = "pt";
        bech32::encode_to_fmt(
            f,
            human_part,
            addr_content.get_ref().to_base32(),
            Variant::Bech32m,
        )
        .map_err(|_| std::fmt::Error)?
    }
}

#[cfg(test)]
mod tests {
    use crate::keys::SpendKey;
    use rand_core::OsRng;

    #[test]
    fn test_address_encoding() {
        let sk = SpendKey::generate(OsRng);
        let fvk = sk.full_viewing_key();
        let ivk = fvk.incoming();
        let (dest, _dtk_d) = ivk.payment_address(0u64.into());

        let _encoded_addr = format!("{}", dest);
    }
}
