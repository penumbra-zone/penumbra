use std::str::FromStr;

use ark_relations::r1cs::{
    ConstraintSynthesizer, ConstraintSystem, OptimizationGoal, SynthesisMode,
};
use decaf377::Fq;
use penumbra_crypto::{
    keys::{NullifierKey, SeedPhrase, SpendKey},
    proofs::groth16::{NullifierDerivationCircuit, NullifierDerivationProof},
    Note, Nullifier, Rseed, Value,
};
use penumbra_proof_params::NULLIFIER_DERIVATION_PROOF_PROVING_KEY;
use penumbra_tct as tct;

use criterion::{criterion_group, criterion_main, Criterion};
use rand_core::OsRng;

fn prove(position: tct::Position, note: Note, nk: NullifierKey, nullifier: Nullifier) {
    let _proof = NullifierDerivationProof::prove(
        &mut OsRng,
        &NULLIFIER_DERIVATION_PROOF_PROVING_KEY,
        position,
        note,
        nk,
        nullifier,
    )
    .expect("Can generate proof");
}

fn nullifier_derivation_proving_time(c: &mut Criterion) {
    let seed_phrase = SeedPhrase::generate(OsRng);
    let sk_sender = SpendKey::from_seed_phrase(seed_phrase, 0);
    let fvk_sender = sk_sender.full_viewing_key();
    let ivk_sender = fvk_sender.incoming();
    let (address, _dtk_d) = ivk_sender.payment_address(0u32.into());

    let nk = *sk_sender.nullifier_key();
    let note = Note::from_parts(
        address,
        Value::from_str("1upenumbra").expect("valid value"),
        Rseed([1u8; 32]),
    )
    .expect("can make a note");
    let nullifier = Nullifier(Fq::from(1));
    let mut sct = tct::Tree::new();
    let note_commitment = note.commit();
    sct.insert(tct::Witness::Keep, note_commitment).unwrap();
    let state_commitment_proof = sct.witness(note_commitment).unwrap();
    let position = state_commitment_proof.position();

    c.bench_function("nullifier derivation proving time", |b| {
        b.iter(|| prove(position, note.clone(), nk, nullifier))
    });

    // Also print out the number of constraints.
    let circuit = NullifierDerivationCircuit::new(nk, note, nullifier, position);

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

criterion_group!(benches, nullifier_derivation_proving_time);
criterion_main!(benches);
