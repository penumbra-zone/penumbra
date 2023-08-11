use criterion::{criterion_group, criterion_main, Criterion};
use rand_core::OsRng;

use penumbra_proof_setup::{
    log::{ContributionHash, Hashable},
    phase1::{CRSElements, Contribution},
};

fn run_phase1_prove(parent: ContributionHash, old: &CRSElements) -> Contribution {
    Contribution::make(&mut OsRng, parent, old)
}

fn run_phase1_verify() {}

fn phase1_benchmarks(c: &mut Criterion) {
    // Generate contribution for degree = 37,655
    // (size of largest proof)
    let d = 37_655;
    let root = CRSElements::root(d);
    let root_hash = root.hash();

    c.bench_function("phase 1 prove", |b| {
        b.iter(|| run_phase1_prove(root_hash, &root))
    });

    // c.bench_function("phase 1 verify", |b| b.iter(|| run_phase1_verify()));
}

criterion_group! {
  name = benches;
  config = Criterion::default().sample_size(10);
  targets = phase1_benchmarks
}
criterion_main!(benches);
