use ark_groth16::{ProvingKey, VerifyingKey};
use decaf377::Bls12_377;

/// Must be implemented to generate proving and verification keys for a circuit.
pub trait ParameterSetup {
    fn generate_test_parameters() -> (ProvingKey<Bls12_377>, VerifyingKey<Bls12_377>);
}
