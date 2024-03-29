use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use fmd::Precision;
use rand_core::OsRng;

use decaf377_fmd as fmd;

fn detect_clues(dk: &fmd::DetectionKey, clues: &[fmd::Clue]) -> usize {
    clues.iter().filter(|clue| dk.examine(clue)).count()
}

fn create_clues(ck: &fmd::ExpandedClueKey, precision: Precision) -> Vec<fmd::Clue> {
    (0..1024)
        .map(|_| {
            ck.create_clue(precision, OsRng)
                .expect("precision must not be too large")
        })
        .collect::<Vec<_>>()
}

fn bench(c: &mut Criterion) {
    let dk = fmd::DetectionKey::new(OsRng);
    let ck = dk
        .clue_key()
        .expand()
        .expect("clue key bytes must be valid");

    let clues = vec![
        (4, create_clues(&ck, 4.try_into().unwrap())),
        (5, create_clues(&ck, 5.try_into().unwrap())),
        (6, create_clues(&ck, 6.try_into().unwrap())),
        (7, create_clues(&ck, 7.try_into().unwrap())),
        (8, create_clues(&ck, 8.try_into().unwrap())),
    ];

    let mut group = c.benchmark_group("fmd-detection");
    // We're already benchmarking batches of clues, so we don't need as many runs
    group.sample_size(10);
    for (precision_bits, clues) in clues {
        group.throughput(Throughput::Elements(clues.len() as u64));

        group.bench_function(
            format!("detect_clues_precision_{precision_bits}").as_str(),
            |b| b.iter(|| detect_clues(&dk, &clues)),
        );
    }
    group.finish();
}

criterion_group!(benches, bench);
criterion_main!(benches);
