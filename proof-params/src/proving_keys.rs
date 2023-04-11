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

pub fn undelegateclaim_proving_parameters() -> ProvingKey<Bls12_377> {
    let pk_params = include_bytes!("gen/undelegateclaim_pk.bin");
    load_proving_parameters(pk_params, crate::undelegateclaim::PROVING_KEY_ID)
}

/// Given a byte slice, deserialize it into a proving key.
pub fn load_proving_parameters(bytes: &[u8], expected_id: &str) -> ProvingKey<Bls12_377> {
    let pk = ProvingKey::deserialize_unchecked(bytes).expect("can deserialize ProvingKey");
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
