use crate::PublicKeyShare;

pub struct Committee {
    pub shares: Vec<PublicKeyShare>,
    pub threshold: u32,
}
