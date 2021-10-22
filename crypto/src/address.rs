use ark_serialize::CanonicalDeserialize;
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
