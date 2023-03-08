use ark_groth16::ProvingKey;
use ark_serialize::CanonicalDeserialize;

use decaf377::Bls12_377;

pub fn output_proving_parameters() -> ProvingKey<Bls12_377> {
    let pk_params = include_bytes!("gen/output_pk.bin");
    // If the system does not have Git LFS installed, then the files will
    // exist but they will be tiny pointers. We want to detect this and
    // panic if so, alerting the user that they should go and install Git LFS.
    if pk_params.len() < 500 {
        panic!("output proving key is too small; did you install Git LFS?")
    }
    // TODO: Instead of panicking, download here using the Git LFS pointer
    load_proving_parameters(pk_params)
}

pub fn spend_proving_parameters() -> ProvingKey<Bls12_377> {
    let pk_params = include_bytes!("gen/spend_pk.bin");
    // If the system does not have Git LFS installed, then the files will
    // exist but they will be tiny pointers. We want to detect this and
    // panic if so, alerting the user that they should go and install Git LFS.
    if pk_params.len() < 500 {
        panic!("spend proving key is too small; did you install Git LFS?")
    }
    // TODO: Instead of panicking, download here using the Git LFS pointer
    load_proving_parameters(pk_params)
}

/// Given a byte slice, deserialize it into a proving key.
pub fn load_proving_parameters(pk_params: &[u8]) -> ProvingKey<Bls12_377> {
    ProvingKey::deserialize_unchecked(pk_params).expect("can deserialize ProvingKey")
}
