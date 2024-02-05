#![deny(clippy::unwrap_used)]
// Requires nightly.
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

#[cfg(test)]
mod tests {
    use penumbra_proof_params::{SPEND_PROOF_PROVING_KEY, generate_constraint_matrices, generate_test_parameters};
    use anyhow::Result;
    use decaf377::Bls12_377;
    use decaf377_v05::Bls12_377 as Bls12_377_05;
    use once_cell::sync::{Lazy, OnceCell};
    use penumbra_shielded_pool::{SpendCircuit, OutputCircuit, ConvertCircuit, NullifierDerivationCircuit};
    use penumbra_dex::{swap::proof::SwapCircuit, swap_claim::proof::SwapClaimCircuit};
    use ark_groth16::{ProvingKey, VerifyingKey};
    use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
    use std::ops::Deref;
    use rand_chacha::{ChaCha20Rng, rand_core::SeedableRng};

    /// Proving key associated with decaf377 v0.8 backend
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
                println!("pk_id is: {:?}", pk_id);
                if pk_id != self.pk_id {
                    println!("proving key ID mismatch!");
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
                .expect("can serialize vk");
            use sha2::Digest;
            let hash = sha2::Sha256::digest(&buf);
            use bech32::ToBase32;
            bech32::encode("groth16vk", hash.to_base32(), bech32::Variant::Bech32m)
                .expect("can encode vk as bech32")
        }
    }

    pub static SPEND_PROOF_PROVING_KEY_OLD: Lazy<LazyProvingKeyOld> = Lazy::new(|| {
        let spend_proving_key = LazyProvingKeyOld::new(spend::PROVING_KEY_ID);

        spend_proving_key
            .try_load(include_bytes!("../../../crates/crypto/proof-params/src/gen/spend_pk.bin"))
            .expect("bundled proving key is valid");

        spend_proving_key
    });

    pub mod spend {
        include!("../../../crates/crypto/proof-params/src/gen/spend_id.rs");
    }

    #[test]
    fn test_generate_matrices() {
        let spent_circuit_matrix = generate_constraint_matrices::<SpendCircuit>();
        let output_circuit_matrix = generate_constraint_matrices::<OutputCircuit>();
        let swap_circuit_matrix = generate_constraint_matrices::<SwapCircuit>();
        let swap_claim_circuit_matrix = generate_constraint_matrices::<SwapClaimCircuit>();
        let convert_circuit_matrix = generate_constraint_matrices::<ConvertCircuit>();
        let nullifier_derivation_circuit_matrix = generate_constraint_matrices::<NullifierDerivationCircuit>();
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
        let pk = &*SPEND_PROOF_PROVING_KEY;
        let pk_old = &*SPEND_PROOF_PROVING_KEY_OLD;
    }
}