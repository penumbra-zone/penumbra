use ark_ec::pairing::Pairing;
use ark_relations::r1cs::ConstraintMatrices;
use criterion::{criterion_group, criterion_main, Criterion};
use decaf377::Bls12_377;
use rand_core::OsRng;

use penumbra_dex::swap_claim::proof::SwapClaimCircuit;
use penumbra_proof_params::generate_constraint_matrices;
use penumbra_proof_setup::{
    circuit_degree,
    log::{ContributionHash, Hashable},
    transition, ExtraTransitionInformation, Phase2CRSElements, Phase2Contribution,
    Phase2RawContribution, {Phase1CRSElements, Phase1Contribution, Phase1RawContribution},
};

fn run_phase1(parent: ContributionHash, old: &Phase1CRSElements) -> Phase1Contribution {
    Phase1Contribution::make(&mut OsRng, parent, old)
}

fn check_phase1(contribution: &Phase1RawContribution, parent: &Phase1CRSElements) -> bool {
    let validated_contribution = contribution.validate(&mut OsRng);
    if validated_contribution.is_none() {
        return false;
    }
    let validated_contribution =
        validated_contribution.expect("validated contribution must be set");
    validated_contribution.is_linked_to(parent)
}

fn run_phase_transition(
    phase1: &Phase1CRSElements,
    circuit: &ConstraintMatrices<<Bls12_377 as Pairing>::ScalarField>,
) -> anyhow::Result<(ExtraTransitionInformation, Phase2CRSElements)> {
    transition(phase1, circuit)
}

fn run_phase2(parent: ContributionHash, old: &Phase2CRSElements) -> Phase2Contribution {
    Phase2Contribution::make(&mut OsRng, parent, old)
}

fn check_phase2(
    contribution: Phase2RawContribution,
    root: &Phase2CRSElements,
    parent: &Phase2CRSElements,
) -> bool {
    let validated_contribution = contribution.validate(&mut OsRng, root);
    if validated_contribution.is_none() {
        return false;
    }
    let validated_contribution =
        validated_contribution.expect("validated contribution must be set");
    validated_contribution.is_linked_to(parent)
}

fn benchmarks(c: &mut Criterion) {
    // Generate contribution for degree = 37,061, which gets rounded up to next power of 2.
    // (size of largest proof)
    let matrices = generate_constraint_matrices::<SwapClaimCircuit>();
    let d = circuit_degree(&matrices).expect("failed to calculate circuit degree");

    let root = Phase1CRSElements::root(d);
    let root_hash = root.hash();

    let phase1out = run_phase1(root_hash, &root);
    c.bench_function("phase 1", |b| b.iter(|| run_phase1(root_hash, &root)));
    c.bench_function("phase 1 check", |b| {
        b.iter(|| check_phase1(&phase1out.clone().into(), &root))
    });

    let (_, phase2root) = run_phase_transition(&phase1out.new_elements, &matrices)
        .expect("failed to perform transition");
    let phase2root_hash = phase2root.hash();
    c.bench_function("phase transition", |b| {
        b.iter(|| run_phase_transition(&phase1out.new_elements, &matrices))
    });

    let phase2out = run_phase2(phase2root_hash, &phase2root);
    c.bench_function("phase 2", |b| {
        b.iter(|| run_phase2(phase2root_hash, &phase2root))
    });
    c.bench_function("phase 2 check", |b| {
        b.iter(|| check_phase2(phase2out.clone().into(), &phase2root, &phase2root))
    });
}

criterion_group! {
  name = benches;
  config = Criterion::default().sample_size(10);
  targets = benchmarks
}
criterion_main!(benches);
