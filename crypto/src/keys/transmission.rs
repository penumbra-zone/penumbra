use decaf377;

/// Represents a (diversified) transmission key.
#[derive(Copy, Clone)]
pub struct TransmissionKey(pub decaf377::Element);
