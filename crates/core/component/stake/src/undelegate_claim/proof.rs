use ark_groth16::r1cs_to_qap::LibsnarkReduction;
use ark_r1cs_std::uint8::UInt8;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use decaf377::Bls12_377;

use ark_ff::ToConstraintField;
use ark_groth16::{Groth16, PreparedVerifyingKey, Proof, ProvingKey, VerifyingKey};
use ark_r1cs_std::prelude::*;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef};
use ark_snark::SNARK;
use penumbra_proto::{core::crypto::v1alpha1 as pb, DomainType, TypeUrl};
use rand_core::OsRng;

use penumbra_crypto::{
    asset,
    asset::{AmountVar, AssetIdVar},
    balance,
    balance::commitment::BalanceCommitmentVar,
    proofs::groth16::{ParameterSetup, VerifyingKeyExt, GROTH16_PROOF_LENGTH_BYTES},
    Amount, Balance, FieldExt, Fq, Fr, Value, STAKING_TOKEN_ASSET_ID,
};

use crate::{Penalty, PenaltyVar};

#[derive(Clone, Debug)]
pub struct UndelegateClaimCircuit {
    unbonding_amount: Amount,
    balance_blinding: Fr,
    pub balance_commitment: balance::Commitment,
    pub unbonding_id: asset::Id,
    pub penalty: Penalty,
}

impl UndelegateClaimCircuit {
    pub fn new(
        unbonding_amount: Amount,
        balance_blinding: Fr,
        balance_commitment: balance::Commitment,
        unbonding_id: asset::Id,
        penalty: Penalty,
    ) -> Self {
        Self {
            unbonding_amount,
            balance_blinding,
            balance_commitment,
            unbonding_id,
            penalty,
        }
    }
}

impl ConstraintSynthesizer<Fq> for UndelegateClaimCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fq>) -> ark_relations::r1cs::Result<()> {
        // Witnesses
        let unbonding_amount_var =
            AmountVar::new_witness(cs.clone(), || Ok(self.unbonding_amount))?;
        let balance_blinding_arr: [u8; 32] = self.balance_blinding.to_bytes();
        let balance_blinding_vars = UInt8::new_witness_vec(cs.clone(), &balance_blinding_arr)?;

        // Inputs
        let claimed_balance_commitment =
            BalanceCommitmentVar::new_input(cs.clone(), || Ok(self.balance_commitment))?;
        let unbonding_id = AssetIdVar::new_input(cs.clone(), || Ok(self.unbonding_id))?;
        let penalty_var = PenaltyVar::new_input(cs, || Ok(self.penalty))?;

        // Constraints
        let expected_balance = penalty_var.balance_for_claim(unbonding_id, unbonding_amount_var)?;
        let expected_balance_commitment = expected_balance.commit(balance_blinding_vars)?;
        expected_balance_commitment.enforce_equal(&claimed_balance_commitment)?;

        Ok(())
    }
}

impl ParameterSetup for UndelegateClaimCircuit {
    fn generate_test_parameters() -> (ProvingKey<Bls12_377>, VerifyingKey<Bls12_377>) {
        let penalty = Penalty(1);
        let balance_blinding = Fr::from(1);
        let unbonding_amount = Amount::from(1u64);
        let unbonding_id = *STAKING_TOKEN_ASSET_ID;
        let balance_commitment = Balance::from(Value {
            asset_id: unbonding_id,
            amount: unbonding_amount,
        })
        .commit(balance_blinding);

        let circuit = UndelegateClaimCircuit {
            penalty,
            unbonding_amount,
            balance_blinding,
            balance_commitment,
            unbonding_id,
        };
        let (pk, vk) =
            Groth16::<Bls12_377, LibsnarkReduction>::circuit_specific_setup(circuit, &mut OsRng)
                .expect("can perform circuit specific setup");
        (pk, vk)
    }
}

#[derive(Clone, Debug)]
pub struct UndelegateClaimProof([u8; GROTH16_PROOF_LENGTH_BYTES]);

impl UndelegateClaimProof {
    #![allow(clippy::too_many_arguments)]
    /// Generate an `UndelegateClaimProof` given the proving key, public inputs,
    /// witness data, and two random elements `blinding_r` and `blinding_s`.
    pub fn prove(
        blinding_r: Fq,
        blinding_s: Fq,
        pk: &ProvingKey<Bls12_377>,
        unbonding_amount: Amount,
        balance_blinding: Fr,
        balance_commitment: balance::Commitment,
        unbonding_id: asset::Id,
        penalty: Penalty,
    ) -> anyhow::Result<Self> {
        let circuit = UndelegateClaimCircuit {
            unbonding_amount,
            balance_blinding,
            balance_commitment,
            unbonding_id,
            penalty,
        };
        let proof = Groth16::<Bls12_377, LibsnarkReduction>::create_proof_with_reduction(
            circuit, pk, blinding_r, blinding_s,
        )
        .map_err(|err| anyhow::anyhow!(err))?;
        let mut proof_bytes = [0u8; GROTH16_PROOF_LENGTH_BYTES];
        Proof::serialize_compressed(&proof, &mut proof_bytes[..]).expect("can serialize Proof");
        Ok(Self(proof_bytes))
    }

    /// Called to verify the proof using the provided public inputs.
    #[tracing::instrument(level="debug", skip(self, vk), fields(self = ?base64::encode(&self.clone().encode_to_vec()), vk = ?vk.debug_id()))]
    pub fn verify(
        &self,
        vk: &PreparedVerifyingKey<Bls12_377>,
        balance_commitment: balance::Commitment,
        unbonding_id: asset::Id,
        penalty: Penalty,
    ) -> anyhow::Result<()> {
        let proof =
            Proof::deserialize_compressed_unchecked(&self.0[..]).map_err(|e| anyhow::anyhow!(e))?;

        let mut public_inputs = Vec::new();
        public_inputs.extend(balance_commitment.0.to_field_elements().unwrap());
        public_inputs.extend(unbonding_id.0.to_field_elements().unwrap());
        public_inputs.extend(penalty.to_field_elements().unwrap());

        tracing::trace!(?public_inputs);
        let start = std::time::Instant::now();
        let proof_result = Groth16::<Bls12_377, LibsnarkReduction>::verify_with_processed_vk(
            vk,
            public_inputs.as_slice(),
            &proof,
        )
        .map_err(|err| anyhow::anyhow!(err))?;
        tracing::debug!(?proof_result, elapsed = ?start.elapsed());
        proof_result
            .then_some(())
            .ok_or_else(|| anyhow::anyhow!("undelegate claim proof did not verify"))
    }
}

impl TypeUrl for UndelegateClaimProof {
    const TYPE_URL: &'static str = "/penumbra.core.crypto.v1alpha1.ZKUndelegateClaimProof";
}

impl DomainType for UndelegateClaimProof {
    type Proto = pb::ZkUndelegateClaimProof;
}

impl From<UndelegateClaimProof> for pb::ZkUndelegateClaimProof {
    fn from(proof: UndelegateClaimProof) -> Self {
        pb::ZkUndelegateClaimProof {
            inner: proof.0.to_vec(),
        }
    }
}

impl TryFrom<pb::ZkUndelegateClaimProof> for UndelegateClaimProof {
    type Error = anyhow::Error;

    fn try_from(proto: pb::ZkUndelegateClaimProof) -> Result<Self, Self::Error> {
        Ok(UndelegateClaimProof(proto.inner[..].try_into()?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_ff::{PrimeField, UniformRand};
    use decaf377::{Fq, Fr};
    use penumbra_crypto::{rdsa, Amount};
    use proptest::prelude::*;
    use rand_core::OsRng;

    use crate::{IdentityKey, Penalty, UnbondingToken};

    fn fr_strategy() -> BoxedStrategy<Fr> {
        any::<[u8; 32]>()
            .prop_map(|bytes| Fr::from_le_bytes_mod_order(&bytes[..]))
            .boxed()
    }

    proptest! {
    #![proptest_config(ProptestConfig::with_cases(2))]
    #[test]
    fn undelegate_claim_proof_happy_path(validator_randomness in fr_strategy(), balance_blinding in fr_strategy(), value1_amount in 2..200u64, penalty_amount in 0..200u64) {
            let (pk, vk) = UndelegateClaimCircuit::generate_prepared_test_parameters();

            let sk = rdsa::SigningKey::new_from_field(validator_randomness);
            let validator_identity = IdentityKey((&sk).into());
            let unbonding_amount = Amount::from(value1_amount);

            let start_epoch_index = 1;
            let unbonding_token = UnbondingToken::new(validator_identity, start_epoch_index);
            let unbonding_id = unbonding_token.id();
            let penalty = Penalty(penalty_amount);
            let balance = penalty.balance_for_claim(unbonding_id, unbonding_amount);
            let balance_commitment = balance.commit(balance_blinding);

            let blinding_r = Fq::rand(&mut OsRng);
            let blinding_s = Fq::rand(&mut OsRng);
            let proof = UndelegateClaimProof::prove(
                blinding_r,
                blinding_s,
                &pk,
                unbonding_amount,
                balance_blinding,
                balance_commitment,
                unbonding_id,
                penalty
            )
            .expect("can create proof");

            let proof_result = proof.verify(&vk, balance_commitment, unbonding_id, penalty);

            assert!(proof_result.is_ok());
        }
    }
}
