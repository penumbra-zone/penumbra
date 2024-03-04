use std::str::FromStr;

use ark_relations::r1cs::{
    ConstraintSynthesizer, ConstraintSystem, OptimizationGoal, SynthesisMode,
};
use decaf377::{Fq, Fr};
use decaf377_rdsa::{SpendAuth, VerificationKey};
use penumbra_asset::Value;
use penumbra_governance::{
    DelegatorVoteCircuit, DelegatorVoteProof, DelegatorVoteProofPrivate, DelegatorVoteProofPublic,
};
use penumbra_keys::keys::{Bip44Path, SeedPhrase, SpendKey};
use penumbra_proof_params::{DummyWitness, DELEGATOR_VOTE_PROOF_PROVING_KEY};
use penumbra_sct::Nullifier;
use penumbra_shielded_pool::Note;
use penumbra_tct as tct;

use criterion::{criterion_group, criterion_main, Criterion};
use rand_core::OsRng;

#[allow(clippy::too_many_arguments)]
fn prove(r: Fq, s: Fq, public: DelegatorVoteProofPublic, private: DelegatorVoteProofPrivate) {
    let _proof =
        DelegatorVoteProof::prove(r, s, &DELEGATOR_VOTE_PROOF_PROVING_KEY, public, private)
            .expect("can create proof");
}

fn delegator_vote_proving_time(c: &mut Criterion) {
    let value_to_send = Value::from_str("1upenumbra").expect("valid value");

    let seed_phrase = SeedPhrase::generate(OsRng);
    let sk_sender = SpendKey::from_seed_phrase_bip44(seed_phrase, &Bip44Path::new(0));
    let fvk_sender = sk_sender.full_viewing_key();
    let ivk_sender = fvk_sender.incoming();
    let (sender, _dtk_d) = ivk_sender.payment_address(0u32.into());

    let note = Note::generate(&mut OsRng, &sender, value_to_send);
    let note_commitment = note.commit();
    let spend_auth_randomizer = Fr::from(0u64);
    let rsk = sk_sender.spend_auth_key().randomize(&spend_auth_randomizer);
    let nk = *sk_sender.nullifier_key();
    let ak: VerificationKey<SpendAuth> = sk_sender.spend_auth_key().into();
    let mut sct = tct::Tree::new();

    sct.insert(tct::Witness::Keep, note_commitment).unwrap();
    let anchor = sct.root();
    let state_commitment_proof = sct.witness(note_commitment).unwrap();
    let v_blinding = Fr::from(0u32);
    let balance_commitment = value_to_send.commit(v_blinding);
    let rk: VerificationKey<SpendAuth> = rsk.into();
    let nullifier = Nullifier::derive(&nk, state_commitment_proof.position(), &note_commitment);
    sct.end_epoch().unwrap();

    let first_note_commitment = Note::generate(&mut OsRng, &sender, value_to_send).commit();
    sct.insert(tct::Witness::Keep, first_note_commitment)
        .unwrap();
    let start_position = sct.witness(first_note_commitment).unwrap().position();

    let r = Fq::rand(&mut OsRng);
    let s = Fq::rand(&mut OsRng);
    let public = DelegatorVoteProofPublic {
        anchor,
        balance_commitment,
        nullifier,
        rk,
        start_position,
    };
    let private = DelegatorVoteProofPrivate {
        state_commitment_proof,
        note,
        v_blinding,
        spend_auth_randomizer,
        ak,
        nk,
    };

    c.bench_function("delegator proving", |b| {
        b.iter(|| prove(r, s, public.clone(), private.clone()))
    });

    // Also print out the number of constraints.
    let circuit = DelegatorVoteCircuit::with_dummy_witness();

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

criterion_group!(benches, delegator_vote_proving_time);
criterion_main!(benches);
