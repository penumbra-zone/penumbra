use ark_relations::r1cs::{
    ConstraintSynthesizer, ConstraintSystem, OptimizationGoal, SynthesisMode,
};
use decaf377::{Fq, Fr};
use penumbra_sdk_asset::Balance;
use penumbra_sdk_proof_params::{DummyWitness, OUTPUT_PROOF_PROVING_KEY};
use penumbra_sdk_shielded_pool::output::{OutputProofPrivate, OutputProofPublic};
use penumbra_sdk_shielded_pool::test_proof_helpers::proof_test_helpers::generate_test_data;
use penumbra_sdk_shielded_pool::{OutputCircuit, OutputProof};

use criterion::{criterion_group, criterion_main, Criterion};
use rand_core::OsRng;

fn output_proving_time(c: &mut Criterion) {
    let mut rng = OsRng;

    // Generate valid test data with compliance encryption
    let test_data = generate_test_data(&mut rng, 1, 100, false); // unregulated for simplicity

    let note_commitment = test_data.note.commit();
    let balance_commitment = (-Balance::from(test_data.value)).commit(test_data.balance_blinding);

    // Create dummy leaves and blinded hashes
    let dummy_leaf = penumbra_sdk_compliance::ComplianceLeaf {
        address: test_data.address.clone(),
        key: test_data.ack.clone(),
        asset_id: test_data.note.asset_id(),
    };
    let dummy_nonce = Fr::from(0u64);
    let receiver_leaf_hash =
        penumbra_sdk_compliance::blind_counterparty_leaf(dummy_leaf.commit(), dummy_nonce);
    let counterparty_leaf_hash =
        penumbra_sdk_compliance::blind_sender_leaf(dummy_leaf.commit(), dummy_nonce);

    let public = OutputProofPublic {
        balance_commitment,
        note_commitment,
        compliance_epk: test_data.compliance_epk,
        compliance_ciphertext: test_data.compliance_ciphertext,
        asset_anchor: test_data.asset_anchor,
        compliance_anchor: test_data.compliance_anchor,
        target_timestamp: test_data.timestamp,
        receiver_leaf_hash,
        counterparty_leaf_hash,
    };

    let private = OutputProofPrivate {
        note: test_data.note,
        balance_blinding: test_data.balance_blinding,
        asset_path: penumbra_sdk_compliance::MerklePath::default(),
        asset_position: 0,
        is_regulated: false,
        compliance_path: penumbra_sdk_compliance::MerklePath::default(),
        compliance_position: 0,
        user_leaf: test_data.user_leaf,
        compliance_ephemeral_secret: test_data.ephemeral_secret,
        counterparty_leaf: dummy_leaf,
        tx_blinding_nonce: dummy_nonce,
    };

    let r = Fq::rand(&mut rng);
    let s = Fq::rand(&mut rng);

    c.bench_function("output proving", |b| {
        b.iter(|| {
            let _proof = OutputProof::prove(
                r,
                s,
                &OUTPUT_PROOF_PROVING_KEY,
                public.clone(),
                private.clone(),
            )
            .expect("can create proof");
        })
    });

    // Also print out the number of constraints.
    let circuit = OutputCircuit::with_dummy_witness();

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

criterion_group!(benches, output_proving_time);
criterion_main!(benches);
