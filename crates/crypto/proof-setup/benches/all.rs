use criterion::{criterion_group, criterion_main, Criterion};
use penumbra_sdk_proof_setup::all::{
    transition, AllExtraTransitionInformation, Phase1CeremonyCRS, Phase1CeremonyContribution,
    Phase1RawCeremonyContribution, Phase2CeremonyCRS, Phase2CeremonyContribution,
    Phase2RawCeremonyContribution,
};

fn run_phase1(old: &Phase1CeremonyCRS) -> Phase1CeremonyContribution {
    Phase1CeremonyContribution::make(old)
}

fn check_phase1(contribution: Phase1RawCeremonyContribution, parent: &Phase1CeremonyCRS) -> bool {
    let validated_contribution = contribution.validate();
    if validated_contribution.is_none() {
        return false;
    }
    let validated_contribution =
        validated_contribution.expect("validated contribution must be set");
    validated_contribution.is_linked_to(parent)
}

fn run_phase_transition(
    phase1: &Phase1CeremonyCRS,
) -> anyhow::Result<(AllExtraTransitionInformation, Phase2CeremonyCRS)> {
    transition(phase1)
}

fn run_phase2(old: &Phase2CeremonyCRS) -> Phase2CeremonyContribution {
    Phase2CeremonyContribution::make(old)
}

fn check_phase2(
    contribution: Phase2RawCeremonyContribution,
    root: &Phase2CeremonyCRS,
    parent: &Phase2CeremonyCRS,
) -> bool {
    let validated_contribution = contribution.validate(root);
    if validated_contribution.is_none() {
        return false;
    }
    let validated_contribution =
        validated_contribution.expect("validated contribution must be set");
    validated_contribution.is_linked_to(parent)
}

fn benchmarks(c: &mut Criterion) {
    let root = Phase1CeremonyCRS::root().unwrap();
    let phase1out = run_phase1(&root);
    c.bench_function("phase 1", |b| b.iter(|| run_phase1(&root)));
    c.bench_function("phase 1 check", |b| {
        b.iter(|| check_phase1(phase1out.clone().into(), &root))
    });

    let (_, phase2root) =
        run_phase_transition(&phase1out.new_elements()).expect("failed to perform transition");
    c.bench_function("phase transition", |b| {
        b.iter(|| run_phase_transition(&phase1out.new_elements()))
    });

    let phase2out = run_phase2(&phase2root);
    c.bench_function("phase 2", |b| b.iter(|| run_phase2(&phase2root)));
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
