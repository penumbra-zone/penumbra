use decaf377::Fp;
use criterion::{criterion_group, criterion_main, Criterion, black_box};

fn generate_fp() -> Fp {
    let mut rng = rand::thread_rng();
    Fp::rand(&mut rng)
}

fn bench_field_addition(c: &mut Criterion) {
    let mut x = generate_fp();
    let y = generate_fp();
    
    c.bench_function("decaf377: field addition", |b| b.iter(|| {
        for _ in 0..10000 {
            x = black_box(x) + black_box(y);
        }
    }));
}

fn bench_field_subtraction(c: &mut Criterion) {
    let mut x = generate_fp();
    let y = generate_fp();
    
    c.bench_function("decaf377: field subtraction", |b| b.iter(|| {
        for _ in 0..10000 {
            x = black_box(x) - black_box(y);
        }
    }));
}

fn bench_field_multiplication(c: &mut Criterion) {
    let mut x = generate_fp();
    let y = generate_fp();
    
    c.bench_function("decaf377: field multiplication", |b| b.iter(|| {
        for _ in 0..10000 {
            x = black_box(x) * black_box(y);
        }
    }));
}

fn bench_field_negation(c: &mut Criterion) {
    let mut x = generate_fp();
    let y = generate_fp();
    
    c.bench_function("decaf377: field negation", |b| b.iter(|| {
        for _ in 0..10000 {
            x = black_box(x) + black_box(-y);
        }
    }));
}

fn bench_field_square(c: &mut Criterion) {
    let mut x = generate_fp();
    
    c.bench_function("decaf377: field squaring", |b| b.iter(|| {
        for _ in 0..10000 {
            x = black_box(x).square()
        }
    }));
}

fn bench_field_inverse(c: &mut Criterion) {
    let mut x = generate_fp();

    c.bench_function("decaf377: field inverse", |b| b.iter(|| {
        for _ in 0..10000 {
            x = black_box(x).inverse().expect("inverse")
        }
    }));
}

criterion_group!(benches, bench_field_addition, bench_field_subtraction, bench_field_multiplication, bench_field_negation, bench_field_square, bench_field_inverse);
criterion_main!(benches);