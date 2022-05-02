use ark_poly_commit::kzg10::UniversalParams;
use ark_serialize::CanonicalDeserialize;
use once_cell::sync::Lazy;
use std::fs;

use decaf377::Bls12_377;

/// Universal parameters for the KZG10 polynomial commitment scheme.
///
/// These parameters were generated using `KZG10<Bls12_377, DensePolynomial<Fq>>`
/// from the `ark-poly-commit` crate.
pub static KZG10_PP: Lazy<UniversalParams<Bls12_377>> = Lazy::new(|| {
    UniversalParams::<Bls12_377>::deserialize(
        fs::File::open("./proofs/pcs").expect("Unable to open file"),
    )
    .expect("can parse setup file")
});
