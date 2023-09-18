use ark_ec::pairing::Pairing;
use ark_groth16::{
    r1cs_to_qap::LibsnarkReduction, Groth16, PreparedVerifyingKey, ProvingKey, VerifyingKey,
};
use ark_relations::r1cs::{self, ConstraintMatrices, ConstraintSynthesizer};
use ark_serialize::CanonicalSerialize;
use ark_snark::SNARK;
use decaf377::Bls12_377;
use rand_core::CryptoRngCore;

/// This trait characterizes circuits which can generate constraints.
pub trait DummyWitness: ConstraintSynthesizer<<Bls12_377 as Pairing>::ScalarField> {
    /// This will create a circuit with dummy witness values, for constraint synthesis
    ///
    /// (The reason this is needed is because constraint synthesis encapsulates both the act
    /// of generating the constraints, but also that of providing the witness values when proving).
    /// ((For the record, I am not a fan of this)).
    fn with_dummy_witness() -> Self;
}

/// Generate constraint matrices from a circuit type.
///
/// This is useful because it provides a way to get the actual constraints
/// associated with some circuit, without actually generating a proving key via a trusted setup.
/// This is what you need for doing a setup ceremony, among other things.
pub fn generate_constraint_matrices<T: DummyWitness>(
) -> ConstraintMatrices<<Bls12_377 as Pairing>::ScalarField> {
    let circuit = T::with_dummy_witness();

    let cs = r1cs::ConstraintSystem::new_ref();
    cs.set_optimization_goal(r1cs::OptimizationGoal::Constraints);
    cs.set_mode(r1cs::SynthesisMode::Setup);
    // For why this is ok, see `generate_test_parameters`.
    circuit
        .generate_constraints(cs.clone())
        .expect("can generate constraints from circuit");
    cs.finalize();

    // I honestly don't know why this would fail.
    // But if it does, it's not at runtime in a node.
    cs.to_matrices()
        .expect("can convert R1CS constraints into matrices")
}

/// Generate parameters for proving and verifying, for *tests*.
///
/// These parameters should not be used for actual production code,
/// because the randomness may not have been securely destroyed.
pub fn generate_test_parameters<T: DummyWitness>(
    rng: &mut impl CryptoRngCore,
) -> (ProvingKey<Bls12_377>, VerifyingKey<Bls12_377>) {
    let circuit = T::with_dummy_witness();

    // Unwrapping here is ok because:
    // 1. This code is not run in node software at run time (or shouldn't be)
    // 2. If this fails, there's a bug in one of our circuits (which is bad)
    Groth16::<Bls12_377, LibsnarkReduction>::circuit_specific_setup(circuit, rng)
        .expect("can generate constraints from circuit")
}

/// A variant of `generate_test_parameters` which spits out a verifying key with some
/// precomputation.
pub fn generate_prepared_test_parameters<T: DummyWitness>(
    rng: &mut impl CryptoRngCore,
) -> (ProvingKey<Bls12_377>, PreparedVerifyingKey<Bls12_377>) {
    let (pk, vk) = generate_test_parameters::<T>(rng);
    (pk, vk.into())
}

pub trait VerifyingKeyExt {
    fn debug_id(&self) -> String;
}

impl VerifyingKeyExt for VerifyingKey<Bls12_377> {
    fn debug_id(&self) -> String {
        let mut buf = Vec::new();
        self.serialize_compressed(&mut buf)
            .expect("can serialize vk");
        use sha2::Digest;
        let hash = sha2::Sha256::digest(&buf);
        use bech32::ToBase32;
        bech32::encode("groth16vk", hash.to_base32(), bech32::Variant::Bech32m)
            .expect("can encode vk as bech32")
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
        self.serialize_compressed(&mut buf)
            .expect("can serialize pk");
        use sha2::Digest;
        let hash = sha2::Sha256::digest(&buf);
        use bech32::ToBase32;
        bech32::encode("groth16pk", hash.to_base32(), bech32::Variant::Bech32m)
            .expect("can encode pk as bech32")
    }
}
