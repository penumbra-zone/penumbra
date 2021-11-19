use ark_ff::PrimeField;

use crate::{fmd, ka, prf, Address, Fr};

use super::{DiversifierIndex, DiversifierKey};

pub const IVK_LEN_BYTES: usize = 64;

/// Allows viewing incoming notes, i.e., notes sent to the spending key this
/// key is derived from.
#[derive(Clone, Debug)]
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

        let dtk_d = fmd::DetectionKey::from_field(Fr::from_le_bytes_mod_order(
            prf::expand(b"PenumbraExpndFMD", &self.ivk.to_bytes(), d.as_ref()).as_bytes(),
        ));
        let ck_d = dtk_d.clue_key();

        (
            Address::from_components(d, g_d, pk_d, ck_d).expect("pk_d is valid"),
            dtk_d,
        )
    }

    /// Perform key agreement with a given public key.
    pub fn key_agreement_with(&self, pk: &ka::Public) -> Result<ka::SharedSecret, ka::Error> {
        self.ivk.key_agreement_with(pk)
    }

    /// Derive a transmission key from the given diversified base.
    pub fn diversified_public(&self, diversified_generator: &decaf377::Element) -> ka::Public {
        self.ivk.diversified_public(diversified_generator)
    }
}
