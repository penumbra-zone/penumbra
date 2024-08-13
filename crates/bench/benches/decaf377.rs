use criterion::{black_box, criterion_group, criterion_main, Criterion};
use decaf377::{Fp, Fq};

// ------------------------------------------------ BLS12-377 Base Field Modulus Fp in Decaf377 ------------------------------------------------

fn generate_fp() -> Fp {
    let mut rng = rand::thread_rng();
    Fp::rand(&mut rng)
}

fn bench_base_field_addition(c: &mut Criterion) {
    let mut x = generate_fp();
    let y = generate_fp();

    c.bench_function("decaf377: fp field addition", |b| {
        b.iter(|| {
            for _ in 0..100000 {
                x = black_box(x) + black_box(y);
            }
        })
    });
}

fn bench_base_field_subtraction(c: &mut Criterion) {
    let mut x = generate_fp();
    let y = generate_fp();

    c.bench_function("decaf377: fp field subtraction", |b| {
        b.iter(|| {
            for _ in 0..100000 {
                x = black_box(x) - black_box(y);
            }
        })
    });
}

fn bench_base_field_multiplication(c: &mut Criterion) {
    let mut x = generate_fp();
    let y = generate_fp();

    c.bench_function("decaf377: fp field multiplication", |b| {
        b.iter(|| {
            for _ in 0..100000 {
                x = black_box(x) * black_box(y);
            }
        })
    });
}

fn bench_base_field_negation(c: &mut Criterion) {
    let mut x = generate_fp();
    let y = generate_fp();

    c.bench_function("decaf377: fp field negation", |b| {
        b.iter(|| {
            for _ in 0..100000 {
                x = black_box(x) + black_box(-y);
            }
        })
    });
}

fn bench_base_field_square(c: &mut Criterion) {
    let mut x = generate_fp();

    c.bench_function("decaf377: fp field squaring", |b| {
        b.iter(|| {
            for _ in 0..100000 {
                x = black_box(x).square()
            }
        })
    });
}

fn bench_base_field_inverse(c: &mut Criterion) {
    let mut x = generate_fp();

    c.bench_function("decaf377: fp field inverse", |b| {
        b.iter(|| {
            for _ in 0..100000 {
                x = black_box(x).inverse().expect("inverse")
            }
        })
    });
}

// ------------------------------------------------ BLS12-377 Scalar Field Modulus Fq in Decaf377 ------------------------------------------------

fn generate_fq() -> Fq {
    let mut rng = rand::thread_rng();
    Fq::rand(&mut rng)
}

fn bench_scalar_field_addition(c: &mut Criterion) {
    let mut x = generate_fq();
    let y = generate_fq();

    c.bench_function("decaf377: fq field addition", |b| {
        b.iter(|| {
            for _ in 0..100000 {
                x = black_box(x) + black_box(y);
            }
        })
    });
}

fn bench_scalar_field_subtraction(c: &mut Criterion) {
    let mut x = generate_fq();
    let y = generate_fq();

    c.bench_function("decaf377: fq field subtraction", |b| {
        b.iter(|| {
            for _ in 0..100000 {
                x = black_box(x) - black_box(y);
            }
        })
    });
}

fn bench_scalar_field_multiplication(c: &mut Criterion) {
    let mut x = generate_fq();
    let y = generate_fq();

    c.bench_function("decaf377: fq field multiplication", |b| {
        b.iter(|| {
            for _ in 0..100000 {
                x = black_box(x) * black_box(y);
            }
        })
    });
}

fn bench_scalar_field_negation(c: &mut Criterion) {
    let mut x = generate_fq();
    let y = generate_fq();

    c.bench_function("decaf377: fq field negation", |b| {
        b.iter(|| {
            for _ in 0..100000 {
                x = black_box(x) + black_box(-y);
            }
        })
    });
}

fn bench_scalar_field_square(c: &mut Criterion) {
    let mut x = generate_fq();

    c.bench_function("decaf377: fq field squaring", |b| {
        b.iter(|| {
            for _ in 0..100000 {
                x = black_box(x).square()
            }
        })
    });
}

fn bench_scalar_field_inverse(c: &mut Criterion) {
    let mut x = generate_fq();

    c.bench_function("decaf377: fq field inverse", |b| {
        b.iter(|| {
            for _ in 0..100000 {
                x = black_box(x).inverse().expect("inverse")
            }
        })
    });
}

criterion_group!(
    benches,
    bench_base_field_addition,
    bench_base_field_subtraction,
    bench_base_field_multiplication,
    bench_base_field_negation,
    bench_base_field_square,
    bench_base_field_inverse,
    bench_scalar_field_addition,
    bench_scalar_field_subtraction,
    bench_scalar_field_multiplication,
    bench_scalar_field_negation,
    bench_scalar_field_square,
    bench_scalar_field_inverse
);
criterion_main!(benches);
