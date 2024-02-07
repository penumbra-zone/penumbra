#![deny(clippy::unwrap_used)]
// Requires nightly.
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

use ark_ec::pairing::Pairing;
use ark_ff::Field;
use anyhow::{bail, Result};
use decaf377::Bls12_377;
use decaf377_v05::{Bls12_377 as Bls12_377_05, Fr};
use once_cell::sync::{Lazy, OnceCell};
use penumbra_keys::keys::SeedPhrase;
use penumbra_shielded_pool::{ConvertCircuit, Note, NullifierDerivationCircuit, OutputCircuit, Rseed, SpendCircuit};
use penumbra_dex::{swap::proof::SwapCircuit, swap_claim::proof::SwapClaimCircuit};
use ark_groth16::{PreparedVerifyingKey, ProvingKey, VerifyingKey};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use std::ops::Deref;
use rand_chacha::{ChaCha20Rng, rand_core::SeedableRng};

/// Proving key associated with decaf377 v0.5 backend
#[derive(Debug, Default)]
pub struct LazyProvingKeyOld {
    pk_id: &'static str,
    inner: OnceCell<ProvingKey<Bls12_377_05>>,
}

impl LazyProvingKeyOld {
    fn new(pk_id: &'static str) -> Self {
        LazyProvingKeyOld {
            pk_id,
            inner: OnceCell::new(),
        }
    }

    pub fn try_load(&self, bytes: &[u8]) -> Result<&ProvingKey<Bls12_377_05>> {
        println!("entered try_load!");
        self.inner.get_or_try_init(|| {
            let pk = ProvingKey::deserialize_uncompressed_unchecked(bytes)?;

            let pk_id = pk.debug_id();
            println!("pk_id: {:?}", pk_id);

            if pk_id != self.pk_id {
                bail!(
                    "proving key ID mismatch: expected {}, loaded {}",
                    self.pk_id,
                    pk_id
                );
            }
            Ok(pk)
        })
    }
}

impl Deref for LazyProvingKeyOld {
    type Target = ProvingKey<Bls12_377_05>;

    fn deref(&self) -> &Self::Target {
        self.inner.get().expect("Proving key cannot be loaded!")
    }
}

pub trait ProvingKeyExt {
    fn debug_id(&self) -> String;
}

impl ProvingKeyExt for ProvingKey<Bls12_377_05> {
    fn debug_id(&self) -> String {
        let mut buf = Vec::new();
        self.serialize_compressed(&mut buf)
            .expect("can serialize pk");
        use sha2::Digest;
        let hash = sha2::Sha256::digest(&buf);
        use bech32::ToBase32;
        bech32::encode("groth16pk", hash.to_base32(), bech32::Variant::Bech32m)
            .expect("can encode pk as bech32")
    }
}
pub trait VerifyingKeyExt {
    fn debug_id(&self) -> String;
}

impl VerifyingKeyExt for VerifyingKey<Bls12_377_05> {
    fn debug_id(&self) -> String {
        let mut buf = Vec::new();
        self.serialize_compressed(&mut buf)
            .expect("can serialize vk");
        use sha2::Digest;
        let hash = sha2::Sha256::digest(&buf);
        use bech32::ToBase32;
        bech32::encode("groth16vk", hash.to_base32(), bech32::Variant::Bech32m)
            .expect("can encode vk as bech32")
    }
}

pub mod spend {
    include!("../../../crates/crypto/proof-params/src/gen/spend_id.rs");
}

pub mod delegator_vote {
    include!("../../../crates/crypto/proof-params/src/gen/delegator_vote_id.rs");
}

pub static SPEND_PROOF_PROVING_KEY_OLD: Lazy<LazyProvingKeyOld> = Lazy::new(|| {
    let spend_proving_key = LazyProvingKeyOld::new(spend::PROVING_KEY_ID);

    spend_proving_key
        .try_load(include_bytes!("../../../crates/crypto/proof-params/src/gen/spend_pk.bin"))
        .expect("bundled proving key is valid");

    spend_proving_key
});

// Proving key for the delegator vote proof.
pub static DELEGATOR_VOTE_PROOF_PROVING_KEY_OLD: Lazy<LazyProvingKeyOld> = Lazy::new(|| {
    let delegator_vote_proving_key = LazyProvingKeyOld::new(delegator_vote::PROVING_KEY_ID);

    delegator_vote_proving_key
        .try_load(include_bytes!("../../../crates/crypto/proof-params/src/gen/delegator_vote_pk.bin"))
        .expect("bundled proving key is valid");

    delegator_vote_proving_key
});

fn delegator_vote_verification_parameters() -> VerifyingKey<Bls12_377_05> {
    let vk_params = include_bytes!("../../../crates/crypto/proof-params/src/gen/delegator_vote_vk.param");
    VerifyingKey::deserialize_uncompressed_unchecked(&vk_params[..])
        .expect("can deserialize VerifyingKey")
}

/// Verification key for the delegator vote proof.
pub static DELEGATOR_VOTE_PROOF_VERIFICATION_KEY_OLD: Lazy<PreparedVerifyingKey<Bls12_377_05>> =
Lazy::new(|| delegator_vote_verification_parameters().into());

#[cfg(test)]
mod tests {
    use ark_ec::AffineRepr;
    use penumbra_proof_params::{generate_constraint_matrices, generate_test_parameters, DELEGATOR_VOTE_PROOF_PROVING_KEY, DELEGATOR_VOTE_PROOF_VERIFICATION_KEY};
    use penumbra_governance::DelegatorVoteCircuit;
    use super::*;

    #[test]
    fn test_generate_matrices() {
        let spent_circuit_matrix = generate_constraint_matrices::<SpendCircuit>();
        let output_circuit_matrix = generate_constraint_matrices::<OutputCircuit>();
        let swap_circuit_matrix = generate_constraint_matrices::<SwapCircuit>();
        let swap_claim_circuit_matrix = generate_constraint_matrices::<SwapClaimCircuit>();
        let convert_circuit_matrix = generate_constraint_matrices::<ConvertCircuit>();
        let nullifier_derivation_circuit_matrix = generate_constraint_matrices::<NullifierDerivationCircuit>();
        let delegator_vote_circuit_matrix = generate_constraint_matrices::<DelegatorVoteCircuit>();
    }

    #[test]
    fn test_generate_parameters() {
        let mut seed_phrase_randomness: [u8; 32] = [0u8; 32];
        let mut rng = ChaCha20Rng::from_seed(seed_phrase_randomness);

        let (pk, vk) = generate_test_parameters::<OutputCircuit>(&mut rng);
        let (pk, vk) = generate_test_parameters::<SwapCircuit>(&mut rng);
        let (pk, vk) = generate_test_parameters::<SwapClaimCircuit>(&mut rng);
        let (pk, vk) = generate_test_parameters::<ConvertCircuit>(&mut rng);
        let (pk, vk) = generate_test_parameters::<NullifierDerivationCircuit>(&mut rng);
    }

    #[test]
    fn test_proving_key_serialization() {
        // Delegator vote keys
        let pk = &*DELEGATOR_VOTE_PROOF_PROVING_KEY;
        let pk_old = &*DELEGATOR_VOTE_PROOF_PROVING_KEY_OLD;
        let vk = &*DELEGATOR_VOTE_PROOF_VERIFICATION_KEY;
        let vk_old = &*DELEGATOR_VOTE_PROOF_VERIFICATION_KEY_OLD;

        // Assert validity of prover keys
        for i in 0..pk.a_query.len() {
            if (pk.a_query[i].x() != None && pk.a_query[i].y() != None 
                && pk_old.a_query[i].x() != None && pk_old.a_query[i].y() != None) {
                assert_eq!(pk.a_query[i].x().unwrap().0.0, pk_old.a_query[i].x().unwrap().0.0);
                assert_eq!(pk.a_query[i].y().unwrap().0.0, pk_old.a_query[i].y().unwrap().0.0);
            }
        }
        for i in 0..pk.b_g1_query.len() {
            if (pk.b_g1_query[i].x() != None && pk.b_g1_query[i].y() != None 
                && pk_old.b_g1_query[i].x() != None && pk_old.b_g1_query[i].y() != None) {
                assert_eq!(pk.b_g1_query[i].x().unwrap().0.0, pk_old.b_g1_query[i].x().unwrap().0.0);
                assert_eq!(pk.b_g1_query[i].y().unwrap().0.0, pk_old.b_g1_query[i].y().unwrap().0.0);
            }
        }
        for i in 0..pk.b_g2_query.len() {
            if (pk.b_g2_query[i].x() != None && pk.b_g2_query[i].y() != None 
                && pk_old.b_g2_query[i].x() != None && pk_old.b_g2_query[i].y() != None) {
                assert_eq!(pk.b_g2_query[i].x().unwrap().c0.0.0, pk_old.b_g2_query[i].x().unwrap().c0.0.0);
                assert_eq!(pk.b_g2_query[i].x().unwrap().c1.0.0, pk_old.b_g2_query[i].x().unwrap().c1.0.0);
                assert_eq!(pk.b_g2_query[i].y().unwrap().c0.0.0, pk_old.b_g2_query[i].y().unwrap().c0.0.0);
                assert_eq!(pk.b_g2_query[i].y().unwrap().c1.0.0, pk_old.b_g2_query[i].y().unwrap().c1.0.0);
            }
        }
        assert_eq!(pk.beta_g1.x().unwrap().0.0, pk_old.beta_g1.x().unwrap().0.0);
        assert_eq!(pk.delta_g1.x().unwrap().0.0, pk_old.delta_g1.x().unwrap().0.0);
        for i in 0..pk.h_query.len() {
            if (pk.h_query[i].x() != None && pk.h_query[i].y() != None 
                && pk_old.h_query[i].x() != None && pk_old.h_query[i].y() != None) {
                assert_eq!(pk.h_query[i].x().unwrap().0.0, pk_old.h_query[i].x().unwrap().0.0);
                assert_eq!(pk.h_query[i].y().unwrap().0.0, pk_old.h_query[i].y().unwrap().0.0);
            }
        }
        for i in 0..pk.l_query.len() {
            if (pk.l_query[i].x() != None && pk.l_query[i].y() != None 
                && pk_old.l_query[i].x() != None && pk_old.l_query[i].y() != None) {
                assert_eq!(pk.l_query[i].x().unwrap().0.0, pk_old.l_query[i].x().unwrap().0.0);
                assert_eq!(pk.l_query[i].y().unwrap().0.0, pk_old.l_query[i].y().unwrap().0.0);
            }
        }

        // Assert validity of verifier keys    
        assert_eq!(vk.alpha_g1_beta_g2.c0.c0.c0.0.0, vk_old.alpha_g1_beta_g2.c0.c0.c0.0.0);
        assert_eq!(vk.alpha_g1_beta_g2.c0.c0.c1.0.0, vk_old.alpha_g1_beta_g2.c0.c0.c1.0.0);
        assert_eq!(vk.alpha_g1_beta_g2.c0.c1.c0.0.0, vk_old.alpha_g1_beta_g2.c0.c1.c0.0.0);
        assert_eq!(vk.alpha_g1_beta_g2.c0.c1.c1.0.0, vk_old.alpha_g1_beta_g2.c0.c1.c1.0.0);
        assert_eq!(vk.alpha_g1_beta_g2.c0.c2.c0.0.0, vk_old.alpha_g1_beta_g2.c0.c2.c0.0.0);
        assert_eq!(vk.alpha_g1_beta_g2.c0.c2.c1.0.0, vk_old.alpha_g1_beta_g2.c0.c2.c1.0.0);
        assert_eq!(vk.alpha_g1_beta_g2.c1.c0.c0.0.0, vk_old.alpha_g1_beta_g2.c1.c0.c0.0.0);
        assert_eq!(vk.alpha_g1_beta_g2.c1.c0.c1.0.0, vk_old.alpha_g1_beta_g2.c1.c0.c1.0.0);
        assert_eq!(vk.alpha_g1_beta_g2.c1.c1.c0.0.0, vk_old.alpha_g1_beta_g2.c1.c1.c0.0.0);
        assert_eq!(vk.alpha_g1_beta_g2.c1.c1.c1.0.0, vk_old.alpha_g1_beta_g2.c1.c1.c1.0.0);
        assert_eq!(vk.alpha_g1_beta_g2.c1.c2.c0.0.0, vk_old.alpha_g1_beta_g2.c1.c2.c0.0.0);
        assert_eq!(vk.alpha_g1_beta_g2.c1.c2.c1.0.0, vk_old.alpha_g1_beta_g2.c1.c2.c1.0.0);
        for i in 0..vk.delta_g2_neg_pc.ell_coeffs.len() {
            assert_eq!(vk.delta_g2_neg_pc.ell_coeffs[i].0.c0.0.0, vk_old.delta_g2_neg_pc.ell_coeffs[i].0.c0.0.0);
            assert_eq!(vk.delta_g2_neg_pc.ell_coeffs[i].0.c1.0.0, vk_old.delta_g2_neg_pc.ell_coeffs[i].0.c1.0.0);
            assert_eq!(vk.delta_g2_neg_pc.ell_coeffs[i].1.c0.0.0, vk_old.delta_g2_neg_pc.ell_coeffs[i].1.c0.0.0);
            assert_eq!(vk.delta_g2_neg_pc.ell_coeffs[i].1.c1.0.0, vk_old.delta_g2_neg_pc.ell_coeffs[i].1.c1.0.0);
            assert_eq!(vk.delta_g2_neg_pc.ell_coeffs[i].2.c0.0.0, vk_old.delta_g2_neg_pc.ell_coeffs[i].2.c0.0.0);
            assert_eq!(vk.delta_g2_neg_pc.ell_coeffs[i].2.c1.0.0, vk_old.delta_g2_neg_pc.ell_coeffs[i].2.c1.0.0);
        }
        assert_eq!(vk.delta_g2_neg_pc.infinity, vk_old.delta_g2_neg_pc.infinity);
        for i in 0..vk.gamma_g2_neg_pc.ell_coeffs.len() {
            assert_eq!(vk.gamma_g2_neg_pc.ell_coeffs[i].0.c0.0.0, vk_old.gamma_g2_neg_pc.ell_coeffs[i].0.c0.0.0);
            assert_eq!(vk.gamma_g2_neg_pc.ell_coeffs[i].0.c1.0.0, vk_old.gamma_g2_neg_pc.ell_coeffs[i].0.c1.0.0);
            assert_eq!(vk.gamma_g2_neg_pc.ell_coeffs[i].1.c0.0.0, vk_old.gamma_g2_neg_pc.ell_coeffs[i].1.c0.0.0);
            assert_eq!(vk.gamma_g2_neg_pc.ell_coeffs[i].1.c1.0.0, vk_old.gamma_g2_neg_pc.ell_coeffs[i].1.c1.0.0);
            assert_eq!(vk.gamma_g2_neg_pc.ell_coeffs[i].2.c0.0.0, vk_old.gamma_g2_neg_pc.ell_coeffs[i].2.c0.0.0);
            assert_eq!(vk.gamma_g2_neg_pc.ell_coeffs[i].2.c1.0.0, vk_old.gamma_g2_neg_pc.ell_coeffs[i].2.c1.0.0);
        }
    }
}