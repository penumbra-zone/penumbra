use ark_bls12_377::Fq as ArkFp;
use ark_ff::{Field, UniformRand};
use ark_ff::fields::models::fp::Fp384;
use criterion::{criterion_group, criterion_main, Criterion, black_box};

fn generate_fp_arkworks() -> ArkFp{
    let mut rng = rand::thread_rng();
    ArkFp::rand(&mut rng)
}

fn bench_field_addition(c: &mut Criterion) {
    let mut x = generate_fp_arkworks();
    let y = generate_fp_arkworks();
    
    c.bench_function("arkworks: field addition", |b| b.iter(|| {
        for _ in 0..10000 {
            x = black_box(x) + black_box(y);
        }
    }));
}

fn bench_field_subtraction(c: &mut Criterion) {
    let mut x = generate_fp_arkworks();
    let y = generate_fp_arkworks();
    
    c.bench_function("arkworks: field subtraction", |b| b.iter(|| {
        for _ in 0..10000 {
            x = black_box(x) - black_box(y);
        }
    }));
}

fn bench_field_mutliplication(c: &mut Criterion) {
    let mut x = generate_fp_arkworks();
    let y = generate_fp_arkworks();
    
    c.bench_function("arkworks: field multiplication", |b| b.iter(|| {
        for _ in 0..10000 {
            x = black_box(x) * black_box(y);
        }
    }));
}

fn bench_field_negation(c: &mut Criterion) {
    let mut x = generate_fp_arkworks();
    let y = generate_fp_arkworks();
    
    c.bench_function("arkworks: field negation", |b| b.iter(|| {
        for _ in 0..10000 {
            x = black_box(x) + black_box(-y);
        }
    }));
}

fn bench_field_square(c: &mut Criterion) {
    let mut x = generate_fp_arkworks();
    c.bench_function("arkworks: field squaring", |b| b.iter(|| {
        for _ in 0..10000 {
            x = Fp384::square(&x)
        }
    }));
}

fn bench_field_inverse(c: &mut Criterion) {
    let mut x = generate_fp_arkworks();

    c.bench_function("arkworks: field inverse", |b| b.iter(|| {
        for _ in 0..10000 {
            x = Fp384::inverse(&x).expect("inverse")
        }
    }));
}

criterion_group!(benches, bench_field_addition, bench_field_subtraction, bench_field_mutliplication, bench_field_negation, bench_field_square, bench_field_inverse);
criterion_main!(benches);