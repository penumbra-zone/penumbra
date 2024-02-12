use ark_ff::{PrimeField, Zero};
use rand_core::{CryptoRng, RngCore};

use ark_r1cs_std::prelude::*;
use ark_relations::r1cs::SynthesisError;
use decaf377::{
    r1cs::{ElementVar, FqVar},
    Fq, Fr,
};

use super::{AddressIndex, Diversifier, DiversifierKey};
use crate::{
    fmd, ka,
    keys::{AuthorizationKeyVar, NullifierKeyVar, IVK_DOMAIN_SEP},
    prf, Address,
};

pub const IVK_LEN_BYTES: usize = 64;
const MOD_R_QUOTIENT: usize = 4;

/// Allows viewing incoming notes, i.e., notes sent to the spending key this
/// key is derived from.
#[derive(Clone, Debug, PartialEq, Eq)]
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
    inner: FqVar,
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

        // OOC: Reduce `ivk_mod_q` modulo r
        let r_modulus: Fq = Fq::from(Fr::MODULUS);
        let ivk_mod_q_ooc: Fq = ivk_mod_q.value().unwrap_or_default();
        let ivk_mod_r_ooc = Fr::from_le_bytes_mod_order(&ivk_mod_q_ooc.to_bytes());

        // We also need ivk reduced mod r as an Fq for inserting back into the circuit
        let ivk_mod_r_ooc_q = Fq::from_le_bytes_mod_order(&ivk_mod_r_ooc.to_bytes());
        let ivk_mod_r = FqVar::new_witness(cs.clone(), || Ok(ivk_mod_r_ooc_q))?;

        // Finally, we figure out how many times we needed to subtract r from ivk_mod_q_ooc to get ivk_mod_r_ooc.
        let mut temp_ivk_mod_q = ivk_mod_q_ooc;
        let mut a = 0;
        while temp_ivk_mod_q > r_modulus {
            temp_ivk_mod_q -= r_modulus;
            a += 1;
        }

        // Now we add constraints to demonstrate that `ivk_mod_r` is the correct
        // reduction from `ivk_mod_q`.
        //
        // Constrain: ivk_mod_q = mod_r * a + ivk_mod_r
        let mod_r_var = FqVar::new_constant(cs.clone(), r_modulus)?;
        let a_var = FqVar::new_witness(cs.clone(), || Ok(Fq::from(a as u64)))?;
        let rhs = &mod_r_var * &a_var + &ivk_mod_r;
        ivk_mod_q.enforce_equal(&rhs)?;

        // Constrain: a <= 4
        //
        // We could use `enforce_cmp` to add an a <= 4 constraint, but it's cheaper
        // to add constraints to demonstrate a(a-1)(a-2)(a-3)(a-4) = 0.
        let mut mul = a_var.clone();
        for i in 1..=MOD_R_QUOTIENT {
            mul *= a_var.clone() - FqVar::new_constant(cs.clone(), Fq::from(i as u64))?;
        }
        let zero = FqVar::new_constant(cs, Fq::zero())?;
        mul.enforce_equal(&zero)?;

        // Constrain: ivk_mod_r < r
        // Here we can use the existing `enforce_cmp` method on FqVar as r <= (q-1)/2.
        ivk_mod_r.enforce_cmp(&mod_r_var, core::cmp::Ordering::Less, false)?;

        Ok(IncomingViewingKeyVar { inner: ivk_mod_r })
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
    use crate::keys::{Bip44Path, SeedPhrase, SpendKey};
    use proptest::prelude::*;

    use super::*;

    #[test]
    fn views_address_succeeds_on_own_address() {
        let rng = rand::rngs::OsRng;
        let spend_key =
            SpendKey::from_seed_phrase_bip44(SeedPhrase::generate(rng), &Bip44Path::new(0));
        let ivk = spend_key.full_viewing_key().incoming();
        let own_address = ivk.payment_address(AddressIndex::from(0u32)).0;
        assert!(ivk.views_address(&own_address));
    }

    proptest! {
        #[test]
        fn views_address_succeeds_on_own_ephemeral_address(address_index in any::<u32>()) {
            let rng = rand::rngs::OsRng;
            let spend_key = SpendKey::from_seed_phrase_bip44(SeedPhrase::generate(rng), &Bip44Path::new(0));
            let fvk = spend_key.full_viewing_key();
            let (own_address, _) = fvk.ephemeral_address(rng, AddressIndex::from(address_index));
            let ivk = fvk.incoming();
            assert!(ivk.views_address(&own_address));

            let derived_address_index = fvk.address_index(&own_address);
            assert_eq!(derived_address_index.expect("index exists").account, AddressIndex::from(address_index).account);
        }
    }

    #[test]
    fn views_address_fails_on_other_address() {
        let rng = rand::rngs::OsRng;
        let spend_key =
            SpendKey::from_seed_phrase_bip44(SeedPhrase::generate(rng), &Bip44Path::new(0));
        let ivk = spend_key.full_viewing_key().incoming();

        let other_address =
            SpendKey::from_seed_phrase_bip44(SeedPhrase::generate(rng), &Bip44Path::new(0))
                .full_viewing_key()
                .incoming()
                .payment_address(AddressIndex::from(0u32))
                .0;

        assert!(!ivk.views_address(&other_address));
    }

    #[test]
    fn enforce_field_assumptions() {
        use num_bigint::BigUint;
        use num_traits::ops::checked::CheckedSub;

        let fq_modulus: BigUint = Fq::MODULUS.into();
        let max_q: BigUint = &fq_modulus - 1u32;
        let fr_modulus: BigUint = Fr::MODULUS.into();
        assert!(
            fr_modulus < fq_modulus,
            "we assume that our scalar field is smaller than our base field"
        );

        let mut multiple = 0;
        let mut res = max_q;
        loop {
            res = if let Some(x) = res.checked_sub(&fr_modulus) {
                multiple += 1;
                x
            } else {
                break;
            };
        }

        assert_eq!(
            MOD_R_QUOTIENT, multiple,
            "`a = fr_modulus * 4 + r mod q` only works on specific curve parameters"
        );
    }
}
