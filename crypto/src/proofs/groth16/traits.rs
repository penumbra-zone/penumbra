use ark_groth16::{PreparedVerifyingKey, ProvingKey};
use ark_serialize::CanonicalSerialize;
use decaf377::Bls12_377;

/// Must be implemented to generate proving and verification keys for a circuit.
pub trait ParameterSetup {
    fn generate_test_parameters() -> (ProvingKey<Bls12_377>, PreparedVerifyingKey<Bls12_377>);
}

pub trait VerifyingKeyExt {
    fn debug_id(&self) -> String;
}

impl VerifyingKeyExt for PreparedVerifyingKey<Bls12_377> {
    fn debug_id(&self) -> String {
        let mut buf = Vec::new();
        self.vk.serialize(&mut buf).unwrap();
        use sha2::Digest;
        let hash = sha2::Sha256::digest(&buf);
        use bech32::ToBase32;
        bech32::encode("vk", hash.to_base32(), bech32::Variant::Bech32m).unwrap()
    }
}
