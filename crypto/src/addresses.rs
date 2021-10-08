use decaf377;

use crate::keys::{Diversifier, TransmissionKey};

/// Represents a diversified Payment Address.
pub struct PaymentAddress {
    pub diversifier: Diversifier,
    // The diversified generator.
    pub g_d: decaf377::Element,
    pub transmission_key: TransmissionKey,
}

impl PaymentAddress {
    pub fn new(diversifier: Diversifier, transmission_key: TransmissionKey) -> Self {
        PaymentAddress {
            diversifier,
            g_d: diversifier.diversified_generator(),
            transmission_key,
        }
    }
}
