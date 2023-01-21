use ark_ff::PrimeField;
use rand_core::{CryptoRng, RngCore};

use super::{AddressIndex, Diversifier, DiversifierKey};
use crate::{fmd, ka, prf, Address, Fr};

pub const IVK_LEN_BYTES: usize = 64;

/// Allows viewing incoming notes, i.e., notes sent to the spending key this
/// key is derived from.
#[derive(Clone, Debug)]
pub struct IncomingViewingKey {
    pub(super) ivk: ka::Secret,
    pub(super) dk: DiversifierKey,
}

impl IncomingViewingKey {
    /// Derive a shielded payment address with the given [`AddressIndex`].
    pub fn payment_address(&self, index: AddressIndex) -> (Address, fmd::DetectionKey) {
        let d = self.dk.diversifier_for_index(&index);
        let g_d = d.diversified_generator();
        let pk_d = self.ivk.diversified_public(&g_d);

        let dtk_d = fmd::DetectionKey::from_field(Fr::from_le_bytes_mod_order(
            prf::expand(b"PenumbraExpndFMD", &self.ivk.to_bytes(), d.as_ref()).as_bytes(),
        ));
        let ck_d = dtk_d.clue_key();

        (
            Address::from_components(d, pk_d, ck_d).expect("pk_d is valid"),
            dtk_d,
        )
    }

    /// Derive a random ephemeral address.
    pub fn ephemeral_address<R: RngCore + CryptoRng>(
        &self,
        mut rng: R,
    ) -> (Address, fmd::DetectionKey) {
        let mut random_index = [0u8; 16];
        // ensure that the index is outside the range of u64 with rejection sampling
        while u128::from_le_bytes(random_index) <= 2u128.pow(64) {
            rng.fill_bytes(&mut random_index);
        }
        let index = AddressIndex::Random(random_index);
        self.payment_address(index)
    }

    /// Perform key agreement with a given public key.
    pub fn key_agreement_with(&self, pk: &ka::Public) -> Result<ka::SharedSecret, ka::Error> {
        self.ivk.key_agreement_with(pk)
    }

    /// Derive a transmission key from the given diversified base.
    pub fn diversified_public(&self, diversified_generator: &decaf377::Element) -> ka::Public {
        self.ivk.diversified_public(diversified_generator)
    }

    /// Returns the index used to create the given diversifier (if it was
    /// created using this incoming viewing key)
    pub fn index_for_diversifier(&self, diversifier: &Diversifier) -> AddressIndex {
        self.dk.index_for_diversifier(diversifier)
    }

    /// Check whether this address is viewable by this incoming viewing key.
    pub fn views_address(&self, address: &Address) -> bool {
        self.ivk.diversified_public(address.diversified_generator()) == *address.transmission_key()
    }

    /// Returns the index of the given address, if the address is viewed by this
    /// viewing key; otherwise, returns `None`.
    pub fn address_index(&self, address: &Address) -> Option<AddressIndex> {
        if self.views_address(address) {
            Some(self.index_for_diversifier(address.diversifier()))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use crate::keys::{SeedPhrase, SpendKey};

    use super::*;

    #[test]
    fn views_address_succeeds_on_own_address() {
        let rng = rand::rngs::OsRng;
        let spend_key = SpendKey::from_seed_phrase(SeedPhrase::generate(rng), 0);
        let ivk = spend_key.full_viewing_key().incoming();
        let own_address = ivk.payment_address(AddressIndex::from(0u64)).0;
        assert!(ivk.views_address(&own_address));
    }

    #[test]
    fn views_address_fails_on_other_address() {
        let rng = rand::rngs::OsRng;
        let spend_key = SpendKey::from_seed_phrase(SeedPhrase::generate(rng), 0);
        let ivk = spend_key.full_viewing_key().incoming();

        let other_address = SpendKey::from_seed_phrase(SeedPhrase::generate(rng), 0)
            .full_viewing_key()
            .incoming()
            .payment_address(AddressIndex::from(0u64))
            .0;

        assert!(!ivk.views_address(&other_address));
    }
}
