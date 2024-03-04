use ark_relations::r1cs::{
    ConstraintSynthesizer, ConstraintSystem, OptimizationGoal, SynthesisMode,
};
use decaf377::Fq;
use penumbra_asset::asset;
use penumbra_dex::{
    swap::SwapPlaintext,
    swap_claim::{SwapClaimCircuit, SwapClaimProof, SwapClaimProofPrivate, SwapClaimProofPublic},
    BatchSwapOutputData, TradingPair,
};
use penumbra_fee::Fee;
use penumbra_keys::keys::{Bip44Path, SeedPhrase, SpendKey};
use penumbra_num::Amount;
use penumbra_proof_params::{DummyWitness, SWAPCLAIM_PROOF_PROVING_KEY};
use penumbra_sct::Nullifier;
use penumbra_tct as tct;

use criterion::{criterion_group, criterion_main, Criterion};
use rand_core::OsRng;

#[allow(clippy::too_many_arguments)]
fn prove(r: Fq, s: Fq, public: SwapClaimProofPublic, private: SwapClaimProofPrivate) {
    let _proof = SwapClaimProof::prove(r, s, &SWAPCLAIM_PROOF_PROVING_KEY, public, private)
        .expect("can create proof");
}

fn swap_claim_proving_time(c: &mut Criterion) {
    let seed_phrase = SeedPhrase::generate(OsRng);
    let sk_recipient = SpendKey::from_seed_phrase_bip44(seed_phrase, &Bip44Path::new(0));
    let fvk_recipient = sk_recipient.full_viewing_key();
    let ivk_recipient = fvk_recipient.incoming();
    let (claim_address, _dtk_d) = ivk_recipient.payment_address(0u32.into());
    let nk = *sk_recipient.nullifier_key();

    let gm = asset::Cache::with_known_assets().get_unit("gm").unwrap();
    let gn = asset::Cache::with_known_assets().get_unit("gn").unwrap();
    let trading_pair = TradingPair::new(gm.id(), gn.id());

    let delta_1_i = Amount::from(2u64);
    let delta_2_i = Amount::from(0u64);
    let fee = Fee::default();

    let swap_plaintext = SwapPlaintext::new(
        &mut OsRng,
        trading_pair,
        delta_1_i,
        delta_2_i,
        fee,
        claim_address,
    );
    let claim_fee = swap_plaintext.clone().claim_fee;
    let mut sct = tct::Tree::new();
    let swap_commitment = swap_plaintext.swap_commitment();
    sct.insert(tct::Witness::Keep, swap_commitment).unwrap();
    let anchor = sct.root();
    let state_commitment_proof = sct.witness(swap_commitment).unwrap();
    let position = state_commitment_proof.position();
    let nullifier = Nullifier::derive(&nk, position, &swap_commitment);
    let epoch_duration = 20;
    let height = epoch_duration * position.epoch() + position.block();

    let output_data = BatchSwapOutputData {
        delta_1: Amount::from(1000u64),
        delta_2: Amount::from(1000u64),
        lambda_1: Amount::from(50u64),
        lambda_2: Amount::from(25u64),
        unfilled_1: Amount::from(23u64),
        unfilled_2: Amount::from(50u64),
        height: height.into(),
        trading_pair: swap_plaintext.trading_pair,
        epoch_starting_height: (epoch_duration * position.epoch()).into(),
    };
    let (lambda_1, lambda_2) = output_data.pro_rata_outputs((delta_1_i, delta_2_i));

    let (output_rseed_1, output_rseed_2) = swap_plaintext.output_rseeds();
    let note_blinding_1 = output_rseed_1.derive_note_blinding();
    let note_blinding_2 = output_rseed_2.derive_note_blinding();
    let (output_1_note, output_2_note) = swap_plaintext.output_notes(&output_data);
    let note_commitment_1 = output_1_note.commit();
    let note_commitment_2 = output_2_note.commit();

    let public = SwapClaimProofPublic {
        anchor,
        nullifier,
        claim_fee,
        output_data,
        note_commitment_1,
        note_commitment_2,
    };
    let private = SwapClaimProofPrivate {
        swap_plaintext,
        state_commitment_proof,
        nk,
        lambda_1,
        lambda_2,
        note_blinding_1,
        note_blinding_2,
    };

    let r = Fq::rand(&mut OsRng);
    let s = Fq::rand(&mut OsRng);

    c.bench_function("swap claim proving", |b| {
        b.iter(|| prove(r, s, public.clone(), private.clone()))
    });

    // Also print out the number of constraints.
    let circuit = SwapClaimCircuit::with_dummy_witness();

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

criterion_group!(benches, swap_claim_proving_time);
criterion_main!(benches);
