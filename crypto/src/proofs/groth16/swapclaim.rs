use ark_ff::ToConstraintField;
use ark_groth16::{Groth16, PreparedVerifyingKey, Proof, ProvingKey, VerifyingKey};
use ark_r1cs_std::prelude::*;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use ark_snark::SNARK;
use decaf377::{r1cs::FqVar, Bls12_377, FieldExt};
use decaf377_fmd as fmd;
use penumbra_proto::{core::crypto::v1alpha1 as pb, DomainType};
use penumbra_tct as tct;
use rand::{CryptoRng, Rng};
use rand_core::OsRng;

use crate::{
    asset, balance,
    dex::{
        swap::{SwapPlaintext, SwapPlaintextVar},
        TradingPair,
    },
    ka,
    keys::Diversifier,
    note,
    transaction::Fee,
    Address, Fq, Fr, Rseed, Value,
};

use super::{ParameterSetup, GROTH16_PROOF_LENGTH_BYTES};

/// SwapClaim consumes an existing Swap NFT so they are most similar to Spend operations,
/// however the note commitment proof needs to be for a specific block due to clearing prices
/// only being valid for particular blocks (i.e. the exchange rates of assets change over time).
pub struct SwapClaimCircuit {
    /// The swap being claimed
    swap_plaintext: SwapPlaintext,
    /// Inclusion proof for the swap commitment
    state_commitment_proof: tct::Proof,
    /// Anchor
    pub anchor: tct::Root,
}

impl ConstraintSynthesizer<Fq> for SwapClaimCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fq>) -> ark_relations::r1cs::Result<()> {
        // Witnesses
        let swap_plaintext_var =
            SwapPlaintextVar::new_witness(cs.clone(), || Ok(self.swap_plaintext.clone()))?;

        let claimed_swap_commitment = note::StateCommitmentVar::new_witness(cs.clone(), || {
            Ok(self.state_commitment_proof.commitment())
        })?;

        let position_var = tct::r1cs::PositionVar::new_witness(cs.clone(), || {
            Ok(self.state_commitment_proof.position())
        })?;
        let merkle_path_var =
            tct::r1cs::MerkleAuthPathVar::new(cs.clone(), self.state_commitment_proof)?;

        // Inputs
        let anchor_var = FqVar::new_input(cs.clone(), || Ok(Fq::from(self.anchor)))?;

        // Swap commitment integrity check
        let swap_commitment = swap_plaintext_var.commit()?;
        claimed_swap_commitment.enforce_equal(&swap_commitment)?;

        // Merkle path integrity. Ensure the provided note commitment is in the TCT.
        merkle_path_var.verify(
            cs.clone(),
            &Boolean::TRUE,
            position_var.inner,
            anchor_var,
            claimed_swap_commitment.inner(),
        )?;

        Ok(())
    }
}

impl ParameterSetup for SwapClaimCircuit {
    fn generate_test_parameters() -> (ProvingKey<Bls12_377>, VerifyingKey<Bls12_377>) {
        let trading_pair = TradingPair {
            asset_1: asset::REGISTRY.parse_denom("upenumbra").unwrap().id(),
            asset_2: asset::REGISTRY.parse_denom("nala").unwrap().id(),
        };
        let diversifier_bytes = [1u8; 16];
        let pk_d_bytes = decaf377::basepoint().vartime_compress().0;
        let clue_key_bytes = [1; 32];
        let diversifier = Diversifier(diversifier_bytes);
        let address = Address::from_components(
            diversifier,
            ka::Public(pk_d_bytes),
            fmd::ClueKey(clue_key_bytes),
        )
        .expect("generated 1 address");
        let swap_plaintext = SwapPlaintext {
            trading_pair,
            delta_1_i: 100000u64.into(),
            delta_2_i: 1u64.into(),
            claim_fee: Fee(Value {
                amount: 3u64.into(),
                asset_id: asset::REGISTRY.parse_denom("upenumbra").unwrap().id(),
            }),
            claim_address: address,
            rseed: Rseed([1u8; 32]),
        };
        let mut sct = tct::Tree::new();
        let swap_commitment = swap_plaintext.swap_commitment();
        sct.insert(tct::Witness::Keep, swap_commitment).unwrap();
        let anchor = sct.root();
        let state_commitment_proof = sct.witness(swap_commitment).unwrap();

        let circuit = SwapClaimCircuit {
            swap_plaintext,
            state_commitment_proof,
            anchor,
        };
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
        swap_plaintext: SwapPlaintext,
        state_commitment_proof: tct::Proof,
        anchor: tct::Root,
    ) -> anyhow::Result<Self> {
        let circuit = SwapClaimCircuit {
            swap_plaintext,
            state_commitment_proof,
            anchor,
        };
        let proof = Groth16::prove(pk, circuit, rng).map_err(|err| anyhow::anyhow!(err))?;
        Ok(Self(proof))
    }

    /// Called to verify the proof using the provided public inputs.
    pub fn verify(
        &self,
        vk: &PreparedVerifyingKey<Bls12_377>,
        anchor: tct::Root,
    ) -> anyhow::Result<()> {
        let mut public_inputs = Vec::new();
        public_inputs.extend(Fq::from(anchor.0).to_field_elements().unwrap());

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
