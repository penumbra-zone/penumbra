use ark_ff::PrimeField;

use crate::{
    ka,
    rdsa::{SpendAuth, VerificationKey},
    Fr,
};

pub const OVK_LEN_BYTES: usize = 32;
pub const IVK_LEN_BYTES: usize = 32;

use super::{NullifierDerivingKey, NK_LEN_BYTES, SPENDSEED_LEN_BYTES};

/// An `OutgoingViewingKey` allows one to identify outgoing notes.
#[derive(Copy, Clone)]
pub struct OutgoingViewingKey(pub [u8; OVK_LEN_BYTES]);

/// The `FullViewingKey` allows one to identify incoming and outgoing notes only.
pub struct FullViewingKey {
    pub(super) ak: VerificationKey<SpendAuth>,
    pub(super) nk: NullifierDerivingKey,
    pub(super) ovk: OutgoingViewingKey,
}

impl FullViewingKey {
    /// Derive the incoming viewing key for this full viewing key.
    pub fn incoming(&self) -> ka::Secret {
        let mut hasher = blake2b_simd::State::new();
        hasher.update(b"Penumbra_IncomingViewingKey");
        let ak_bytes: [u8; SPENDSEED_LEN_BYTES] = self.ak.into();
        hasher.update(&ak_bytes);
        let nk_bytes: [u8; NK_LEN_BYTES] = self.nk.0.compress().into();
        hasher.update(&nk_bytes);
        let hash_result = hasher.finalize();
        let sk = Fr::from_le_bytes_mod_order(hash_result.as_bytes());

        ka::Secret::new_from_field(sk)
    }

    /// Return the outgoing viewing key for this full viewing key.
    pub fn outgoing(&self) -> &OutgoingViewingKey {
        &self.ovk
    }
}
