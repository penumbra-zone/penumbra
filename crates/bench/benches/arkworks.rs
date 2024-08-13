use ark_bls12_377::{Fq as ArkFq, Fr as ArkFr};
use ark_ff::fields::models::fp::{Fp256, Fp384};
use ark_ff::{Field, UniformRand};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

// ------------------------------------------------ BLS12-377 Base Field Modulus Fq in Arkworks ------------------------------------------------

fn generate_fq_arkworks() -> ArkFq {
    let mut rng = rand::thread_rng();
    ArkFq::rand(&mut rng)
}

fn bench_base_field_addition(c: &mut Criterion) {
    let mut x = generate_fq_arkworks();
    let y = generate_fq_arkworks();

    c.bench_function("arkworks:fq field addition", |b| {
        b.iter(|| {
            for _ in 0..100000 {
                x = black_box(x) + black_box(y);
            }
        })
    });
}

fn bench_base_field_subtraction(c: &mut Criterion) {
    let mut x = generate_fq_arkworks();
    let y = generate_fq_arkworks();

    c.bench_function("arkworks: fq field subtraction", |b| {
        b.iter(|| {
            for _ in 0..100000 {
                x = black_box(x) - black_box(y);
            }
        })
    });
}

fn bench_base_field_mutliplication(c: &mut Criterion) {
    let mut x = generate_fq_arkworks();
    let y = generate_fq_arkworks();

    c.bench_function("arkworks: fq field multiplication", |b| {
        b.iter(|| {
            for _ in 0..100000 {
                x = black_box(x) * black_box(y);
            }
        })
    });
}

fn bench_base_field_negation(c: &mut Criterion) {
    let mut x = generate_fq_arkworks();
    let y = generate_fq_arkworks();

    c.bench_function("arkworks: fq field negation", |b| {
        b.iter(|| {
            for _ in 0..100000 {
                x = black_box(x) + black_box(-y);
            }
        })
    });
}

fn bench_base_field_square(c: &mut Criterion) {
    let mut x = generate_fq_arkworks();
    c.bench_function("arkworks: fq field squaring", |b| {
        b.iter(|| {
            for _ in 0..100000 {
                x = Fp384::square(&x)
            }
        })
    });
}

fn bench_base_field_inverse(c: &mut Criterion) {
    let mut x = generate_fq_arkworks();

    c.bench_function("arkworks: fq field inverse", |b| {
        b.iter(|| {
            for _ in 0..100000 {
                x = Fp384::inverse(&x).expect("inverse")
            }
        })
    });
}

// ------------------------------------------------ BLS12-377 Scalar Field Modulus Fr in Arkworks ------------------------------------------------

fn generate_fr_arkworks() -> ArkFr {
    let mut rng = rand::thread_rng();
    ArkFr::rand(&mut rng)
}

fn bench_scalar_field_addition(c: &mut Criterion) {
    let mut x = generate_fr_arkworks();
    let y = generate_fr_arkworks();

    c.bench_function("arkworks: fr field addition", |b| {
        b.iter(|| {
            for _ in 0..100000 {
                x = black_box(x) + black_box(y);
            }
        })
    });
}

fn bench_scalar_field_subtraction(c: &mut Criterion) {
    let mut x = generate_fr_arkworks();
    let y = generate_fr_arkworks();

    c.bench_function("arkworks: fr field subtraction", |b| {
        b.iter(|| {
            for _ in 0..100000 {
                x = black_box(x) - black_box(y);
            }
        })
    });
}

fn bench_scalar_field_mutliplication(c: &mut Criterion) {
    let mut x = generate_fr_arkworks();
    let y = generate_fr_arkworks();

    c.bench_function("arkworks: fr field multiplication", |b| {
        b.iter(|| {
            for _ in 0..100000 {
                x = black_box(x) * black_box(y);
            }
        })
    });
}

fn bench_scalar_field_negation(c: &mut Criterion) {
    let mut x = generate_fr_arkworks();
    let y = generate_fr_arkworks();

    c.bench_function("arkworks: fr field negation", |b| {
        b.iter(|| {
            for _ in 0..100000 {
                x = black_box(x) + black_box(-y);
            }
        })
    });
}

fn bench_scalar_field_square(c: &mut Criterion) {
    let mut x = generate_fr_arkworks();
    c.bench_function("arkworks: fr field squaring", |b| {
        b.iter(|| {
            for _ in 0..100000 {
                x = Fp256::square(&x)
            }
        })
    });
}

fn bench_scalar_field_inverse(c: &mut Criterion) {
    let mut x = generate_fr_arkworks();

    c.bench_function("arkworks: fr field inverse", |b| {
        b.iter(|| {
            for _ in 0..100000 {
                x = Fp256::inverse(&x).expect("inverse")
            }
        })
    });
}

criterion_group!(
    benches,
    bench_base_field_addition,
    bench_base_field_subtraction,
    bench_base_field_mutliplication,
    bench_base_field_negation,
    bench_base_field_square,
    bench_base_field_inverse,
    bench_scalar_field_addition,
    bench_scalar_field_subtraction,
    bench_scalar_field_mutliplication,
    bench_scalar_field_negation,
    bench_scalar_field_square,
    bench_scalar_field_inverse
);
criterion_main!(benches);
