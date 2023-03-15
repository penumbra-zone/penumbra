use ark_groth16::{PreparedVerifyingKey, ProvingKey, VerifyingKey};
use ark_serialize::CanonicalDeserialize;
use decaf377::Bls12_377;
use once_cell::sync::Lazy;

#[cfg(feature = "proving-keys")]
mod proving_keys;

/// Proving key for the spend proof.
#[cfg(feature = "proving-keys")]
pub static SPEND_PROOF_PROVING_KEY: Lazy<ProvingKey<Bls12_377>> =
    Lazy::new(proving_keys::spend_proving_parameters);

/// Verifying key for the spend proof.
pub static SPEND_PROOF_VERIFICATION_KEY: Lazy<PreparedVerifyingKey<Bls12_377>> =
    Lazy::new(|| spend_verification_parameters().into());

pub mod spend {
    include!("gen/spend_id.rs");
}

/// Proving key for the output proof.
#[cfg(feature = "proving-keys")]
pub static OUTPUT_PROOF_PROVING_KEY: Lazy<ProvingKey<Bls12_377>> =
    Lazy::new(proving_keys::output_proving_parameters);

/// Proving key for the spend proof.
pub static OUTPUT_PROOF_VERIFICATION_KEY: Lazy<PreparedVerifyingKey<Bls12_377>> =
    Lazy::new(|| output_verification_parameters().into());

pub mod output {
    include!("gen/output_id.rs");
}

#[cfg(feature = "proving-keys")]
/// Proving key for the swap proof.
pub static SWAP_PROOF_PROVING_KEY: Lazy<ProvingKey<Bls12_377>> =
    Lazy::new(proving_keys::swap_proving_parameters);

/// Verification key for the swap proof.
pub static SWAP_PROOF_VERIFICATION_KEY: Lazy<PreparedVerifyingKey<Bls12_377>> =
    Lazy::new(|| swap_verification_parameters().into());

pub mod swap {
    include!("gen/swap_id.rs");
}

// Note: Here we are using `CanonicalDeserialize::deserialize_unchecked` as the
// parameters are being loaded from a trusted source (our source code).

fn spend_verification_parameters() -> VerifyingKey<Bls12_377> {
    let vk_params = include_bytes!("gen/spend_vk.param");
    VerifyingKey::deserialize_unchecked(&vk_params[..]).expect("can deserialize VerifyingKey")
}

fn output_verification_parameters() -> VerifyingKey<Bls12_377> {
    let vk_params = include_bytes!("gen/output_vk.param");
    VerifyingKey::deserialize_unchecked(&vk_params[..]).expect("can deserialize VerifyingKey")
}

fn swap_verification_parameters() -> VerifyingKey<Bls12_377> {
    let vk_params = include_bytes!("gen/swap_vk.param");
    VerifyingKey::deserialize_unchecked(&vk_params[..]).expect("can deserialize VerifyingKey")
}
