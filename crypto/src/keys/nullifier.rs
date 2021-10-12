use ark_ff::PrimeField;

use crate::Fr;

use super::SpendingKey;

pub const NK_LEN_BYTES: usize = 32;

#[derive(Copy, Clone)]
pub struct NullifierPrivateKey(pub Fr);

impl NullifierPrivateKey {
    pub fn derive(key: &SpendingKey) -> Self {
        let mut hasher = blake2b_simd::State::new();
        hasher.update(b"Penumbra_ExpandSeed");
        hasher.update(&key.0);
        hasher.update(&[1; 1]);
        let hash_result = hasher.finalize();

        Self(Fr::from_le_bytes_mod_order(hash_result.as_bytes()))
    }
}

pub struct NullifierDerivingKey(pub decaf377::Element);

impl NullifierDerivingKey {
    pub fn derive(nsk: &NullifierPrivateKey) -> Self {
        Self(decaf377::Element::basepoint() * nsk.0)
    }
}
