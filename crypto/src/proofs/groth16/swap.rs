use ark_ff::ToConstraintField;
use ark_groth16::{Groth16, PreparedVerifyingKey, Proof, ProvingKey};
use ark_r1cs_std::prelude::*;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef};
use ark_snark::SNARK;
use decaf377::{Bls12_377, FieldExt};
use penumbra_tct as tct;
use rand::{CryptoRng, Rng};

use crate::{
    balance::{self, commitment::BalanceCommitmentVar, BalanceVar},
    dex::swap::{SwapPlaintext, SwapPlaintextVar},
    note::StateCommitmentVar,
    Fq, Fr,
};

pub struct SwapCircuit {
    /// The swap plaintext.
    swap_plaintext: SwapPlaintext,
    /// The blinding factor for the fee commitment.
    fee_blinding: Fr,
    /// Balance commitment of the swap.
    pub balance_commitment: balance::Commitment,
    /// Swap commitment.
    pub swap_commitment: tct::Commitment,
    /// Balance commitment of the fee.
    pub fee_commitment: balance::Commitment,
}

impl ConstraintSynthesizer<Fq> for SwapCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fq>) -> ark_relations::r1cs::Result<()> {
        // Witnesses
        let swap_plaintext_var =
            SwapPlaintextVar::new_witness(cs.clone(), || Ok(self.swap_plaintext.clone()))?;
        let fee_blinding_var = UInt8::new_witness_vec(cs.clone(), &self.fee_blinding.to_bytes())?;

        // Inputs
        let claimed_balance_commitment =
            BalanceCommitmentVar::new_input(cs.clone(), || Ok(self.balance_commitment))?;
        let claimed_swap_commitment =
            StateCommitmentVar::new_input(cs.clone(), || Ok(self.swap_commitment))?;
        let claimed_fee_commitment =
            BalanceCommitmentVar::new_input(cs.clone(), || Ok(self.fee_commitment))?;

        // Swap commitment integrity check
        claimed_swap_commitment.enforce_equal(&swap_plaintext_var.swap_commitment)?;

        // Fee commitment integrity check
        let fee_balance = BalanceVar::from_negative_value_var(swap_plaintext_var.claim_fee.clone());
        let fee_commitment = fee_balance.commit(fee_blinding_var)?;
        claimed_fee_commitment.enforce_equal(&fee_commitment)?;

        // TODO: Reconstruct swap action balance commitment
        // TODO: Balance commitment integrity check

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct SwapProof(Proof<Bls12_377>);

impl SwapProof {
    #![allow(clippy::too_many_arguments)]
    pub fn prove<R: CryptoRng + Rng>(
        rng: &mut R,
        pk: &ProvingKey<Bls12_377>,
        swap_plaintext: SwapPlaintext,
        fee_blinding: Fr,
        balance_commitment: balance::Commitment,
        swap_commitment: tct::Commitment,
        fee_commitment: balance::Commitment,
    ) -> anyhow::Result<Self> {
        let circuit = SwapCircuit {
            swap_plaintext,
            fee_blinding,
            balance_commitment,
            swap_commitment,
            fee_commitment,
        };
        let proof = Groth16::prove(pk, circuit, rng).map_err(|err| anyhow::anyhow!(err))?;
        Ok(Self(proof))
    }

    /// Called to verify the proof using the provided public inputs.
    ///
    /// The public inputs are:
    /// * balance commitment,
    /// * swap commitment,
    /// * fee commimtment,
    ///
    // Commented out, but this may be useful when debugging proof verification failures,
    // to check that the proof data and verification keys are consistent.
    //#[tracing::instrument(skip(self, vk), fields(self = ?base64::encode(&self.clone().encode_to_vec()), vk = ?vk.debug_id()))]
    #[tracing::instrument(skip(self, vk))]
    pub fn verify(
        &self,
        vk: &PreparedVerifyingKey<Bls12_377>,
        balance_commitment: balance::Commitment,
        swap_commitment: tct::Commitment,
        fee_commitment: balance::Commitment,
    ) -> anyhow::Result<()> {
        let mut public_inputs = Vec::new();
        public_inputs.extend(balance_commitment.0.to_field_elements().unwrap());
        public_inputs.extend(swap_commitment.0.to_field_elements().unwrap());
        public_inputs.extend(fee_commitment.0.to_field_elements().unwrap());

        tracing::trace!(?public_inputs);
        let start = std::time::Instant::now();
        let proof_result = Groth16::verify_with_processed_vk(vk, public_inputs.as_slice(), &self.0)
            .map_err(|err| anyhow::anyhow!(err))?;
        tracing::debug!(?proof_result, elapsed = ?start.elapsed());
        proof_result
            .then_some(())
            .ok_or_else(|| anyhow::anyhow!("proof did not verify"))
    }
}
