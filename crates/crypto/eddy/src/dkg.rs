//! Types used to perform distributed key generation (WIP).

use crate::PublicKeyShare;

pub struct Committee {
    pub shares: Vec<PublicKeyShare>,
    pub threshold: u32,
}
