use ark_relations::r1cs::{
    ConstraintSynthesizer, ConstraintSystem, OptimizationGoal, SynthesisMode,
};
use decaf377::{Fq, Fr};
use penumbra_asset::{Balance, Value, STAKING_TOKEN_ASSET_ID};
use penumbra_num::{fixpoint::U128x128, Amount};
use penumbra_proof_params::{DummyWitness, CONVERT_PROOF_PROVING_KEY};
use penumbra_shielded_pool::{
    ConvertCircuit, ConvertProof, ConvertProofPrivate, ConvertProofPublic,
};

use criterion::{criterion_group, criterion_main, Criterion};
use rand_core::OsRng;

fn prove(r: Fq, s: Fq, public: ConvertProofPublic, private: ConvertProofPrivate) {
    let _proof = ConvertProof::prove(r, s, &CONVERT_PROOF_PROVING_KEY, public, private)
        .expect("can generate proof");
}

fn dummy_instance() -> (ConvertProofPublic, ConvertProofPrivate) {
    let amount = Amount::from(1u64);
    let balance_blinding = Fr::from(1u64);
    let from = *STAKING_TOKEN_ASSET_ID;
    let to = *STAKING_TOKEN_ASSET_ID;
    let rate = U128x128::from(1u64);
    let balance = Balance::from(Value {
        asset_id: to,
        amount,
    }) - Balance::from(Value {
        asset_id: from,
        amount,
    });
    let balance_commitment = balance.commit(balance_blinding);
    (
        ConvertProofPublic {
            from,
            to,
            rate,
            balance_commitment,
        },
        ConvertProofPrivate {
            amount,
            balance_blinding,
        },
    )
}

fn convert_proving_time(c: &mut Criterion) {
    let (public, private) = dummy_instance();

    let r = Fq::rand(&mut OsRng);
    let s = Fq::rand(&mut OsRng);

    c.bench_function("convert proving", |b| {
        b.iter(|| prove(r, s, public.clone(), private.clone()))
    });

    // Also print out the number of constraints.
    let circuit = ConvertCircuit::with_dummy_witness();

    let cs = ConstraintSystem::new_ref();
    cs.set_optimization_goal(OptimizationGoal::Constraints);
    cs.set_mode(SynthesisMode::Setup);

    circuit
        .generate_constraints(cs.clone())
        .expect("can generate constraints");
    cs.finalize();
    let num_constraints = cs.num_constraints();
    println!("Number of constraints: {}", num_constraints);
}

criterion_group!(benches, convert_proving_time);
criterion_main!(benches);
