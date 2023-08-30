// Todo: prune public interface once we know exactly what's needed.
mod dlog;
mod group;
pub mod log;
mod phase1;
mod phase2;

use ark_poly::EvaluationDomain;
use ark_poly::Radix2EvaluationDomain;
use ark_relations::r1cs::ConstraintMatrices;

pub use phase1::CRSElements as Phase1CRSElements;
pub use phase1::Contribution as Phase1Contribution;
pub use phase1::RawContribution as Phase1RawContribution;

pub use phase2::CRSElements as Phase2CRSElements;
pub use phase2::Contribution as Phase2Contribution;
pub use phase2::RawContribution as Phase2RawContribution;

use anyhow::{anyhow, Result};

// Compute the degree associated with a given circuit.
//
// This degree can then be used for both phases.
pub fn circuit_degree(circuit: &ConstraintMatrices<group::F>) -> Result<usize> {
    let circuit_size = circuit.num_constraints + circuit.num_instance_variables;
    Radix2EvaluationDomain::<group::F>::compute_size_of_domain(circuit_size)
        .ok_or(anyhow!("Circuit of size {} is too large", circuit_size))
}
