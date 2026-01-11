use ark_bls12_377::{Config, Fq as ArkFq, Fr, G1Projective, G2Projective};
use ark_ec::Group;
use ark_ff::UniformRand;
use ark_r1cs_std::groups::bls12::{G1Var, G2Var};
use ark_r1cs_std::pairing::bls12::PairingVar;
use ark_r1cs_std::pairing::PairingVar as PairingVarTrait;
use ark_r1cs_std::prelude::*;
use ark_relations::r1cs::{
    ConstraintSynthesizer, ConstraintSystem, ConstraintSystemRef, OptimizationGoal, SynthesisError,
    SynthesisMode,
};
use decaf377::Fq;
use penumbra_sdk_proof_params::DummyWitness;
use penumbra_sdk_shielded_pool::{OutputCircuit, SpendCircuit};

/// BLS threshold signature verification circuit
struct BlsThresholdCircuit {
    aggregate_sig: G1Projective,
    aggregate_pubkey: G2Projective,
    message_hash: G1Projective,
}

impl ConstraintSynthesizer<ArkFq> for BlsThresholdCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<ArkFq>) -> Result<(), SynthesisError> {
        let sig_var = G1Var::<Config>::new_witness(cs.clone(), || Ok(self.aggregate_sig))?;
        let pubkey_var = G2Var::<Config>::new_witness(cs.clone(), || Ok(self.aggregate_pubkey))?;
        let msg_var = G1Var::<Config>::new_witness(cs.clone(), || Ok(self.message_hash))?;
        let g2_gen = G2Var::<Config>::new_constant(cs.clone(), G2Projective::generator())?;

        let prepared_sig = PairingVar::<Config>::prepare_g1(&sig_var)?;
        let prepared_g2 = PairingVar::<Config>::prepare_g2(&g2_gen)?;
        let prepared_msg = PairingVar::<Config>::prepare_g1(&msg_var)?;
        let prepared_pubkey = PairingVar::<Config>::prepare_g2(&pubkey_var)?;

        let lhs = PairingVar::<Config>::pairing(prepared_sig, prepared_g2)?;
        let rhs = PairingVar::<Config>::pairing(prepared_msg, prepared_pubkey)?;
        lhs.enforce_equal(&rhs)?;

        Ok(())
    }
}

impl BlsThresholdCircuit {
    fn generate_valid() -> Self {
        let mut rng = ark_std::test_rng();
        let sk = Fr::rand(&mut rng);
        let pubkey = G2Projective::generator() * sk;
        let msg = G1Projective::rand(&mut rng);
        let sig = msg * sk;
        Self {
            aggregate_sig: sig,
            aggregate_pubkey: pubkey,
            message_hash: msg,
        }
    }
}

fn count_constraints<C: ConstraintSynthesizer<Fq>>(circuit: C) -> usize {
    let cs = ConstraintSystem::<Fq>::new_ref();
    cs.set_optimization_goal(OptimizationGoal::Constraints);
    cs.set_mode(SynthesisMode::Setup);
    circuit
        .generate_constraints(cs.clone())
        .expect("can generate constraints");
    cs.finalize();
    cs.num_constraints()
}

fn count_bls_constraints() -> usize {
    let circuit = BlsThresholdCircuit::generate_valid();
    let cs = ConstraintSystem::<ArkFq>::new_ref();
    cs.set_optimization_goal(OptimizationGoal::Constraints);
    cs.set_mode(SynthesisMode::Setup);
    circuit
        .generate_constraints(cs.clone())
        .expect("can generate constraints");
    cs.finalize();
    cs.num_constraints()
}

fn measure_bls_time() -> std::time::Duration {
    let start = std::time::Instant::now();
    let circuit = BlsThresholdCircuit::generate_valid();
    let cs = ConstraintSystem::<ArkFq>::new_ref();
    cs.set_optimization_goal(OptimizationGoal::Constraints);
    cs.set_mode(SynthesisMode::Setup);
    circuit
        .generate_constraints(cs.clone())
        .expect("can generate constraints");
    cs.finalize();
    start.elapsed()
}

fn measure_time<C: ConstraintSynthesizer<Fq>, F: Fn() -> C>(f: F) -> std::time::Duration {
    let start = std::time::Instant::now();
    let circuit = f();
    let cs = ConstraintSystem::<Fq>::new_ref();
    cs.set_optimization_goal(OptimizationGoal::Constraints);
    cs.set_mode(SynthesisMode::Setup);
    circuit
        .generate_constraints(cs.clone())
        .expect("can generate constraints");
    cs.finalize();
    start.elapsed()
}

#[test]
fn print_constraint_counts() {
    // Warm up
    let _ = count_bls_constraints();
    let _ = count_constraints(SpendCircuit::with_dummy_witness());
    let _ = count_constraints(OutputCircuit::with_dummy_witness());

    // Measure
    let bls_count = count_bls_constraints();
    let spend_count = count_constraints(SpendCircuit::with_dummy_witness());
    let output_count = count_constraints(OutputCircuit::with_dummy_witness());

    // Time (average of 3 runs)
    let bls_time: std::time::Duration = (0..3)
        .map(|_| measure_bls_time())
        .sum::<std::time::Duration>()
        / 3;
    let spend_time: std::time::Duration = (0..3)
        .map(|_| measure_time(SpendCircuit::with_dummy_witness))
        .sum::<std::time::Duration>()
        / 3;
    let output_time: std::time::Duration = (0..3)
        .map(|_| measure_time(OutputCircuit::with_dummy_witness))
        .sum::<std::time::Duration>()
        / 3;

    println!();
    println!("=== Circuit Constraint Counts & Timing ===");
    println!(
        "BLS Threshold: {} constraints, {:?} constraint gen",
        bls_count, bls_time
    );
    println!(
        "Spend:         {} constraints, {:?} constraint gen",
        spend_count, spend_time
    );
    println!(
        "Output:        {} constraints, {:?} constraint gen",
        output_count, output_time
    );
    println!();
    println!("CSV:");
    println!("circuit,constraints,constraint_gen_ms");
    println!(
        "BLS Threshold,{},{:.2}",
        bls_count,
        bls_time.as_secs_f64() * 1000.0
    );
    println!(
        "Spend,{},{:.2}",
        spend_count,
        spend_time.as_secs_f64() * 1000.0
    );
    println!(
        "Output,{},{:.2}",
        output_count,
        output_time.as_secs_f64() * 1000.0
    );
}
