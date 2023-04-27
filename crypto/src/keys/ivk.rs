use ark_ff::PrimeField;
use rand_core::{CryptoRng, RngCore};

use ark_r1cs_std::fields::nonnative::NonNativeFieldVar;
use ark_r1cs_std::prelude::*;
use ark_relations::r1cs::SynthesisError;
use decaf377::{
    r1cs::{ElementVar, FqVar},
    FieldExt, Fq,
};

use super::{AddressIndex, Diversifier, DiversifierKey};
use crate::{
    fmd, ka,
    keys::{AuthorizationKeyVar, NullifierKeyVar, IVK_DOMAIN_SEP},
    prf, Address, Fr,
};

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

    /// Derive an ephemeral address for the provided account.
    pub fn ephemeral_address<R: RngCore + CryptoRng>(
        &self,
        mut rng: R,
        mut address_index: AddressIndex,
    ) -> (Address, fmd::DetectionKey) {
        let mut random_index = [0u8; 12];

        rng.fill_bytes(&mut random_index);

        address_index.randomizer = random_index;

        self.payment_address(address_index)
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
    // TODO: re-evaluate relative to FVK methods
    pub(super) fn address_index(&self, address: &Address) -> Option<AddressIndex> {
        if self.views_address(address) {
            Some(self.index_for_diversifier(address.diversifier()))
        } else {
            None
        }
    }
}

pub struct IncomingViewingKeyVar {
    inner: NonNativeFieldVar<Fr, Fq>,
}

impl IncomingViewingKeyVar {
    /// Derive the incoming viewing key from the nk and the ak.
    pub fn derive(nk: &NullifierKeyVar, ak: &AuthorizationKeyVar) -> Result<Self, SynthesisError> {
        let cs = nk.inner.cs();
        let ivk_domain_sep = FqVar::new_constant(cs.clone(), *IVK_DOMAIN_SEP)?;
        let ivk_mod_q = poseidon377::r1cs::hash_2(
            cs.clone(),
            &ivk_domain_sep,
            (nk.inner.clone(), ak.inner.compress_to_field()?),
        )?;

        // Reduce `ivk_mod_q` modulo r
        let inner_ivk_mod_q: Fq = ivk_mod_q.value().unwrap_or_default();
        let ivk_mod_r = Fr::from_le_bytes_mod_order(&inner_ivk_mod_q.to_bytes());
        let ivk = NonNativeFieldVar::<Fr, Fq>::new_variable(
            cs,
            || Ok(ivk_mod_r),
            AllocationMode::Witness,
        )?;
        Ok(IncomingViewingKeyVar { inner: ivk })
    }

    /// Derive a transmission key from the given diversified base.
    pub fn diversified_public(
        &self,
        diversified_generator: &ElementVar,
    ) -> Result<ElementVar, SynthesisError> {
        let ivk_vars = self.inner.to_bits_le()?;
        diversified_generator.scalar_mul_le(ivk_vars.to_bits_le()?.iter())
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
        let own_address = ivk.payment_address(AddressIndex::from(0u32)).0;
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
            .payment_address(AddressIndex::from(0u32))
            .0;

        assert!(!ivk.views_address(&other_address));
    }
}
