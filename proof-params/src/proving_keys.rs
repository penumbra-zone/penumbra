use ark_groth16::ProvingKey;
use ark_serialize::CanonicalDeserialize;

use decaf377::Bls12_377;
use penumbra_crypto::proofs::groth16::ProvingKeyExt;

pub fn output_proving_parameters() -> ProvingKey<Bls12_377> {
    let pk_params = include_bytes!("gen/output_pk.bin");
    load_proving_parameters(pk_params, crate::output::PROVING_KEY_ID)
}

pub fn spend_proving_parameters() -> ProvingKey<Bls12_377> {
    let pk_params = include_bytes!("gen/spend_pk.bin");
    load_proving_parameters(pk_params, crate::spend::PROVING_KEY_ID)
}

pub fn swap_proving_parameters() -> ProvingKey<Bls12_377> {
    let pk_params = include_bytes!("gen/swap_pk.bin");
    load_proving_parameters(pk_params, crate::swap::PROVING_KEY_ID)
}

/// Given a byte slice, deserialize it into a proving key.
pub fn load_proving_parameters(pk_params: &[u8], expected_id: &str) -> ProvingKey<Bls12_377> {
    // If the system does not have Git LFS installed, then the files will
    // exist but they will be tiny pointers. We want to detect this and
    // panic if so, alerting the user that they should go and install Git LFS.
    if pk_params.len() < 500 {
        panic!("proving key is too small; did you install Git LFS?")
    }
    // TODO: Instead of panicking, download here using the Git LFS pointer?
    let pk = ProvingKey::deserialize_unchecked(pk_params).expect("can deserialize ProvingKey");
    let pk_id = pk.debug_id();
    // Double-check that the ID of the proving key we loaded matches the hardcoded one,
    // in case there was some problem with git-lfs updating the file, or something.
    assert_eq!(
        expected_id, pk_id,
        "proving key ID mismatch: expected {}, loaded {}",
        expected_id, pk_id
    );
    pk
}
