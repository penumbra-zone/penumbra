use std::ops::Deref;

use ark_bls12_377::{Bls12_377, Fr as BlsScalar};
use ark_ec::TEModelParameters;
use ark_ff::{BigInteger, PrimeField};
use ark_poly::univariate::DensePolynomial;
use ark_poly_commit::sonic_pc::SonicKZG10;
use ark_poly_commit::PolynomialCommitment;
use rand::rngs::OsRng;

use ark_ec::models::twisted_edwards_extended::GroupAffine;
use ark_ec::{AffineCurve, ProjectiveCurve};

use ark_ed_on_bls12_377::EdwardsParameters as DecafParameters;
use decaf377::{proof::ComposerExt, Element, Fq, Fr};
use decaf377_ka as ka;

use plonk_core::circuit::{verify_proof, Circuit, PublicInputBuilder};
use plonk_core::constraint_system::StandardComposer;
use plonk_core::error::Error;
use plonk_core::prelude::*;

use crate::{value, Value};

// Public:
// * vcm (value commitment)
// * ncm (note commitment)
// * epk (point)
//
// Witnesses:
// * g_d (point)
// * pk_d (point)
// * v (u64 value plus asset ID (scalar))
// * vblind (Fr)
// * nblind (Fq)
// * esk (scalar)
//
// Output circuits check:
// 1. Diversified base is not identity (implemented).
// 2. Ephemeral public key integrity (partially implemented).
// 3. Value commitment integrity (not implemented).
// 4. Note commitment integrity (not implemented).
#[derive(derivative::Derivative)]
#[derivative(Debug(bound = ""), Default(bound = ""))]
pub struct OutputCircuit<F, P>
where
    F: PrimeField,
    P: TEModelParameters<BaseField = F>,
{
    // Private: The diversified base for the destination address.
    g_d: GroupAffine<P>,
    // Private: The ephemeral secret key that corresponds to the public key.
    esk: F,
    // Private: blinding factor for value commitment.
    v_blinding: Fr,
    // Private: value.
    value: Value,
    // Public: The ephemeral public key.
    epk: GroupAffine<P>,
    // Public: Value commitment.
    value_commitment: value::Commitment,
}

impl<F, P> Circuit<F, P> for OutputCircuit<F, P>
where
    F: PrimeField,
    P: TEModelParameters<BaseField = F>,
    ark_ec::twisted_edwards_extended::GroupAffine<P>: From<Element>,
{
    const CIRCUIT_ID: [u8; 32] = [0xff; 32]; // Replace

    fn gadget(
        &mut self,
        composer: &mut StandardComposer<F, P>,
    ) -> Result<(), plonk_core::error::Error> {
        // The use of decaf means that we do not need to check that the
        // diversified basepoint is of small order. However we instead
        // check it is not identity.
        let g_d = composer.add_affine(self.g_d);
        let identity = decaf377::Element::default();
        let identity_point = composer.add_affine(identity.into());

        // Apply the constraint for the diversified basepoint identity check.
        composer.assert_points_not_equal(g_d, identity_point);

        // Ephemeral public key integrity.
        // Checks that [esk] g_d == epk
        let esk_var = composer.add_input(self.esk);
        let epk_var_computed = composer.variable_base_scalar_mul(esk_var, g_d);
        // Apply the ephemeral public key integrity check constraint.
        composer.assert_equal_public_point(epk_var_computed, self.epk);

        // Value commitment integrity.
        // This checks that: value_commitment == -self.value.commit(self.v_blinding)
        // We do this by computing the inverse of the value commitment as:
        // P = a + b = [v] G_v + [v_blinding] H
        // then taking the inverse of P.
        let value_fq = F::from(self.value.amount);
        let value_var = composer.add_input(value_fq);
        let g_v_element = self.value.asset_id.value_generator().into();
        let g_v_var = composer.add_affine(g_v_element);
        let a_var = composer.variable_base_scalar_mul(value_var, g_v_var);

        let v_blinding_fq = F::from_le_bytes_mod_order(&self.v_blinding.into_repr().to_bytes_le());
        let v_blinding_var = composer.add_input(v_blinding_fq);
        let H_affine = value::VALUE_BLINDING_GENERATOR.deref().clone().into();
        let H_var = composer.add_affine(H_affine);
        let b_var = composer.variable_base_scalar_mul(v_blinding_var, H_var);

        let inv_value_commitment_computed = composer.point_addition_gate(a_var, b_var);

        // Would be nicer to have a `point_inverse` method here like jellyfish PLONK
        let one_var = composer.add_input(F::one());
        let value_commitment_computed =
            composer.conditional_point_neg(one_var, inv_value_commitment_computed);
        let value_commitment_point = composer.add_affine(self.value_commitment.0.into());
        // Instead of adding the point to the circuit, we should use `StandardComposer::assert_equal_public_point`
        // let value_commitment_var = composer.add_input(value_commitment_point);

        // TODO: Add constraint to circuit for value commitment integrity (not added now for comparative
        // benchmarking with jellyfish PLONK)

        // Note commitment integrity.

        dbg!(composer.circuit_size());

        Ok(())
    }

    fn padded_circuit_size(&self) -> usize {
        1 << 13
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::time::Instant;

    use crate::{
        asset,
        keys::{SeedPhrase, SpendKey, SpendSeed},
        Note, Value,
    };
    use ark_bls12_377::Bls12_377;
    use ark_ff::UniformRand;

    use decaf377::{EdwardsParameters, Fq, Fr};
    use decaf377_ka as ka;

    use rand::SeedableRng;
    use rand_chacha::ChaChaRng;

    #[test]
    fn run_zkp_output_proof_happy() {
        let mut rng = ChaChaRng::seed_from_u64(666);

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
        let secret_bytes = esk.to_bytes();
        let esk_fq = Fq::from_le_bytes_mod_order(&secret_bytes);

        // Generate CRS
        type PC = SonicKZG10<Bls12_377, DensePolynomial<BlsScalar>>;
        //type PC = KZG10::<Bls12_381>; //Use a different polynomial commitment
        // scheme

        let pp = PC::setup(1 << 13, None, &mut OsRng).expect("Unable to sample public parameters.");

        let mut circuit = OutputCircuit::<BlsScalar, DecafParameters>::default();
        // Compile the circuit
        let start = Instant::now();
        let (pk_p, verifier_data) = circuit.compile::<PC>(&pp).expect("circuit preprocessing");
        let duration = start.elapsed();
        println!("Time elapsed in proof preprocessing: {:?}", duration);

        let epk_ark_point: GroupAffine<EdwardsParameters> = epk_element.into();

        // Prover POV
        let start = Instant::now();
        let proof = {
            let mut circuit: OutputCircuit<BlsScalar, DecafParameters> = OutputCircuit {
                g_d: g_d.into(),
                esk: esk_fq,
                epk: epk_ark_point,
                v_blinding,
                value: value_to_send,
                value_commitment,
            };
            circuit.gen_proof::<PC>(&pp, pk_p, b"penumbra_OutputProof")
        }
        .expect("can create proof");
        let duration = start.elapsed();
        println!("Time elapsed in proof creation: {:?}", duration);

        // Verifier POV
        let start = Instant::now();
        let public_inputs = PublicInputBuilder::new()
            .add_input(&epk_ark_point)
            .unwrap()
            .finish();
        let VerifierData { key, pi_pos } = verifier_data;
        verify_proof::<BlsScalar, DecafParameters, PC>(
            &pp,
            key,
            &proof,
            &public_inputs,
            &pi_pos, // ?
            b"penumbra_OutputProof",
        )
        .expect("proof should verify");
        let duration = start.elapsed();
        println!("Time elapsed in proof verification: {:?}", duration);
    }
}
