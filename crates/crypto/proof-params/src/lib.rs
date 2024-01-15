#![deny(clippy::unwrap_used)]
use anyhow::{bail, Result};
use ark_groth16::{PreparedVerifyingKey, ProvingKey, VerifyingKey};
use ark_serialize::CanonicalDeserialize;
use decaf377::Bls12_377;
use once_cell::sync::{Lazy, OnceCell};
use std::ops::Deref;

/// The length of our Groth16 proofs in bytes.
pub const GROTH16_PROOF_LENGTH_BYTES: usize = 192;

mod traits;

pub use traits::{
    generate_constraint_matrices, generate_prepared_test_parameters, generate_test_parameters,
    DummyWitness, ProvingKeyExt, VerifyingKeyExt,
};

/// A wrapper around a proving key that can be lazily loaded.
///
/// One instance of this struct is created for each proving key.
///
/// The behavior of those instances is controlled by the `bundled-proving-keys`
/// feature. When the feature is enabled, the proving key data is bundled into
/// the binary at compile time, and the proving key is loaded from the bundled
/// data on first use.  When the feature is not enabled, the proving key must be
/// loaded using `try_load` prior to its first use.
///
/// The `bundled-proving-keys` feature needs access to proving keys at build
/// time.  When pulling the crate as a dependency, these may not be available.
/// To address this, the `download-proving-keys` feature will download them from
/// the network at build time. All proving keys are checked against hardcoded hashes
/// to ensure they have not been tampered with.
#[derive(Debug, Default)]
pub struct LazyProvingKey {
    pk_id: &'static str,
    inner: OnceCell<ProvingKey<Bls12_377>>,
}

impl LazyProvingKey {
    // Not making this pub means only the statically defined proving keys can exist.
    fn new(pk_id: &'static str) -> Self {
        LazyProvingKey {
            pk_id,
            inner: OnceCell::new(),
        }
    }

    /// Attempt to load the proving key from the given bytes.
    ///
    /// The provided bytes are validated against a hardcoded hash of the expected proving key,
    /// so passing the wrong proving key will fail.
    ///
    /// If the proving key is already loaded, this method is a no-op.
    pub fn try_load(&self, bytes: &[u8]) -> Result<&ProvingKey<Bls12_377>> {
        self.inner.get_or_try_init(|| {
            let pk = ProvingKey::deserialize_uncompressed_unchecked(bytes)?;

            let pk_id = pk.debug_id();
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

    /// Attempt to load the proving key from the given bytes.
    ///
    /// This method bypasses the validation checks against the hardcoded
    /// hash of the expected proving key.
    pub fn try_load_unchecked(&self, bytes: &[u8]) -> Result<&ProvingKey<Bls12_377>> {
        self.inner.get_or_try_init(|| {
            let pk = ProvingKey::deserialize_uncompressed_unchecked(bytes)?;

            Ok(pk)
        })
    }
}

impl Deref for LazyProvingKey {
    type Target = ProvingKey<Bls12_377>;

    fn deref(&self) -> &Self::Target {
        self.inner.get().expect("Proving key cannot be loaded!")
    }
}

// Note: Conditionally load the proving key objects if the
// bundled-proving-keys is present.

/// Proving key for the spend proof.
pub static SPEND_PROOF_PROVING_KEY: Lazy<LazyProvingKey> = Lazy::new(|| {
    let spend_proving_key = LazyProvingKey::new(spend::PROVING_KEY_ID);

    #[cfg(feature = "bundled-proving-keys")]
    spend_proving_key
        .try_load(include_bytes!("gen/spend_pk.bin"))
        .expect("bundled proving key is valid");

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
    let output_proving_key = LazyProvingKey::new(output::PROVING_KEY_ID);

    #[cfg(feature = "bundled-proving-keys")]
    output_proving_key
        .try_load(include_bytes!("gen/output_pk.bin"))
        .expect("bundled proving key is valid");

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
    let swap_proving_key = LazyProvingKey::new(swap::PROVING_KEY_ID);

    #[cfg(feature = "bundled-proving-keys")]
    swap_proving_key
        .try_load(include_bytes!("gen/swap_pk.bin"))
        .expect("bundled proving key is valid");

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
    let swapclaim_proving_key = LazyProvingKey::new(swapclaim::PROVING_KEY_ID);

    #[cfg(feature = "bundled-proving-keys")]
    swapclaim_proving_key
        .try_load(include_bytes!("gen/swapclaim_pk.bin"))
        .expect("bundled proving key is valid");

    swapclaim_proving_key
});

/// Verification key for the swap claim proof.
pub static SWAPCLAIM_PROOF_VERIFICATION_KEY: Lazy<PreparedVerifyingKey<Bls12_377>> =
    Lazy::new(|| swapclaim_verification_parameters().into());

pub mod swapclaim {
    include!("gen/swapclaim_id.rs");
}

/// Proving key for the convert proof.
pub static CONVERT_PROOF_PROVING_KEY: Lazy<LazyProvingKey> = Lazy::new(|| {
    let convert_proving_key = LazyProvingKey::new(convert::PROVING_KEY_ID);

    #[cfg(feature = "bundled-proving-keys")]
    convert_proving_key
        .try_load(include_bytes!("gen/convert_pk.bin"))
        .expect("bundled proving key is valid");

    convert_proving_key
});

/// Verification key for the convert proof.
pub static CONVERT_PROOF_VERIFICATION_KEY: Lazy<PreparedVerifyingKey<Bls12_377>> =
    Lazy::new(|| convert_verification_parameters().into());

pub mod convert {
    include!("gen/convert_id.rs");
}

/// Proving key for the delegator vote proof.
pub static DELEGATOR_VOTE_PROOF_PROVING_KEY: Lazy<LazyProvingKey> = Lazy::new(|| {
    let delegator_vote_proving_key = LazyProvingKey::new(delegator_vote::PROVING_KEY_ID);

    #[cfg(feature = "bundled-proving-keys")]
    delegator_vote_proving_key
        .try_load(include_bytes!("gen/delegator_vote_pk.bin"))
        .expect("bundled proving key is valid");

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
    let nullifier_proving_key = LazyProvingKey::new(nullifier_derivation::PROVING_KEY_ID);

    #[cfg(feature = "bundled-proving-keys")]
    nullifier_proving_key
        .try_load(include_bytes!("gen/nullifier_derivation_pk.bin"))
        .expect("bundled proving key is valid");

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

fn convert_verification_parameters() -> VerifyingKey<Bls12_377> {
    let vk_params = include_bytes!("gen/convert_vk.param");
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
