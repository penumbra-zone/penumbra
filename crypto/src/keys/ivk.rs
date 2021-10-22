use ark_ff::PrimeField;

use crate::{fmd, ka, Address, Fr};

use super::{DiversifierIndex, DiversifierKey};

pub const IVK_LEN_BYTES: usize = 64;

/// Allows viewing incoming notes, i.e., notes sent to the spending key this
/// key is derived from.
#[derive(Clone)]
pub struct IncomingViewingKey {
    pub(super) ivk: ka::Secret,
    pub(super) dk: DiversifierKey,
}

impl IncomingViewingKey {
    /// Derive a shielded payment address with the given [`DiversifierIndex`].
    pub fn payment_address(&self, index: DiversifierIndex) -> (Address, fmd::DetectionKey) {
        let d = self.dk.diversifier_for_index(&index);
        let g_d = d.diversified_generator();
        let pk_d = self.ivk.diversified_public(&g_d);

        let dtk_d = {
            let mut hasher = blake2b_simd::State::new();
            hasher.update(b"PenumbraExpndFMD");
            hasher.update(&self.ivk.to_bytes());
            hasher.update(d.as_ref());
            let hash_result = hasher.finalize();

            fmd::DetectionKey::from_field(Fr::from_le_bytes_mod_order(hash_result.as_bytes()))
        };
        let ck_d = dtk_d.clue_key();

        (
            Address::from_components(d, g_d, pk_d, ck_d).expect("pk_d is valid"),
            dtk_d,
        )
    }

    /// Derive a transmission key from the given diversified base.
    pub fn diversified_public(&self, diversified_generator: &decaf377::Element) -> ka::Public {
        self.ivk.diversified_public(diversified_generator)
    }
}
