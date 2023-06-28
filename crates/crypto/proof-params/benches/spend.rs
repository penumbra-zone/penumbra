use std::str::FromStr;

use ark_ff::UniformRand;
use ark_relations::r1cs::{
    ConstraintSynthesizer, ConstraintSystem, OptimizationGoal, SynthesisMode,
};
use decaf377::Fr;
use penumbra_crypto::{
    balance,
    keys::{NullifierKey, SeedPhrase, SpendKey},
    proofs::groth16::{SpendCircuit, SpendProof},
    rdsa::{SpendAuth, VerificationKey},
    Fq, Note, Nullifier, Value,
};
use penumbra_proof_params::SPEND_PROOF_PROVING_KEY;
use penumbra_tct as tct;

use criterion::{criterion_group, criterion_main, Criterion};
use rand_core::OsRng;

fn prove(
    r: Fq,
    s: Fq,
    state_commitment_proof: tct::Proof,
    note: Note,
    v_blinding: Fr,
    spend_auth_randomizer: Fr,
    ak: VerificationKey<SpendAuth>,
    nk: NullifierKey,
    anchor: tct::Root,
    balance_commitment: balance::Commitment,
    nullifier: Nullifier,
    rk: VerificationKey<SpendAuth>,
) {
    let _proof = SpendProof::prove(
        r,
        s,
        &SPEND_PROOF_PROVING_KEY,
        state_commitment_proof,
        note,
        v_blinding,
        spend_auth_randomizer,
        ak,
        nk,
        anchor,
        balance_commitment,
        nullifier,
        rk,
    )
    .expect("can create proof");
}

fn spend_proving_time(c: &mut Criterion) {
    let value_to_send = Value::from_str("1upenumbra").expect("valid value");

    let seed_phrase = SeedPhrase::generate(OsRng);
    let sk_sender = SpendKey::from_seed_phrase(seed_phrase, 0);
    let fvk_sender = sk_sender.full_viewing_key();
    let ivk_sender = fvk_sender.incoming();
    let (sender, _dtk_d) = ivk_sender.payment_address(0u32.into());

    let note = Note::generate(&mut OsRng, &sender, value_to_send);
    let note_commitment = note.commit();
    let spend_auth_randomizer = Fr::from(0i32);
    let rsk = sk_sender.spend_auth_key().randomize(&spend_auth_randomizer);
    let nk = *sk_sender.nullifier_key();
    let ak: VerificationKey<SpendAuth> = sk_sender.spend_auth_key().into();
    let mut sct = tct::Tree::new();

    sct.insert(tct::Witness::Keep, note_commitment).unwrap();
    let anchor = sct.root();
    let state_commitment_proof = sct.witness(note_commitment).unwrap();
    let v_blinding = Fr::from(0i32);
    let balance_commitment = value_to_send.commit(v_blinding);
    let rk: VerificationKey<SpendAuth> = rsk.into();
    let nf = nk.derive_nullifier(state_commitment_proof.position(), &note_commitment);

    let r = Fq::rand(&mut OsRng);
    let s = Fq::rand(&mut OsRng);

    c.bench_function("spend proving", |b| {
        b.iter(|| {
            prove(
                r,
                s,
                state_commitment_proof.clone(),
                note.clone(),
                v_blinding,
                spend_auth_randomizer,
                ak,
                nk,
                anchor,
                balance_commitment,
                nf,
                rk,
            )
        })
    });

    // Also print out the number of constraints.
    let circuit = SpendCircuit::new(
        state_commitment_proof,
        note,
        v_blinding,
        spend_auth_randomizer,
        ak,
        nk,
        anchor,
        balance_commitment,
        nf,
        rk,
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

criterion_group!(benches, spend_proving_time);
criterion_main!(benches);
