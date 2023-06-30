use ark_ff::UniformRand;
use ark_relations::r1cs::{
    ConstraintSynthesizer, ConstraintSystem, OptimizationGoal, SynthesisMode,
};
use decaf377::Fr;
use penumbra_crypto::{
    asset, balance, rdsa,
    stake::{IdentityKey, Penalty, UnbondingToken},
    Amount, Fq,
};
use penumbra_proof_params::UNDELEGATECLAIM_PROOF_PROVING_KEY;
use penumbra_stake::{UndelegateClaimCircuit, UndelegateClaimProof};

use criterion::{criterion_group, criterion_main, Criterion};
use rand_core::OsRng;

fn prove(
    r: Fq,
    s: Fq,
    unbonding_amount: Amount,
    balance_blinding: Fr,
    balance_commitment: balance::Commitment,
    unbonding_id: asset::Id,
    penalty: Penalty,
) {
    let _proof = UndelegateClaimProof::prove(
        r,
        s,
        &UNDELEGATECLAIM_PROOF_PROVING_KEY,
        unbonding_amount,
        balance_blinding,
        balance_commitment,
        unbonding_id,
        penalty,
    )
    .expect("can generate proof");
}

fn undelegate_claim_proving_time(c: &mut Criterion) {
    let sk = rdsa::SigningKey::new_from_field(Fr::from(1u32));
    let validator_identity = IdentityKey((&sk).into());
    let unbonding_amount = Amount::from(1u32);

    let balance_blinding = Fr::from(0u32);
    let start_epoch_index = 1;
    let unbonding_token = UnbondingToken::new(validator_identity, start_epoch_index);
    let unbonding_id = unbonding_token.id();
    let penalty = Penalty(1u64);
    let balance = penalty.balance_for_claim(unbonding_id, unbonding_amount);
    let balance_commitment = balance.commit(balance_blinding);

    let r = Fq::rand(&mut OsRng);
    let s = Fq::rand(&mut OsRng);

    c.bench_function("undelegate claim proving", |b| {
        b.iter(|| {
            prove(
                r,
                s,
                unbonding_amount,
                balance_blinding,
                balance_commitment,
                unbonding_id,
                penalty,
            )
        })
    });

    // Also print out the number of constraints.
    let circuit = UndelegateClaimCircuit::new(
        unbonding_amount,
        balance_blinding,
        balance_commitment,
        unbonding_id,
        penalty,
    );

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

criterion_group!(benches, undelegate_claim_proving_time);
criterion_main!(benches);
