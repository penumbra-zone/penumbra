use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use rand_core::OsRng;

use decaf377_fmd as fmd;

fn detect_clues(dk: &fmd::DetectionKey, clues: &[fmd::Clue]) -> usize {
    clues.iter().filter(|clue| dk.examine(clue)).count()
}

fn create_clues(ck: &fmd::ExpandedClueKey, precision: usize) -> Vec<fmd::Clue> {
    (0..1024)
        .map(|_| ck.create_clue(precision, OsRng).unwrap())
        .collect::<Vec<_>>()
}

fn bench(c: &mut Criterion) {
    let dk = fmd::DetectionKey::new(OsRng);
    let ck = dk.clue_key().expand().unwrap();

    let clues = vec![
        (4, create_clues(&ck, 4)),
        (5, create_clues(&ck, 5)),
        (6, create_clues(&ck, 6)),
        (7, create_clues(&ck, 7)),
        (8, create_clues(&ck, 8)),
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
