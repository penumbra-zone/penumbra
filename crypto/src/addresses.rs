use ark_serialize::CanonicalDeserialize;
use decaf377;

use crate::{ka, keys::Diversifier, Fq};

/// A valid payment address.
#[derive(Clone)]
pub struct PaymentAddress {
    diversifier: Diversifier,
    diversified_generator: decaf377::Element,
    transmission_key: ka::Public,
    // A cached copy of the s value of the transmission key's encoding.
    tk_s: Fq,
}

impl PaymentAddress {
    pub fn new(ivk: ka::Secret, diversifier: Diversifier) -> Self {
        let diversified_generator = diversifier.diversified_generator();
        let transmission_key = ivk.diversified_public(&diversified_generator);
        // XXX ugly -- better way to get our hands on the s value?
        // add to decaf377::Encoding? there's compress_to_field already...
        let tk_s = Fq::deserialize(&transmission_key.0[..]).expect("transmission key is valid");

        PaymentAddress {
            diversifier,
            diversified_generator,
            transmission_key,
            tk_s,
        }
    }

    pub fn diversifier(&self) -> &Diversifier {
        &self.diversifier
    }

    pub fn diversified_generator(&self) -> &decaf377::Element {
        &self.diversified_generator
    }

    pub fn transmission_key(&self) -> &ka::Public {
        &self.transmission_key
    }

    pub(crate) fn tk_s(&self) -> &Fq {
        &self.tk_s
    }
}
