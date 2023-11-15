#![deny(clippy::unwrap_used)]
use ark_groth16::{PreparedVerifyingKey, ProvingKey, VerifyingKey};
use ark_serialize::CanonicalDeserialize;
use decaf377::Bls12_377;
use once_cell::sync::{Lazy, OnceCell};
use std::ops::Deref;
use lazy_static::lazy_static;

/// The length of our Groth16 proofs in bytes.
pub const GROTH16_PROOF_LENGTH_BYTES: usize = 192;

mod traits;

pub use traits::{
    generate_constraint_matrices, generate_prepared_test_parameters, generate_test_parameters,
    DummyWitness, ProvingKeyExt, VerifyingKeyExt,
};

#[cfg(feature = "proving-keys")]
mod proving_keys;

#[cfg(feature = "proving-keys")]
#[derive(Debug, Default)]
pub struct LazyProvingKey {
    inner: OnceCell<ProvingKey<Bls12_377>>,
}

#[cfg(feature = "proving-keys")]
impl LazyProvingKey {
    // Lazily construct proving key
    pub fn new() -> Self {
        LazyProvingKey { 
            inner: OnceCell::new(),
        }
    }

    pub fn load(&self, key: &str) -> ProvingKey<Bls12_377> {
        self.inner.get_or_init(|| match key {
            "spend_key" => proving_keys::spend_proving_parameters(),
            _ => todo!()
        }).clone()
    }
}

#[cfg(feature = "proving-keys")]
impl Deref for LazyProvingKey {
    type Target = ProvingKey<Bls12_377>;

    fn deref(&self) -> &Self::Target {
        self.inner.get().expect("Proving key cannot be loaded!")
    }
}

/// Note: Conditionally load the proving key objects in the crate
/// unless the target is wasm. 

/// Proving key for the spend proof.
pub static SPEND_PROOF_PROVING_KEY: Lazy<LazyProvingKey> = Lazy::new(|| {
    let mut spend_proving_key = LazyProvingKey::new();

    #[cfg(all(feature = "proving-keys", not(target_arch = "wasm32")))]
    spend_proving_key.inner.get_or_init(proving_keys::spend_proving_parameters);

    spend_proving_key
});

/// Verification key for the spend proof.
pub static SPEND_PROOF_VERIFICATION_KEY: Lazy<PreparedVerifyingKey<Bls12_377>> =
    Lazy::new(|| spend_verification_parameters().into());

pub mod spend {
    include!("gen/spend_id.rs");
}

/// Proving key for the output proof.
pub static OUTPUT_PROOF_PROVING_KEY: Lazy<LazyProvingKey> = Lazy::new(|| {
    let mut output_proving_key = LazyProvingKey::new();

    #[cfg(all(feature = "proving-keys", not(target_arch = "wasm32")))]
    output_proving_key.inner.get_or_init(proving_keys::output_proving_parameters);

    output_proving_key
});

/// Verification key for the output proof.
pub static OUTPUT_PROOF_VERIFICATION_KEY: Lazy<PreparedVerifyingKey<Bls12_377>> =
    Lazy::new(|| output_verification_parameters().into());

pub mod output {
    include!("gen/output_id.rs");
}

/// Proving key for the swap proof.
pub static SWAP_PROOF_PROVING_KEY: Lazy<LazyProvingKey> = Lazy::new(|| {
    let mut swap_proving_key = LazyProvingKey::new();

    #[cfg(all(feature = "proving-keys", not(target_arch = "wasm32")))]
    swap_proving_key.inner.get_or_init(proving_keys::swap_proving_parameters);

    swap_proving_key
});

/// Verification key for the swap proof.
pub static SWAP_PROOF_VERIFICATION_KEY: Lazy<PreparedVerifyingKey<Bls12_377>> =
    Lazy::new(|| swap_verification_parameters().into());

pub mod swap {
    include!("gen/swap_id.rs");
}

/// Proving key for the swap claim proof.
pub static SWAPCLAIM_PROOF_PROVING_KEY: Lazy<LazyProvingKey> = Lazy::new(|| {
    let mut swap_claim_proving_key = LazyProvingKey::new();

    #[cfg(all(feature = "proving-keys", not(target_arch = "wasm32")))]
    swap_claim_proving_key.inner.get_or_init(proving_keys::swapclaim_proving_parameters);

    swap_claim_proving_key
});

/// Verification key for the swap claim proof.
pub static SWAPCLAIM_PROOF_VERIFICATION_KEY: Lazy<PreparedVerifyingKey<Bls12_377>> =
    Lazy::new(|| swapclaim_verification_parameters().into());

pub mod swapclaim {
    include!("gen/swapclaim_id.rs");
}

/// Proving key for the undelegateclaim proof.
pub static UNDELEGATECLAIM_PROOF_PROVING_KEY: Lazy<LazyProvingKey> = Lazy::new(|| {
    let mut undelegate_claim_proving_key = LazyProvingKey::new();

    #[cfg(all(feature = "proving-keys", not(target_arch = "wasm32")))]
    undelegate_claim_proving_key.inner.get_or_init(proving_keys::undelegateclaim_proving_parameters);

    undelegate_claim_proving_key
});

/// Verification key for the undelegateclaim proof.
pub static UNDELEGATECLAIM_PROOF_VERIFICATION_KEY: Lazy<PreparedVerifyingKey<Bls12_377>> =
    Lazy::new(|| undelegateclaim_verification_parameters().into());

pub mod undelegateclaim {
    include!("gen/undelegateclaim_id.rs");
}

/// Proving key for the delegator vote proof.
pub static DELEGATOR_VOTE_PROOF_PROVING_KEY: Lazy<LazyProvingKey> = Lazy::new(|| {
    let mut delegator_vote_proving_key = LazyProvingKey::new();

    #[cfg(all(feature = "proving-keys", not(target_arch = "wasm32")))]
    delegator_vote_proving_key.inner.get_or_init(proving_keys::delegator_vote_proving_parameters);

    delegator_vote_proving_key
});

/// Verification key for the delegator vote proof.
pub static DELEGATOR_VOTE_PROOF_VERIFICATION_KEY: Lazy<PreparedVerifyingKey<Bls12_377>> =
    Lazy::new(|| delegator_vote_verification_parameters().into());

pub mod delegator_vote {
    include!("gen/delegator_vote_id.rs");
}

/// Proving key for the nullifier derivation proof.
pub static NULLIFIER_DERIVATION_PROOF_PROVING_KEY: Lazy<LazyProvingKey> = Lazy::new(|| {
    let mut nullifier_proving_key = LazyProvingKey::new();

    #[cfg(all(feature = "proving-keys", not(target_arch = "wasm32")))]
    nullifier_proving_key.inner.get_or_init(proving_keys::nullifier_derivation_proving_parameters);

    nullifier_proving_key
});

/// Verification key for the delegator vote proof.
pub static NULLIFIER_DERIVATION_PROOF_VERIFICATION_KEY: Lazy<PreparedVerifyingKey<Bls12_377>> =
    Lazy::new(|| nullifier_derivation_verification_parameters().into());

pub mod nullifier_derivation {
    include!("gen/nullifier_derivation_id.rs");
}

// Note: Here we are using `CanonicalDeserialize::deserialize_uncompressed_unchecked` as the
// parameters are being loaded from a trusted source (our source code).

fn spend_verification_parameters() -> VerifyingKey<Bls12_377> {
    let vk_params = include_bytes!("gen/spend_vk.param");
    VerifyingKey::deserialize_uncompressed_unchecked(&vk_params[..])
        .expect("can deserialize VerifyingKey")
}

fn output_verification_parameters() -> VerifyingKey<Bls12_377> {
    let vk_params = include_bytes!("gen/output_vk.param");
    VerifyingKey::deserialize_uncompressed_unchecked(&vk_params[..])
        .expect("can deserialize VerifyingKey")
}

fn swap_verification_parameters() -> VerifyingKey<Bls12_377> {
    let vk_params = include_bytes!("gen/swap_vk.param");
    VerifyingKey::deserialize_uncompressed_unchecked(&vk_params[..])
        .expect("can deserialize VerifyingKey")
}

fn swapclaim_verification_parameters() -> VerifyingKey<Bls12_377> {
    let vk_params = include_bytes!("gen/swapclaim_vk.param");
    VerifyingKey::deserialize_uncompressed_unchecked(&vk_params[..])
        .expect("can deserialize VerifyingKey")
}

fn undelegateclaim_verification_parameters() -> VerifyingKey<Bls12_377> {
    let vk_params = include_bytes!("gen/undelegateclaim_vk.param");
    VerifyingKey::deserialize_uncompressed_unchecked(&vk_params[..])
        .expect("can deserialize VerifyingKey")
}

fn delegator_vote_verification_parameters() -> VerifyingKey<Bls12_377> {
    let vk_params = include_bytes!("gen/delegator_vote_vk.param");
    VerifyingKey::deserialize_uncompressed_unchecked(&vk_params[..])
        .expect("can deserialize VerifyingKey")
}

fn nullifier_derivation_verification_parameters() -> VerifyingKey<Bls12_377> {
    let vk_params = include_bytes!("gen/nullifier_derivation_vk.param");
    VerifyingKey::deserialize_uncompressed_unchecked(&vk_params[..])
        .expect("can deserialize VerifyingKey")
}