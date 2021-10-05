use crate::keys::{Diversifier, TransmissionKey};

/// Represents a diversified Payment Address.
pub struct PaymentAddress {
    pub diversifier: Diversifier,
    // TODO: Diversiified generator should get added here
    pub transmission_key: TransmissionKey,
}
