use ark_groth16::{PreparedVerifyingKey, ProvingKey, VerifyingKey};
use ark_serialize::CanonicalSerialize;
use decaf377::Bls12_377;

/// Must be implemented to generate proving and verification keys for a circuit.
pub trait ParameterSetup {
    fn generate_test_parameters() -> (ProvingKey<Bls12_377>, VerifyingKey<Bls12_377>);

    fn generate_prepared_test_parameters(
    ) -> (ProvingKey<Bls12_377>, PreparedVerifyingKey<Bls12_377>) {
        let (pk, vk) = Self::generate_test_parameters();
        (pk, vk.into())
    }
}

pub trait VerifyingKeyExt {
    fn debug_id(&self) -> String;
}

impl VerifyingKeyExt for VerifyingKey<Bls12_377> {
    fn debug_id(&self) -> String {
        let mut buf = Vec::new();
        self.serialize_compressed(&mut buf).unwrap();
        use sha2::Digest;
        let hash = sha2::Sha256::digest(&buf);
        use bech32::ToBase32;
        bech32::encode("groth16vk", hash.to_base32(), bech32::Variant::Bech32m).unwrap()
    }
}

impl VerifyingKeyExt for PreparedVerifyingKey<Bls12_377> {
    fn debug_id(&self) -> String {
        self.vk.debug_id()
    }
}

pub trait ProvingKeyExt {
    fn debug_id(&self) -> String;
}

impl ProvingKeyExt for ProvingKey<Bls12_377> {
    fn debug_id(&self) -> String {
        let mut buf = Vec::new();
        self.serialize_compressed(&mut buf).unwrap();
        use sha2::Digest;
        let hash = sha2::Sha256::digest(&buf);
        use bech32::ToBase32;
        bech32::encode("groth16pk", hash.to_base32(), bech32::Variant::Bech32m).unwrap()
    }
}
