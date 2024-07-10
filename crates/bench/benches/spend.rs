use std::str::FromStr;

use ark_relations::r1cs::{
    ConstraintSynthesizer, ConstraintSystem, OptimizationGoal, SynthesisMode,
};
use decaf377::{Fq, Fr};
use decaf377_rdsa::{SpendAuth, VerificationKey};
use penumbra_asset::Value;
use penumbra_keys::keys::{Bip44Path, SeedPhrase, SpendKey};
use penumbra_proof_params::{DummyWitness, SPEND_PROOF_PROVING_KEY};
use penumbra_sct::Nullifier;
use penumbra_shielded_pool::{Note, SpendCircuit, SpendProof, SpendProofPrivate, SpendProofPublic};
use penumbra_tct as tct;

use criterion::{criterion_group, criterion_main, Criterion};
use rand_core::OsRng;

#[allow(clippy::too_many_arguments)]
fn prove(r: Fq, s: Fq, public: SpendProofPublic, private: SpendProofPrivate) {
    let _proof = SpendProof::prove(r, s, &SPEND_PROOF_PROVING_KEY, public, private)
        .expect("can create proof");
}

fn spend_proving_time(c: &mut Criterion) {
    let value_to_send = Value::from_str("1upenumbra").expect("valid value");

    let seed_phrase = SeedPhrase::generate(OsRng);
    let sk_sender = SpendKey::from_seed_phrase_bip44(seed_phrase, &Bip44Path::new(0));
    let fvk_sender = sk_sender.full_viewing_key();
    let ivk_sender = fvk_sender.incoming();
    let (sender, _dtk_d) = ivk_sender.payment_address(0u32.into());

    let note = Note::generate(&mut OsRng, &sender, value_to_send);
    let note_commitment = note.commit();
    let spend_auth_randomizer = Fr::from(0u32);
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
    let nf = Nullifier::derive(&nk, state_commitment_proof.position(), &note_commitment);

    let r = Fq::rand(&mut OsRng);
    let s = Fq::rand(&mut OsRng);
    let public = SpendProofPublic {
        anchor,
        balance_commitment,
        nullifier: nf,
        rk,
    };
    let private = SpendProofPrivate {
        state_commitment_proof,
        note,
        v_blinding,
        spend_auth_randomizer,
        ak,
        nk,
    };

    c.bench_function("spend proving", |b| {
        b.iter(|| prove(r, s, public.clone(), private.clone()))
    });

    // Also print out the number of constraints.
    let circuit = SpendCircuit::with_dummy_witness();

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
