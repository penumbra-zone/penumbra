use ark_ff::ToConstraintField;
use ark_groth16::{Groth16, PreparedVerifyingKey, Proof, ProvingKey, VerifyingKey};
use ark_r1cs_std::prelude::*;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use ark_snark::SNARK;
use decaf377::{Bls12_377, FieldExt};
use decaf377_fmd as fmd;
use penumbra_proto::{core::crypto::v1alpha1 as pb, DomainType};
use penumbra_tct as tct;
use rand::{CryptoRng, Rng};
use rand_core::OsRng;

use crate::{
    asset, balance,
    dex::{swap::SwapPlaintext, TradingPair},
    ka,
    keys::Diversifier,
    transaction::Fee,
    Address, Fq, Fr, Rseed, Value,
};

use super::{ParameterSetup, GROTH16_PROOF_LENGTH_BYTES};

pub struct SwapClaimCircuit {}

impl ConstraintSynthesizer<Fq> for SwapClaimCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fq>) -> ark_relations::r1cs::Result<()> {
        Ok(())
    }
}

impl ParameterSetup for SwapClaimCircuit {
    fn generate_test_parameters() -> (ProvingKey<Bls12_377>, VerifyingKey<Bls12_377>) {
        let circuit = SwapClaimCircuit {};
        let (pk, vk) = Groth16::circuit_specific_setup(circuit, &mut OsRng)
            .expect("can perform circuit specific setup");
        (pk, vk)
    }
}

#[derive(Clone, Debug)]
pub struct SwapClaimProof(Proof<Bls12_377>);

impl SwapClaimProof {
    #![allow(clippy::too_many_arguments)]
    pub fn prove<R: CryptoRng + Rng>(
        rng: &mut R,
        pk: &ProvingKey<Bls12_377>,
    ) -> anyhow::Result<Self> {
        let circuit = SwapClaimCircuit {};
        let proof = Groth16::prove(pk, circuit, rng).map_err(|err| anyhow::anyhow!(err))?;
        Ok(Self(proof))
    }

    /// Called to verify the proof using the provided public inputs.
    #[tracing::instrument(skip(self, vk))]
    pub fn verify(&self, vk: &PreparedVerifyingKey<Bls12_377>) -> anyhow::Result<()> {
        let mut public_inputs = Vec::new();
        // public_inputs.extend(balance_commitment.0.to_field_elements().unwrap());

        tracing::trace!(?public_inputs);
        let start = std::time::Instant::now();
        let proof_result = Groth16::verify_with_processed_vk(vk, public_inputs.as_slice(), &self.0)
            .map_err(|err| anyhow::anyhow!(err))?;
        tracing::debug!(?proof_result, elapsed = ?start.elapsed());
        proof_result
            .then_some(())
            .ok_or_else(|| anyhow::anyhow!("swapclaim proof did not verify"))
    }
}

impl DomainType for SwapClaimProof {
    type Proto = pb::ZkSwapClaimProof;
}

impl From<SwapClaimProof> for pb::ZkSwapClaimProof {
    fn from(proof: SwapClaimProof) -> Self {
        let mut proof_bytes = [0u8; GROTH16_PROOF_LENGTH_BYTES];
        Proof::serialize(&proof.0, &mut proof_bytes[..]).expect("can serialize Proof");
        pb::ZkSwapClaimProof {
            inner: proof_bytes.to_vec(),
        }
    }
}

impl TryFrom<pb::ZkSwapClaimProof> for SwapClaimProof {
    type Error = anyhow::Error;

    fn try_from(proto: pb::ZkSwapClaimProof) -> Result<Self, Self::Error> {
        Ok(SwapClaimProof(
            Proof::deserialize(&proto.inner[..]).map_err(|e| anyhow::anyhow!(e))?,
        ))
    }
}
