use ark_groth16::{PreparedVerifyingKey, ProvingKey, VerifyingKey};
use ark_serialize::CanonicalDeserialize;
use decaf377::Bls12_377;
use once_cell::sync::Lazy;

/// Proving key for the spend proof.
pub static SPEND_PROOF_PROVING_KEY: Lazy<ProvingKey<Bls12_377>> =
    Lazy::new(spend_proving_parameters);

/// Verifying key for the spend proof.
pub static SPEND_PROOF_VERIFICATION_KEY: Lazy<PreparedVerifyingKey<Bls12_377>> =
    Lazy::new(|| spend_verification_parameters().into());

/// Proving key for the output proof.
pub static OUTPUT_PROOF_PROVING_KEY: Lazy<ProvingKey<Bls12_377>> =
    Lazy::new(output_proving_parameters);

/// Proving key for the spend proof.
pub static OUTPUT_PROOF_VERIFICATION_KEY: Lazy<PreparedVerifyingKey<Bls12_377>> =
    Lazy::new(|| output_verification_parameters().into());

// Note: Here we are using `CanonicalDeserialize::deserialize_unchecked` as the
// parameters are being loaded from a trusted source (our source code).

fn spend_proving_parameters() -> ProvingKey<Bls12_377> {
    let pk_params = include_bytes!("gen/spend_pk.bin");
    ProvingKey::deserialize_unchecked(&pk_params[..]).expect("can deserialize ProvingKey")
}

fn spend_verification_parameters() -> VerifyingKey<Bls12_377> {
    let vk_params = include_bytes!("gen/spend_vk.bin");
    VerifyingKey::deserialize_unchecked(&vk_params[..]).expect("can deserialize VerifyingKey")
}

fn output_proving_parameters() -> ProvingKey<Bls12_377> {
    let pk_params = include_bytes!("gen/output_pk.bin");
    ProvingKey::deserialize_unchecked(&pk_params[..]).expect("can deserialize ProvingKey")
}

fn output_verification_parameters() -> VerifyingKey<Bls12_377> {
    let vk_params = include_bytes!("gen/output_vk.bin");
    VerifyingKey::deserialize_unchecked(&vk_params[..]).expect("can deserialize VerifyingKey")
}
