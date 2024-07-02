use std::str::FromStr;

use ark_relations::r1cs::{
    ConstraintSynthesizer, ConstraintSystem, OptimizationGoal, SynthesisMode,
};
use decaf377::Fq;
use penumbra_asset::Value;
use penumbra_keys::keys::{Bip44Path, SeedPhrase, SpendKey};
use penumbra_proof_params::{DummyWitness, NULLIFIER_DERIVATION_PROOF_PROVING_KEY};
use penumbra_sct::Nullifier;
use penumbra_shielded_pool::{
    Note, NullifierDerivationProofPrivate, NullifierDerivationProofPublic, Rseed,
};
use penumbra_shielded_pool::{NullifierDerivationCircuit, NullifierDerivationProof};
use penumbra_tct as tct;

use criterion::{criterion_group, criterion_main, Criterion};
use rand_core::OsRng;

fn prove(public: NullifierDerivationProofPublic, private: NullifierDerivationProofPrivate) {
    let _proof = NullifierDerivationProof::prove(
        &mut OsRng,
        &NULLIFIER_DERIVATION_PROOF_PROVING_KEY,
        public,
        private,
    )
    .expect("Can generate proof");
}

fn nullifier_derivation_proving_time(c: &mut Criterion) {
    let seed_phrase = SeedPhrase::generate(OsRng);
    let sk_sender = SpendKey::from_seed_phrase_bip44(seed_phrase, &Bip44Path::new(0));
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
    let nullifier = Nullifier(Fq::from(1u64));
    let mut sct = tct::Tree::new();
    let note_commitment = note.commit();
    sct.insert(tct::Witness::Keep, note_commitment).unwrap();
    let state_commitment_proof = sct.witness(note_commitment).unwrap();
    let position = state_commitment_proof.position();
    let public = NullifierDerivationProofPublic {
        position,
        note_commitment,
        nullifier,
    };
    let private = NullifierDerivationProofPrivate { nk };

    c.bench_function("nullifier derivation proving", |b| {
        b.iter(|| prove(public.clone(), private.clone()))
    });

    let circuit = NullifierDerivationCircuit::with_dummy_witness();
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
