use crate::Fr;

pub const NK_LEN_BYTES: usize = 32;

#[derive(Copy, Clone)]
pub struct NullifierPrivateKey(pub Fr);

pub struct NullifierDerivingKey(pub decaf377::Element);
