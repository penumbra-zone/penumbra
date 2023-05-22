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

#[cfg(feature = "proving-keys")]
/// Proving key for the swap claim proof.
pub static SWAPCLAIM_PROOF_PROVING_KEY: Lazy<ProvingKey<Bls12_377>> =
    Lazy::new(proving_keys::swapclaim_proving_parameters);

/// Verification key for the swap claim proof.
pub static SWAPCLAIM_PROOF_VERIFICATION_KEY: Lazy<PreparedVerifyingKey<Bls12_377>> =
    Lazy::new(|| swapclaim_verification_parameters().into());

pub mod swapclaim {
    include!("gen/swapclaim_id.rs");
}

#[cfg(feature = "proving-keys")]
/// Proving key for the undelegateclaim proof.
pub static UNDELEGATECLAIM_PROOF_PROVING_KEY: Lazy<ProvingKey<Bls12_377>> =
    Lazy::new(proving_keys::undelegateclaim_proving_parameters);

/// Verification key for the undelegateclaim proof.
pub static UNDELEGATECLAIM_PROOF_VERIFICATION_KEY: Lazy<PreparedVerifyingKey<Bls12_377>> =
    Lazy::new(|| undelegateclaim_verification_parameters().into());

pub mod undelegateclaim {
    include!("gen/undelegateclaim_id.rs");
}

#[cfg(feature = "proving-keys")]
/// Proving key for the delegator vote proof.
pub static DELEGATOR_VOTE_PROOF_PROVING_KEY: Lazy<ProvingKey<Bls12_377>> =
    Lazy::new(proving_keys::delegator_vote_proving_parameters);

/// Verification key for the delegator vote proof.
pub static DELEGATOR_VOTE_PROOF_VERIFICATION_KEY: Lazy<PreparedVerifyingKey<Bls12_377>> =
    Lazy::new(|| delegator_vote_verification_parameters().into());

pub mod delegator_vote {
    include!("gen/delegator_vote_id.rs");
}

#[cfg(feature = "proving-keys")]
/// Proving key for the nullifier derivation proof.
pub static NULLIFIER_DERIVATION_PROOF_PROVING_KEY: Lazy<ProvingKey<Bls12_377>> =
    Lazy::new(proving_keys::nullifier_derivation_proving_parameters);

/// Verification key for the delegator vote proof.
pub static NULLIFIER_DERIVATION_PROOF_VERIFICATION_KEY: Lazy<PreparedVerifyingKey<Bls12_377>> =
    Lazy::new(|| nullifier_derivation_verification_parameters().into());

pub mod nullifier_derivation {
    include!("gen/nullifier_derivation_id.rs");
}

// Note: Here we are using `CanonicalDeserialize::deserialize_uncompressed_unchecked` as the
// parameters are being loaded from a trusted source (our source code).
// TODO: Migrate to `CanonicalDeserialize::deserialize_compressed_unchecked` to save space.

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
