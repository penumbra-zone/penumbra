use ark_ff::PrimeField;

use crate::{
    rdsa::{SpendAuth, VerificationKey},
    Fr,
};

pub const OVK_LEN_BYTES: usize = 32;
pub const IVK_LEN_BYTES: usize = 32;

use super::{
    Diversifier, NullifierDerivingKey, TransmissionKey, NK_LEN_BYTES, SPENDSEED_LEN_BYTES,
};

/// An `IncomingViewingKey` allows one to identify incoming notes.
pub struct IncomingViewingKey(pub Fr);

impl IncomingViewingKey {
    pub fn derive(ak: &VerificationKey<SpendAuth>, nk: &NullifierDerivingKey) -> Self {
        let mut hasher = blake2b_simd::State::new();
        hasher.update(b"Penumbra_IncomingViewingKey");
        let ak_bytes: [u8; SPENDSEED_LEN_BYTES] = ak.into();
        hasher.update(&ak_bytes);
        let nk_bytes: [u8; NK_LEN_BYTES] = nk.0.compress().into();
        hasher.update(&nk_bytes);
        let hash_result = hasher.finalize();

        Self(Fr::from_le_bytes_mod_order(hash_result.as_bytes()))
    }

    #[allow(non_snake_case)]
    pub fn derive_transmission_key(&self, d: &Diversifier) -> TransmissionKey {
        let B_d = d.diversified_generator();
        TransmissionKey(self.0 * B_d)
    }
}

/// An `OutgoingViewingKey` allows one to identify outgoing notes.
#[derive(Copy, Clone)]
pub struct OutgoingViewingKey(pub [u8; OVK_LEN_BYTES]);

/// The `FullViewingKey` allows one to identify incoming and outgoing notes only.
pub struct FullViewingKey {
    pub ak: VerificationKey<SpendAuth>,
    pub nk: NullifierDerivingKey,
    pub ovk: OutgoingViewingKey,
}
