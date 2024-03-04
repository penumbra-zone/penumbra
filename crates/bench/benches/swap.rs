use ark_relations::r1cs::{
    ConstraintSynthesizer, ConstraintSystem, OptimizationGoal, SynthesisMode,
};
use decaf377::{Fq, Fr};
use penumbra_asset::{asset, Balance, Value};
use penumbra_dex::{
    swap::proof::{SwapCircuit, SwapProof, SwapProofPublic},
    swap::{proof::SwapProofPrivate, SwapPlaintext},
    TradingPair,
};
use penumbra_fee::Fee;
use penumbra_keys::keys::{Bip44Path, SeedPhrase, SpendKey};
use penumbra_num::Amount;
use penumbra_proof_params::{DummyWitness, SWAP_PROOF_PROVING_KEY};

use criterion::{criterion_group, criterion_main, Criterion};
use rand_core::OsRng;

fn prove(r: Fq, s: Fq, public: SwapProofPublic, private: SwapProofPrivate) {
    let _proof = SwapProof::prove(r, s, &SWAP_PROOF_PROVING_KEY, public, private)
        .expect("can generate proof");
}

fn swap_proving_time(c: &mut Criterion) {
    let seed_phrase = SeedPhrase::generate(OsRng);
    let sk_recipient = SpendKey::from_seed_phrase_bip44(seed_phrase, &Bip44Path::new(0));
    let fvk_recipient = sk_recipient.full_viewing_key();
    let ivk_recipient = fvk_recipient.incoming();
    let (claim_address, _dtk_d) = ivk_recipient.payment_address(0u32.into());

    let gm = asset::Cache::with_known_assets().get_unit("gm").unwrap();
    let gn = asset::Cache::with_known_assets().get_unit("gn").unwrap();
    let trading_pair = TradingPair::new(gm.id(), gn.id());

    let delta_1 = Amount::from(1u64);
    let delta_2 = Amount::from(0u64);
    let fee = Fee::default();

    let swap_plaintext = SwapPlaintext::new(
        &mut OsRng,
        trading_pair,
        delta_1,
        delta_2,
        fee,
        claim_address,
    );
    let fee_blinding = Fr::from(0u64);
    let fee_commitment = swap_plaintext.claim_fee.commit(fee_blinding);
    let swap_commitment = swap_plaintext.swap_commitment();

    let value_1 = Value {
        amount: swap_plaintext.delta_1_i,
        asset_id: swap_plaintext.trading_pair.asset_1(),
    };
    let value_2 = Value {
        amount: swap_plaintext.delta_2_i,
        asset_id: swap_plaintext.trading_pair.asset_2(),
    };
    let value_fee = Value {
        amount: swap_plaintext.claim_fee.amount(),
        asset_id: swap_plaintext.claim_fee.asset_id(),
    };
    let mut balance = Balance::default();
    balance -= value_1;
    balance -= value_2;
    balance -= value_fee;
    let balance_commitment = balance.commit(Fr::from(0u64));

    let public = SwapProofPublic {
        balance_commitment,
        swap_commitment,
        fee_commitment,
    };
    let private = SwapProofPrivate {
        fee_blinding,
        swap_plaintext,
    };

    let r = Fq::rand(&mut OsRng);
    let s = Fq::rand(&mut OsRng);

    c.bench_function("swap proving", |b| {
        b.iter(|| prove(r, s, public.clone(), private.clone()))
    });

    // Also print out the number of constraints.
    let circuit = SwapCircuit::with_dummy_witness();

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

criterion_group!(benches, swap_proving_time);
criterion_main!(benches);
