use std::ops::Deref;

use ark_ec::{
    twisted_edwards_extended::GroupAffine, ModelParameters, PairingEngine, TEModelParameters,
};
use ark_ff::{BigInteger, PrimeField};
use ark_serialize::CanonicalDeserialize;
use decaf377::{EdwardsParameters, Fq, Fr};
use jf_plonk::{
    circuit::{customized::ecc::Point, Circuit, PlonkCircuit},
    errors::PlonkError,
};
use jf_utils::fr_to_fq;

use crate::{value, Value};
use decaf377_ka as ka;

fn output_proof_circuit<EmbedCurve, PairingCurve>(
    esk: ka::Secret,
    epk: decaf377::Element,
    g_d: decaf377::Element,
    v_blinding: EmbedCurve::ScalarField,
    value_commitment: value::Commitment,
    value: Value,
) -> Result<PlonkCircuit<EmbedCurve::BaseField>, PlonkError>
where
    EmbedCurve: TEModelParameters + Clone,
    <EmbedCurve as ModelParameters>::BaseField: PrimeField,
    PairingCurve: PairingEngine,
    Point<<EmbedCurve as ModelParameters>::BaseField>: From<decaf377::Element>,
{
    let mut circuit = PlonkCircuit::<EmbedCurve::BaseField>::new_turbo_plonk();

    // The use of decaf means that we do not need to check that the
    // diversified basepoint is of small order. However we instead
    // check it is not identity.
    let identity_jf: Point<EmbedCurve::BaseField> = decaf377::Element::default().into();
    let identity_var = circuit.create_constant_point_variable(identity_jf)?;

    let g_d_jf: Point<EmbedCurve::BaseField> = g_d.into();
    let g_d_var = circuit.create_public_point_variable(g_d_jf)?;

    // Connect wires for diversified basepoint identity check.
    let equality_check_computed = circuit.is_equal_point(&identity_var, &g_d_var)?;
    circuit.enforce_false(equality_check_computed)?;

    // Ephemeral public key integrity.
    // Checks that [esk] g_d == epk
    let secret_bytes = esk.to_bytes();
    let esk_fq = fr_to_fq::<_, EmbedCurve>(
        &EmbedCurve::ScalarField::deserialize(&secret_bytes[..]).expect("should be infallible"),
    );
    let esk_var = circuit.create_variable(esk_fq)?;
    let epk_jf: Point<EmbedCurve::BaseField> = epk.into();
    let epk_var = circuit.create_public_point_variable(epk_jf)?;

    // Connect wires for ephemeral public key integrity check.
    let epk_var_computed = circuit.variable_base_scalar_mul::<EmbedCurve>(esk_var, &g_d_var)?;
    circuit.point_equal_gate(&epk_var_computed, &epk_var)?;

    // Value commitment integrity.
    // This checks that: value_commitment == -self.value.commit(self.v_blinding)
    // We do this by computing the inverse of the value commitment as:
    // P = a + b = [v] G_v + [v_blinding] H
    // then taking the inverse of P.
    //
    // Creating `value_fq` as below causes a `WrongQuotientPolyDegree` error when creating the proof,
    // unclear why:
    // `let value_fq = fr_to_fq::<_, EmbedCurve>(&EmbedCurve::Scalar::from(value.amount));`
    // Creating `value_fq` as follows does not cause a `WrongQuotientPolyDegree` error:
    let value_fq = EmbedCurve::BaseField::from(value.amount);
    let value_var = circuit.create_variable(value_fq)?;
    let g_v_element = value.asset_id.value_generator().into();
    let g_v_var = circuit.create_public_point_variable(g_v_element)?;
    let a_var = circuit.variable_base_scalar_mul::<EmbedCurve>(value_var, &g_v_var)?;

    // `v_blinding` is over `P::ScalarField`, lift to `P::BaseField`.
    // This also causes a `WrongQuotientPolyDegree` error:
    //let v_blinding_fq = fr_to_fq::<_, EmbedCurve>(&v_blinding);
    // let v_blinding_fq =
    //     EmbedCurve::BaseField::from_le_bytes_mod_order(&v_blinding.into_repr().to_bytes_le());
    // let v_blinding_var = circuit.create_variable(v_blinding_fq)?;
    // let H_jf: Point<EmbedCurve::BaseField> = value::VALUE_BLINDING_GENERATOR.deref().clone().into();
    // let H_var = circuit.create_public_point_variable(H_jf)?;
    // let b_var = circuit.variable_base_scalar_mul::<EmbedCurve>(v_blinding_var, &H_var)?;
    // let b_var = circuit.variable_base_scalar_mul::<EmbedCurve>(value_var, &H_var)?;

    // let inv_value_commitment_computed = circuit.ecc_add::<EmbedCurve>(&a_var, &b_var)?;
    // let value_commitment_computed = circuit.inverse_point(&inv_value_commitment_computed)?;
    // let value_commitment_jf: Point<EmbedCurve::BaseField> = value_commitment.0.into();
    // let value_commitment_var = circuit.create_public_point_variable(value_commitment_jf)?;
    // Connect wires for value commitment integrity check.
    // The below blows up also with `WrongQuotientPolyDegree` error.
    // circuit.point_equal_gate(&value_commitment_computed, &value_commitment_var)?;

    // TODO: Note commitment integrity.
    // Requires Poseidon377 gadget

    circuit.finalize_for_arithmetization()?;
    Ok(circuit)
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use super::*;
    use crate::{
        asset,
        keys::{SeedPhrase, SpendKey, SpendSeed},
        Note, Value,
    };
    use ark_bls12_377::Bls12_377;
    use ark_ff::UniformRand;

    use decaf377::{EdwardsParameters, Fq, Fr};
    use decaf377_ka as ka;
    use jf_plonk::{
        circuit::Arithmetization,
        proof_system::{PlonkKzgSnark, Snark},
        transcript::StandardTranscript,
    };
    use rand_core::OsRng;

    #[test]
    fn zk_output_proof_run() {
        let mut rng = OsRng;

        let seed_phrase = SeedPhrase::generate(&mut rng);
        let spend_seed = SpendSeed::from_seed_phrase(seed_phrase, 0);
        let sk_recipient = SpendKey::new(spend_seed);
        let fvk_recipient = sk_recipient.full_viewing_key();
        let ivk_recipient = fvk_recipient.incoming();
        let (dest, _dtk_d) = ivk_recipient.payment_address(0u64.into());

        let value_to_send = Value {
            amount: 10,
            asset_id: asset::REGISTRY.parse_denom("upenumbra").unwrap().id(),
        };
        let v_blinding = Fr::rand(&mut rng);
        let note = Note::generate(&mut rng, &dest, value_to_send);
        let esk = ka::Secret::new(&mut rng);
        let epk = esk.diversified_public(&note.diversified_generator());
        let epk_element = decaf377::Encoding(epk.0)
            .decompress()
            .expect("valid pubkey");
        let value_commitment = value_to_send.commit(v_blinding);

        let g_d = *dest.diversified_generator();

        // TODO: The below requires a gadget for `decaf377::Element` (same for all other conversions into
        // `Point<EmbedCurve::BaseField>` from `decaf377::Element`)
        let circuit = output_proof_circuit::<EdwardsParameters, Bls12_377>(
            esk,
            epk_element,
            g_d,
            v_blinding,
            value_commitment,
            value_to_send,
        )
        .expect("can create output proof circuit");

        // Simulate universal setup and get the structured reference string (SRS).
        // TODO: Number of constraints per action, size of this SRS (memory usage)
        let srs_size = circuit.srs_size().expect("can compute SRS size");
        // 8194 so far (excluding the note commitment integrity check)
        dbg!(srs_size);
        // let srs = PlonkKzgSnark::<Bls12_381>::universal_setup(srs_size, &mut rng)
        //     .expect("can do trusted setup");
        let srs = PlonkKzgSnark::<Bls12_377>::universal_setup(srs_size, &mut rng)
            .expect("can do trusted setup");

        // Generate the proving key and verification key from the structured reference string and circuit.
        let start = Instant::now();
        let (pk, vk) =
            PlonkKzgSnark::<Bls12_377>::preprocess(&srs, &circuit).expect("can generate pk and vk");
        let duration = start.elapsed();
        println!("Time elapsed in proof preprocessing: {:?}", duration);

        // Proof generation step. The proof uses the `StandardTranscript` for the Fiat-Shamir transformation.
        let start = Instant::now();
        let proof = PlonkKzgSnark::<Bls12_377>::prove::<_, _, StandardTranscript>(
            &mut rng, &circuit, &pk, None,
        )
        .expect("proof should be created");
        let duration = start.elapsed();
        println!("Time elapsed in proof creation: {:?}", duration);

        // Verification step.
        let public_inputs = circuit.public_input().unwrap();
        let extra_transcript_init_msg = None; // Not using any extra messages
        let start = Instant::now();
        assert!(PlonkKzgSnark::<Bls12_377>::verify::<StandardTranscript>(
            &vk,
            &public_inputs,
            &proof,
            extra_transcript_init_msg,
        )
        .is_ok());
        let duration = start.elapsed();
        println!("Time elapsed in proof verification: {:?}", duration);
    }
}
